use crate::{chain::wallet::Wallet, cli::Cli, error::AppError, result::AppResult};
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
use log::{error, info};
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
  wallet: Wallet,
  topic: Topic,
  swarm: Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>,
}

impl Node {
  pub async fn new(cli: &Cli) -> AppResult<Node> {
    let topic = Topic::new(&cli.net);

    let wallet = Wallet::from_file(&cli.wallet).await?;
    let mut rsa_pkcs8 = wallet.to_pkcs8_der()?;
    let local_peer_key = identity::Keypair::rsa_from_pkcs8(&mut rsa_pkcs8)?;

    let swarm = create_swarm(&local_peer_key, &topic).await?;

    Ok(Node {
      cli: cli.clone(),
      topic,
      wallet,
      swarm,
    })
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self.swarm.listen_on(self.cli.listen.parse()?)?;

    self.dial_peers();

    let mut cmd_lines = io::BufReader::new(io::stdin()).lines();

    loop {
      select! {
        event = self.swarm.next().fuse() => self.handle_swarm_event(event.unwrap()),
        cmd_line = cmd_lines.next().fuse() => {
          if let Some(Ok(ref line)) = cmd_line {
            self.handle_cmd_event(line.as_str())?;
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
          Err(e) => error!("Dialing {:?} failed: {:?}", peer, e),
        },
        Err(err) => error!("Failed to parse address to dial: {:?}", err),
      }
    }
  }

  fn handle_cmd_event(&mut self, cmd: &str) -> AppResult<()> {
    let publish_result = self
      .swarm
      .behaviour_mut()
      .publish(self.topic.clone(), cmd.as_bytes());
    match publish_result {
      Ok(_) => Ok(()),
      Err(err) => Err(Box::new(AppError::from(err))),
    }
  }

  fn handle_swarm_event(&mut self, event: SwarmEvent<GossipsubEvent, GossipsubHandlerError>) {
    match event {
      SwarmEvent::Behaviour(GossipsubEvent::Message {
        message_id: _,
        message,
        propagation_source: peer_id,
      }) => info!("{:?}: {}", peer_id, String::from_utf8_lossy(&message.data)),
      SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {:?}", address),
      _ => (),
    }
  }
}
