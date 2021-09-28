use super::protocol::Frame;
use crate::{
  chain::wallet::Wallet,
  cli::Cli,
  db::database::{create_db, Db},
  network::{protocol::BchainRequest, user_command::UserCommand},
  result::AppResult,
};
use async_std::io;
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
  _db: Db,
  _wallet: Wallet,
  topic: Topic,
  swarm: Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>,
}

impl Node {
  pub async fn new(cli: &Cli) -> AppResult<Node> {
    let topic = Topic::new(&cli.net);

    let _wallet = Wallet::from_file(&cli.wallet).await?;
    let mut rsa_pkcs8 = _wallet.to_pkcs8_der()?;
    let local_peer_key = identity::Keypair::rsa_from_pkcs8(&mut rsa_pkcs8)?;

    let _db = create_db(cli)?;

    let swarm = create_swarm(&local_peer_key, &topic).await?;

    Ok(Node {
      cli: cli.clone(),
      topic,
      _db,
      _wallet,
      swarm,
    })
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self.swarm.listen_on(self.cli.listen.parse()?)?;

    self.dial_peers();

    let mut cmd_lines = io::BufReader::new(io::stdin()).lines();

    loop {
      select! {
        event = self.swarm.next().fuse() => {
          self.handle_swarm_event(event.unwrap()).await?;
        },
        cmd_line = cmd_lines.next().fuse() => {
          if let Some(Ok(ref line)) = cmd_line {
            let cmd: UserCommand = line.parse()?;
            self.handle_cmd_event(&cmd).await?;
          }
        },
        complete => break,
      }
    }

    Ok(())
  }

  fn dial_peers(&mut self) {
    for peer in self.cli.peers.iter() {
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

  async fn handle_cmd_event(&mut self, cmd: &UserCommand) -> AppResult<()> {
    match cmd {
      UserCommand::Msg(msg) => {
        let msg = Frame::BchainRequest(BchainRequest::Msg(msg.into()));
        self.publish_to_swarm(&msg).await?;
      }
      UserCommand::Tx { .. } => info!("currently unsupported"),
      _ => warn!("unrecognized user input"),
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
}
