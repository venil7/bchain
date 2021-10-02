use super::protocol::Frame;
use crate::{
  chain::wallet::Wallet,
  cli::Cli,
  db::database::{create_db, Db},
  network::{protocol::BchainRequest, user_command::UserCommand},
  result::AppResult,
};
use async_std::io;
use async_std::sync::RwLock;
use futures::{prelude::*, select};
use libp2p::{
  gossipsub::{
    self, error::GossipsubHandlerError, subscription_filter::AllowAllSubscriptionFilter, Gossipsub,
    GossipsubEvent, IdentTopic as Topic, IdentityTransform, MessageAuthenticity, ValidationMode,
  },
  identity::{self, Keypair},
  swarm::SwarmEvent,
  PeerId, Swarm,
};
use log::{error, info, warn};
use std::time::Duration;

async fn create_swarm(
  local_peer_key: &Keypair,
  topic: &Topic,
) -> AppResult<Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>> {
  let local_peer_id = PeerId::from(local_peer_key.public());
  let transport = libp2p::development_transport(local_peer_key.clone()).await?;

  let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
    .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
    .build()?;

  let mut gossipsub: gossipsub::Gossipsub = gossipsub::Gossipsub::new(
    MessageAuthenticity::Signed(local_peer_key.clone()),
    gossipsub_config,
  )?;

  gossipsub.subscribe(topic).unwrap();

  Ok(libp2p::Swarm::new(transport, gossipsub, local_peer_id))
}

pub struct Node {
  cli: Cli,
  #[allow(dead_code)]
  db: Db,
  #[allow(dead_code)]
  wallet: Wallet,
  topic: Topic,
  swarm: Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>,
  num_peers: RwLock<i32>,
}

impl Node {
  pub async fn new(cli: &Cli) -> AppResult<Node> {
    let topic = Topic::new(&cli.net);

    let wallet = Wallet::from_file(&cli.wallet).await?;
    let mut rsa_pkcs8 = wallet.to_pkcs8_der()?;
    let local_peer_key = identity::Keypair::rsa_from_pkcs8(&mut rsa_pkcs8)?;

    let db = create_db(cli)?;

    let swarm = create_swarm(&local_peer_key, &topic).await?;
    let cli = cli.clone();
    Ok(Node {
      cli,
      topic,
      db,
      wallet,
      swarm,
      num_peers: RwLock::new(0),
    })
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self.swarm.listen_on(self.cli.listen.parse()?)?;

    let peers = &self.cli.peers.clone()[..];
    self.dial_peers(&peers);

    let mut cmd_lines = io::BufReader::new(io::stdin()).lines();

    loop {
      select! {
        event = self.swarm.next().fuse() => {
          self.handle_swarm_event(event.unwrap()).await?;
        },
        cmd_line = cmd_lines.next().fuse() => {
          if let Some(Ok(ref line)) = cmd_line {
            let cmd: UserCommand = line.parse()?;
            self.handle_user_command(&cmd).await?;
          }
        },
        complete => break,
      }
    }

    Ok(())
  }

  fn dial_peers(&mut self, peers: &[String]) {
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
  }

  async fn handle_user_command(&mut self, cmd: &UserCommand) -> AppResult<()> {
    match cmd {
      UserCommand::Msg(msg) => {
        let msg = Frame::BchainRequest(BchainRequest::Msg(msg.into()));
        self.publish_to_swarm(&msg).await?;
      }
      UserCommand::Peers => {
        self.display_peers().await;
      }
      UserCommand::Dial(peers) => {
        self.dial_peers(peers);
      }
      UserCommand::Unrecognized => warn!("Unrecognized user input"),
      command => warn!("Currently unsupported: {:?}", command),
    }
    Ok(())
  }

  async fn handle_swarm_event(
    &mut self,
    event: SwarmEvent<GossipsubEvent, GossipsubHandlerError>,
  ) -> AppResult<()> {
    match event {
      SwarmEvent::Behaviour(GossipsubEvent::Message { message, .. }) => {
        let frame: Frame = serde_json::from_slice(&message.data)?;
        self.handle_bchain_event(frame);
      }
      SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {:?}", address),
      SwarmEvent::ConnectionEstablished { .. } => self.handle_peer(1).await?,
      SwarmEvent::ConnectionClosed { .. } => self.handle_peer(-1).await?,
      _ => (),
    }
    Ok(())
  }

  fn handle_bchain_event(&mut self, frame: Frame) {
    match frame {
      Frame::BchainRequest(BchainRequest::Msg(msg)) => info!("{}", msg),
      _ => info!("unhandled event"),
    }
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

  async fn handle_peer(&mut self, num: i32) -> AppResult<()> {
    let mut num_peers = self.num_peers.write().await;
    *num_peers += num;
    Ok(())
  }

  async fn display_peers(&self) {
    let num_peers = self.num_peers.read().await;
    info!("Peers: {}", num_peers);
  }
}
