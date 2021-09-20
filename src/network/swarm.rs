use std::time::Duration;

use libp2p::{
  gossipsub::{
    self, subscription_filter::AllowAllSubscriptionFilter, Gossipsub, IdentTopic as Topic,
    IdentityTransform, MessageAuthenticity, ValidationMode,
  },
  identity::Keypair,
  PeerId, Swarm,
};

use crate::result::AppResult;

pub async fn create_swarm(
  local_peer_key: &Keypair,
  topic: &Topic,
) -> AppResult<Swarm<Gossipsub<IdentityTransform, AllowAllSubscriptionFilter>>> {
  let local_peer_id = PeerId::from(local_peer_key.public());
  let transport = libp2p::development_transport(local_peer_key.clone()).await?;

  let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
    .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
    .build()
    .expect("Valid config");
  // build a gossipsub network behaviour
  let mut gossipsub: gossipsub::Gossipsub = gossipsub::Gossipsub::new(
    MessageAuthenticity::Signed(local_peer_key.clone()),
    gossipsub_config,
  )
  .expect("Correct configuration");

  gossipsub.subscribe(&topic).unwrap();

  Ok(libp2p::Swarm::new(transport, gossipsub, local_peer_id))
}
