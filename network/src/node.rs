use crate::commands::UserCommand;
use crate::network::{
  bootstrap_init, local_balance, request_latest_block, request_specific_block, NumPeersConsensus,
};
use crate::protocol::{BchainRequest, BchainResponse, Frame};
use crate::swarm::{create_swarm, BchainSwarm};
use async_std::channel::{self, Receiver, Sender};
use async_std::prelude::FutureExt;
use async_std::sync::{Mutex, RwLock};
use async_std::{io, task};
use bchain_db::database::{create_db, Db};
use bchain_domain::address::Address;
use bchain_domain::block::Block;
use bchain_domain::tx::Tx;
use bchain_domain::{cli::Cli, wallet::Wallet};
use bchain_util::group::peer_majority;
use bchain_util::result::AppResult;
use bchain_util::short::ShortDisplay;
use futures::{prelude::*, select};
use libp2p::gossipsub::{error::GossipsubHandlerError, GossipsubEvent, IdentTopic as Topic};
use libp2p::{identity, swarm::SwarmEvent};
use log::{debug, error, info, warn};
use std::{sync::Arc, time::Duration};

type Channel<T> = (Sender<T>, Receiver<T>);

pub struct Node {
  cli: Cli,
  topic: Topic,
  db: Arc<Mutex<Db>>,
  wallet: Arc<RwLock<Wallet>>,
  swarm: BchainSwarm,
  #[allow(dead_code)]
  bootstrapped: Arc<RwLock<bool>>,

  network_latest: Channel<Block>,
  network_blocks: Channel<Block>,
  #[allow(dead_code)]
  accepted_blocks: Channel<Block>,
  #[allow(dead_code)]
  accepted_tx: Channel<Tx>,

  network_responses: Channel<BchainResponse>,
  network_requests: Channel<BchainRequest>,
}

impl Node {
  pub async fn new(cli: &Cli) -> AppResult<Node> {
    let topic = Topic::new(&cli.net);

    let wallet = Wallet::from_file(&cli.wallet).await?;
    let mut rsa_pkcs8 = wallet.to_pkcs8_der()?;
    let local_peer_key = identity::Keypair::rsa_from_pkcs8(&mut rsa_pkcs8)?;

    let db = create_db(&cli.database)?;

    let swarm = create_swarm(&local_peer_key, &topic).await?;
    let cli = cli.clone();
    Ok(Node {
      cli,
      topic,
      swarm,
      db: Arc::new(Mutex::new(db)),
      wallet: Arc::new(RwLock::new(wallet)),
      bootstrapped: Arc::new(RwLock::new(false)),
      network_latest: channel::unbounded(),
      network_blocks: channel::unbounded(),
      accepted_blocks: channel::unbounded(),
      accepted_tx: channel::unbounded(),
      network_responses: channel::unbounded(),
      network_requests: channel::unbounded(),
    })
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self.swarm.listen_on(self.cli.listen.parse()?)?;

    self.dial_peers(self.cli.peers.clone())?;

    let mut bootstrap = Box::pin(
      async {}
        .delay(Duration::from_secs(self.cli.delay as u64))
        .fuse(),
    );

    let mut cmd_lines = io::BufReader::new(io::stdin()).lines();

    let (_, mut network_responses) = self.network_responses.clone();
    let (_, mut network_requests) = self.network_requests.clone();

    loop {
      select! {
        _ = bootstrap => self.bootstrap()?,
        response = network_responses.next().fuse() => {
          if let Some(ref response) = response {
            self.publish_response(response)?;
          }
        },
        request = network_requests.next().fuse() => {
          if let Some(ref request) = request {
            self.publish_request(request)?;
          }
        },
        event = self.swarm.next().fuse() => {
          if let Some(ref swarm_event) = event {
            self.handle_swarm_event(swarm_event)?;
          }
        },
        cmd_line = cmd_lines.next().fuse() => {
          if let Some(Ok(ref line)) = cmd_line {
            let command = line.parse()?;
            self.handle_user_command(&command)?;
          }
        },
        complete => break,
      }
    }

    Ok(())
  }

  fn dial_peers<P: IntoIterator<Item = String>>(&mut self, peers: P) -> AppResult<()> {
    for peer in peers {
      let to_dial = peer.clone().parse();
      match to_dial {
        Ok(to_dial) => match self.swarm.dial_addr(to_dial) {
          Ok(_) => info!("Dialed {:?}", peer),
          Err(e) => warn!("Dialing {:?} failed: {:?}", peer, e),
        },
        Err(err) => error!("Failed to parse address to dial: {:?}", err),
      }
    }
    Ok(())
  }

  fn handle_user_command(&mut self, cmd: &UserCommand) -> AppResult<()> {
    match cmd {
      UserCommand::Peers => self.display_peers(),
      &UserCommand::Blocks => self.display_blocks(),
      UserCommand::Bootstrap => self.bootstrap()?,
      UserCommand::Dial(peers) => self.dial_peers(peers.clone())?,
      UserCommand::Msg(msg) => self.publish_user_message(msg),
      UserCommand::Balance(address) => self.print_balance(address),
      UserCommand::Tx(address, amount) => self.submit_tx(address, *amount),
      UserCommand::Unrecognized => warn!("Unrecognized user input"),
    }
    Ok(())
  }

  fn handle_swarm_event(
    &mut self,
    event: &SwarmEvent<GossipsubEvent, GossipsubHandlerError>,
  ) -> AppResult<()> {
    match event {
      SwarmEvent::Behaviour(GossipsubEvent::Message { message, .. }) => {
        match serde_json::from_slice(&message.data)? {
          Frame::BchainRequest(request) => self.handle_bchain_request(request),
          Frame::BchainResponse(response) => self.handle_bchain_response(response),
          _ => warn!("Unrecognized bchain event"),
        }
      }
      SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {:?}", address),
      _ => (),
    }
    Ok(())
  }

