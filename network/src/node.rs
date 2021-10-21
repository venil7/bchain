use crate::protocol::{BchainRequest, BchainResponse, Frame};
use crate::swarm::{create_swarm, BchainSwarm};
use crate::user_command::UserCommand;
use async_std::channel::{self, Receiver, Sender};
use async_std::future::timeout;
use async_std::prelude::FutureExt;
use async_std::sync::{Mutex, RwLock};
use async_std::{io, task};
use bchain_db::database::{create_db, Db};
use bchain_domain::block::Block;
use bchain_domain::hash_digest::Hashable;
use bchain_domain::tx::Tx;
use bchain_domain::{cli::Cli, result::AppResult, wallet::Wallet};
use bchain_util::consensus::peer_majority;
use bchain_util::group::group;
use futures::{prelude::*, select};
use libp2p::gossipsub::{error::GossipsubHandlerError, GossipsubEvent, IdentTopic as Topic};
use libp2p::{identity, swarm::SwarmEvent};
use log::{error, info, warn};
use std::{sync::Arc, time::Duration};

type Channel<T> = (Sender<T>, Receiver<T>);
type NumPeersConsensus = (usize, usize);

#[allow(dead_code)]
const TIMEOUT: Duration = Duration::from_secs(10);

pub struct Node {
  cli: Cli,
  topic: Topic,
  db: Arc<Mutex<Db>>,
  wallet: Arc<RwLock<Wallet>>,
  swarm: BchainSwarm,

  network_latest: Channel<i64>,
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
      UserCommand::Bootstrap => self.bootstrap()?,
      UserCommand::Dial(peers) => self.dial_peers(peers.clone())?,
      UserCommand::Msg(msg) => self.publish_user_message(msg),
      UserCommand::Unrecognized => warn!("Unrecognized user input"),
      command => warn!("Currently unsupported: {:?}", command),
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
    info!("bchain request {:?}", request);
    match request {
      BchainRequest::AskLatest => self.respond_latest_block_id(),
      BchainRequest::AskBlock(id) => self.respond_block(id),
      BchainRequest::Msg(msg) => info!("{}", msg),
      _ => warn!("Unhandled bchain request"),
    }
  }

  fn handle_bchain_response(&mut self, response: BchainResponse) {
    match response {
      BchainResponse::Latest(id) => {
        let (network_latest_sender, _) = self.network_latest.clone();
        task::spawn(async move {
          network_latest_sender.send(id).await?;
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

  fn respond_latest_block_id(&mut self) {
    let db = self.db.clone();
    let (send_network_response, _) = self.network_responses.clone();
    info!("respond_latest_block_id");
    task::spawn(async move {
      if let Ok(Some(block)) = db.lock().await.latest_block() {
        info!("responding with latest block id {}", block.id);
        send_network_response
          .send(BchainResponse::Latest(block.id))
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

    task::spawn(async move {
      info!("Bootstrapping network {}", cli.net);
      info!("Peers {}", num_peers);

      if cli.init {
        bootstrap_init(wallet, db).await?;
        return Ok(());
      }

      if num_peers < 1 {
        warn!("Not enough peers to bootstrap");
        return Ok(());
      }

      let npc = (num_peers, consensus);
      if let Some(id) = request_latest_block_id(&npc, network_requests, network_latest).await? {
        info!("latest id from network {}", id)
      }

      Ok(()) as AppResult<()>
    });

    Ok(())
  }
}

async fn bootstrap_init(wallet: Arc<RwLock<Wallet>>, db: Arc<Mutex<Db>>) -> AppResult<()> {
  let genesis = {
    let wallet = wallet.read().await;
    let tx = wallet.new_tx(wallet.public_key(), 1000)?;
    Block::new(Some([tx]))
  };
  let mut db = db.lock().await;
  db.commit_as_genesis(&genesis)?;
  info!("Writing genesis block {}", genesis.hash_digest());
  Ok(())
}

#[allow(dead_code)]
async fn request_latest_block_id(
  npc: &NumPeersConsensus,
  network_requests: Sender<BchainRequest>,
  network_latest: Receiver<i64>,
) -> AppResult<Option<i64>> {
  let (_, majority) = npc;
  // info!("Consensus {}", majority);
  network_requests.send(BchainRequest::AskLatest).await?;
  let mut network_latest_block_stream = group(network_latest, *majority, |&i| i);
  let network_latest_block_id = timeout(TIMEOUT, network_latest_block_stream.next());
  Ok(network_latest_block_id.await?)
}
