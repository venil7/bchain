use bchain_domain::{error::AppError, result::AppResult};
use libp2p::gossipsub::{
  self, subscription_filter::AllowAllSubscriptionFilter, Gossipsub, IdentTopic as Topic,
  IdentityTransform, MessageAuthenticity, ValidationMode,
};
use libp2p::{identity::Keypair, PeerId, Swarm};
use std::time::Duration;

pub type BchainSwarm = Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>;

pub async fn create_swarm(local_peer_key: &Keypair, topic: &Topic) -> AppResult<BchainSwarm> {
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
