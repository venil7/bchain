use async_std::io;
use bchain::chain::wallet::Wallet;
use bchain::cli::Cli;
use bchain::error::AppError;
use bchain::network::swarm::create_swarm;
use bchain::result::AppResult;
use futures::prelude::*;
use futures::select;
use futures::stream::StreamExt;
use libp2p::gossipsub::error::GossipsubHandlerError;
use libp2p::gossipsub::subscription_filter::AllowAllSubscriptionFilter;
use libp2p::gossipsub::IdentityTransform;
use libp2p::gossipsub::{Gossipsub, GossipsubEvent, IdentTopic as Topic};
use libp2p::{identity, swarm::SwarmEvent, PeerId};
use log::{error, info};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> AppResult<()> {
  dotenv::dotenv()?;
  pretty_env_logger::init();

  let cli = Cli::from_args();
  let wallet = Wallet::from_file(&cli.wallet).await?;

  let mut rsa_pkcs8 = wallet.to_pkcs8_der()?;
  let local_peer_key = identity::Keypair::rsa_from_pkcs8(&mut rsa_pkcs8)?;
  let local_peer_id = PeerId::from(local_peer_key.public());
  let topic = Topic::new(cli.net);

  info!("address: {}", wallet.public_address()?);
  info!("local peer id {:?}", local_peer_id);
  info!("peers {:?}", cli.peers);

  //---------------
  let mut swarm = create_swarm(&local_peer_key, &topic).await?;

  // Listen on all interfaces and whatever port the OS assigns
  // swarm.listen_on(cli.listen.parse().unwrap()).unwrap();
  swarm.listen_on(cli.listen.parse()?)?;

  // Reach out to another node if specified
  for peer in cli.peers.iter() {
    let to_dial = peer.clone().parse();
    match to_dial {
      Ok(to_dial) => match swarm.dial_addr(to_dial) {
        Ok(_) => info!("Dialed {:?}", peer),
        Err(e) => error!("Dialing {:?} failed: {:?}", peer, e),
      },
      Err(err) => error!("Failed to parse address to dial: {:?}", err),
    }
  }

  let mut stdin_lines = io::BufReader::new(io::stdin()).lines();

  loop {
    select! {
      event = swarm.next().fuse() => handle_swarm_event(event.unwrap()),
      cmd_line = stdin_lines.next().fuse() => {
        if let Some(Ok(ref line)) = cmd_line {
          handle_cmd_event(line.as_str(), &topic, swarm.behaviour_mut())?;
        }
      },
      complete => break,
    }
  }
  //---------------

  Ok(())
}

fn handle_cmd_event(
  cmd: &str,
  topic: &Topic,
  behavior: &mut Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>,
) -> AppResult<()> {
  let publish_result = behavior.publish(topic.clone(), cmd.as_bytes());
  match publish_result {
    Ok(_) => Ok(()),
    Err(err) => Err(Box::new(AppError::from(err))),
  }
}

fn handle_swarm_event(event: SwarmEvent<GossipsubEvent, GossipsubHandlerError>) {
  match event {
    SwarmEvent::Behaviour(GossipsubEvent::Message {
      message_id: _,
      message,
      propagation_source: peer_id,
    }) => info!("{:?}: {}", peer_id, String::from_utf8_lossy(&message.data),),
    SwarmEvent::NewListenAddr { address, .. } => {
      info!("Listening on {:?}", address);
    }
    _ => {}
  }
}
