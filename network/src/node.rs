use crate::protocol::{BchainRequest, BchainResponse, Frame};
use crate::user_command::UserCommand;
use async_std::channel::{self, Receiver, Sender};
use async_std::sync::{Mutex, RwLock};
use async_std::{io, task};
use bchain_db::database::{create_db, Db};
use bchain_domain::block::Block;
use bchain_domain::{cli::Cli, error::AppError, result::AppResult, wallet::Wallet};
use futures::{prelude::*, select};
use libp2p::gossipsub::{
  self, error::GossipsubHandlerError, subscription_filter::AllowAllSubscriptionFilter, Gossipsub,
  GossipsubEvent, IdentTopic as Topic, IdentityTransform, MessageAuthenticity, ValidationMode,
};
use libp2p::{
  identity::{self, Keypair},
  swarm::SwarmEvent,
  PeerId, Swarm,
};
use log::{error, info, warn};
use std::{sync::Arc, time::Duration};

type Channel<T> = (Sender<T>, Receiver<T>);

async fn create_swarm(
  local_peer_key: &Keypair,
  topic: &Topic,
) -> AppResult<Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>> {
  let local_peer_id = PeerId::from(local_peer_key.public());
  let transport = libp2p::development_transport(local_peer_key.clone()).await?;

  let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
    .heartbeat_interval(Duration::from_secs(10))
    .validation_mode(ValidationMode::Strict)
    .build()
    .unwrap();

  let mut gossipsub: gossipsub::Gossipsub = {
    let gossipsub = gossipsub::Gossipsub::new(
      MessageAuthenticity::Signed(local_peer_key.clone()),
      gossipsub_config,
    );
    match gossipsub {
      Err(msg) => Err(AppError::msg(msg)),
      Ok(res) => Ok(res),
    }
  }?;

  gossipsub.subscribe(topic).unwrap();

  Ok(libp2p::Swarm::new(transport, gossipsub, local_peer_id))
}

pub struct Node {
  cli: Cli,
  topic: Topic,
  #[allow(dead_code)]
  db: Arc<Mutex<Db>>,
  #[allow(dead_code)]
  wallet: Arc<RwLock<Wallet>>,
  swarm: Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>,

  #[allow(dead_code)]
  network_latest: Channel<usize>,
  #[allow(dead_code)]
  network_blocks: Channel<Block>,

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
      network_responses: channel::unbounded(),
      network_requests: channel::unbounded(),
    })
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self.swarm.listen_on(self.cli.listen.parse()?)?;

    self.dial_peers(&self.cli.peers.clone()).await?;

    let mut bootstrap = {
      let duration = self.cli.delay;
      let duration = Duration::from_secs(duration as u64);
      Box::pin(task::sleep(duration).fuse())
    };

    let mut cmd_lines = io::BufReader::new(io::stdin()).lines();

    let (_, mut network_responses) = self.network_responses.clone();
    let (_, mut network_requests) = self.network_requests.clone();

    loop {
      select! {
        _ = bootstrap => {
          self.bootstrap().await?;
        },
        response = network_responses.next().fuse() => {
          if let Some(ref response) = response {
            self.publish_response(response).await?;
          }
        },
        request = network_requests.next().fuse() => {
          if let Some(ref request) = request {
            self.publish_request(request).await?;
          }
        },
        event = self.swarm.next().fuse() => {
          if let Some(ref swarm_event) = event {
            self.handle_swarm_event(swarm_event).await?;
          }
        },
        cmd_line = cmd_lines.next().fuse() => {
          if let Some(Ok(ref line)) = cmd_line {
            self.handle_user_command(&line.parse()?).await?;
          }
        },
        complete => break,
      }
    }

    Ok(())
  }

  async fn dial_peers(&mut self, peers: &[String]) -> AppResult<()> {
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

  async fn handle_user_command(&mut self, cmd: &UserCommand) -> AppResult<()> {
    match cmd {
      UserCommand::Peers => self.display_peers(),
      UserCommand::Bootstrap => self.bootstrap().await?,
      UserCommand::Dial(peers) => self.dial_peers(peers).await?,
      UserCommand::Unrecognized => warn!("Unrecognized user input"),
      UserCommand::Msg(msg) => self.publish_user_message(msg),
      command => warn!("Currently unsupported: {:?}", command),
    }
    Ok(())
  }

  async fn handle_swarm_event(
    &mut self,
    event: &SwarmEvent<GossipsubEvent, GossipsubHandlerError>,
  ) -> AppResult<()> {
    match event {
      SwarmEvent::Behaviour(GossipsubEvent::Message { message, .. }) => {
        let frame: Frame = serde_json::from_slice(&message.data)?;
        self.handle_bchain_event(frame);
      }
      SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {:?}", address),
      _ => (),
    }
    Ok(())
  }

  fn handle_bchain_event(&mut self, frame: Frame) {
    match frame {
      Frame::BchainRequest(request) => self.handle_bchain_request(request),
      Frame::BchainResponse(response) => self.handle_bchain_response(response),
      _ => warn!("Unrecognized bchain event"),
    }
  }

  fn handle_bchain_request(&mut self, request: BchainRequest) {
    match request {
      BchainRequest::AskLatest(_) => self.respond_latest_block_id(),
      BchainRequest::AskBlock(id) => self.respond_block(id),
      BchainRequest::Msg(msg) => info!("{}", msg),
      _ => warn!("Unhandled bchain request"),
    }
  }

  fn handle_bchain_response(&mut self, response: BchainResponse) {
    match response {
      _ => warn!("Unhandled bchain response"),
    }
  }

  async fn publish_response(&mut self, res: &BchainResponse) -> AppResult<()> {
    self
      .publish_to_swarm(&Frame::BchainResponse(res.clone()))
      .await?;
    Ok(())
  }

  async fn publish_request(&mut self, req: &BchainRequest) -> AppResult<()> {
    self
      .publish_to_swarm(&Frame::BchainRequest(req.clone()))
      .await?;
    Ok(())
  }

  async fn publish_to_swarm(&mut self, frame: &Frame) -> AppResult<()> {
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

  fn num_peers(&self) -> usize {
    self.swarm.network_info().num_peers()
  }

  fn display_peers(&self) {
    info!("Peers: {}", self.num_peers());
  }

  fn respond_latest_block_id(&mut self) {
    let db = self.db.clone();
    let (send_network_response, _) = self.network_responses.clone();
    task::spawn(async move {
      if let Ok(Some(block)) = db.lock().await.latest_block() {
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

  async fn bootstrap(&mut self) -> AppResult<()> {
    info!("Bootstrapping..");

    self.display_peers();
    Ok(())
  }
}
