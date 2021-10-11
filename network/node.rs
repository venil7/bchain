use super::protocol::{BchainResponse, Frame};
use crate::{
  chain::wallet::Wallet,
  cli::Cli,
  db::database::{create_db, Db},
  network::{protocol::BchainRequest, user_command::UserCommand},
  result::AppResult,
};
use async_std::{io, sync::Mutex, task};
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
use std::{sync::Arc, time::Duration};

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
  topic: Topic,
  #[allow(dead_code)]
  db: Arc<Mutex<Db>>,
  #[allow(dead_code)]
  wallet: Arc<Mutex<Wallet>>,
  swarm: Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>,
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
      swarm,
      db: Arc::new(Mutex::new(db)),
      wallet: Arc::new(Mutex::new(wallet)),
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

    loop {
      select! {
        _ = bootstrap => {
          self.bootstrap().await?;
        },
        event = self.swarm.next().fuse() => {
          self.handle_swarm_event(event.unwrap()).await?;
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
          Ok(_) => {
            info!("Dialed {:?}", peer);
          }
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
      UserCommand::Msg(msg) => {
        let msg = Frame::BchainRequest(BchainRequest::Msg(msg.into()));
        self.publish_to_swarm(&msg).await?;
      }
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
      BchainRequest::Msg(msg) => info!("{}", msg),
      _ => warn!("Unhandled bchain request"),
    }
  }

  fn handle_bchain_response(&mut self, response: BchainResponse) {
    match response {
      _ => warn!("Unhandled bchain response"),
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

  fn num_peers(&self) -> usize {
    self.swarm.network_info().num_peers()
  }

  fn display_peers(&self) {
    info!("Peers: {}", self.num_peers());
  }

  async fn bootstrap(&mut self) -> AppResult<()> {
    info!("Bootstrapping..");

    self.display_peers();
    Ok(())
  }
}