  fn handle_bchain_request(&mut self, request: BchainRequest) {
    info!("Network request: {:?}", request);
    match request {
      BchainRequest::AskLatest => self.respond_latest_block(),
      BchainRequest::AskBlock(id) => self.respond_block(id),
      BchainRequest::Msg(msg) => info!("{}", msg),
      _ => warn!("Unhandled bchain request"),
    }
  }

  fn handle_bchain_response(&mut self, response: BchainResponse) {
    debug!("Network response: {:?}", response);
    match response {
      BchainResponse::Latest(block) => {
        let (network_latest_sender, _) = self.network_latest.clone();
        task::spawn(async move {
          network_latest_sender.send(block).await?;
          Ok(()) as AppResult<()>
        });
      }
      BchainResponse::Block(block) => {
        let (network_block_sender, _) = self.network_blocks.clone();
        task::spawn(async move {
          network_block_sender.send(block).await?;
          Ok(()) as AppResult<()>
        });
      }
      BchainResponse::AcceptBlock(block) => {
        task::spawn(async move { block });
      }
      BchainResponse::AcceptTx(block) => {
        task::spawn(async move { block });
      }
      BchainResponse::Error(err) => error!("{:?}", err),
    }
  }

  fn publish_response(&mut self, res: &BchainResponse) -> AppResult<()> {
    self.publish_to_swarm(&Frame::BchainResponse(res.clone()))?;
    Ok(())
  }

  fn publish_request(&mut self, req: &BchainRequest) -> AppResult<()> {
    self.publish_to_swarm(&Frame::BchainRequest(req.clone()))?;
    Ok(())
  }

  fn publish_to_swarm(&mut self, frame: &Frame) -> AppResult<()> {
    let bytes = serde_json::to_vec(frame)?;
    let publish_result = self
      .swarm
      .behaviour_mut()
      .publish(self.topic.clone(), bytes);

    if let Err(e) = publish_result {
      error!("{:?}", e);
    }
    Ok(())
  }

  fn num_peers_consensus(&self) -> NumPeersConsensus {
    let num_peers = self.swarm.network_info().num_peers();
    (num_peers, peer_majority(num_peers))
  }

  fn display_peers(&self) {
    info!("Peers: {}", self.num_peers_consensus().0);
  }

  fn display_blocks(&self) {
    let db = self.db.clone();
    task::spawn(async move {
      let recent = db.lock().await.recent_blocks(10)?;
      for block in recent {
        info!("{}", block);
      }
      Ok(()) as AppResult<()>
    });
  }

  fn respond_latest_block(&mut self) {
    let db = self.db.clone();
    let (send_network_response, _) = self.network_responses.clone();
    task::spawn(async move {
      if let Ok(Some(block)) = db.lock().await.latest_block() {
        send_network_response
          .send(BchainResponse::Latest(block))
          .await?;
      }
      AppResult::Ok(())
    });
  }

  fn respond_block(&mut self, id: i64) {
    let db = self.db.clone();
    let (send_network_response, _) = self.network_responses.clone();
    task::spawn(async move {
      if let Ok(Some(block)) = db.lock().await.get_block(id) {
        send_network_response
          .send(BchainResponse::Block(block))
          .await?;
      }
      AppResult::Ok(())
    });
  }

  fn publish_user_message(&mut self, str: &str) {
    let (send_network_request, _) = self.network_requests.clone();
    let str = str.into();
    task::spawn(async move {
      send_network_request.send(BchainRequest::Msg(str)).await?;
      AppResult::Ok(())
    });
  }

  fn bootstrap(&mut self) -> AppResult<()> {
    let (num_peers, consensus) = self.num_peers_consensus();
    let cli = self.cli.clone();

    let wallet = self.wallet.clone();
    let db = self.db.clone();

    let (network_requests, _) = self.network_requests.clone();
    let (_, network_latest) = self.network_latest.clone();
    let (_, network_blocks) = self.network_blocks.clone();

    task::spawn(async move {
      info!("Bootstrapping network {}, peers {}", cli.net, num_peers);

      if cli.init {
        bootstrap_init(wallet, db).await?;
        return Ok(());
      }

      if num_peers < 1 {
        warn!("Not enough peers to bootstrap");
        return Ok(());
      }

      let npc = (num_peers, consensus);

      loop {
        let network_latest_block =
          request_latest_block(&npc, network_requests.clone(), network_latest.clone()).await?;
        let local_latest_block = db.lock().await.latest_block()?;

        if network_latest_block == local_latest_block {
          info!("Local and remote blocks syncronized");
          break;
        }

        if local_latest_block < network_latest_block {
          let block = request_specific_block(
            network_latest_block.unwrap().id,
            &npc,
            network_requests.clone(),
            network_blocks.clone(),
          )
          .await?;

          if let Some(block) = block {
            db.lock().await.commit_block(&block)?;
          }
        }
      }

      info!("Bootstrap complete");

      Ok(()) as AppResult<()>
    });

    Ok(())
  }

  pub(crate) fn print_balance(&self, address: &Option<Address>) {
    let wallet = self.wallet.clone();
    let db = self.db.clone();
    let address = address.clone();
    task::spawn(async move {
      let address = address.unwrap_or(wallet.read().await.public_address());
      let balance = local_balance(&address, db).await?;
      info!("Balance: ¢{} @ {}", balance, address.short());
      Ok(()) as AppResult<()>
    });
  }

  pub(crate) fn submit_tx(&self, address: &Address, amount: u64) {
    info!("tx {:?} {:?}", address, amount);
  }
}
