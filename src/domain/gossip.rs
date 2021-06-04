use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Peer(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gossip {
  AllPeers(Vec<Peer>),
  DiffPeers(Vec<Peer>),
}

impl TryFrom<Peer> for SocketAddr {
  type Error = std::net::AddrParseError;

  fn try_from(peer: Peer) -> Result<Self, Self::Error> {
    peer.0.parse()
  }
}

impl ToString for Peer {
  fn to_string(&self) -> String {
    self.0.to_string()
  }
}
