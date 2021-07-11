use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Peer(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gossip {
  AllPeers(Vec<Peer>),
  DiffPeers(Vec<Peer>),
}

impl Deref for Peer {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &&self.0
  }
}

impl TryFrom<Peer> for SocketAddr {
  type Error = std::net::AddrParseError;

  fn try_from(peer: Peer) -> Result<Self, Self::Error> {
    peer.0.parse()
  }
}
impl From<SocketAddr> for Peer {
  fn from(socket_address: SocketAddr) -> Peer {
    Peer(socket_address.to_string())
  }
}

impl FromStr for Peer {
  type Err = AppError;
  fn from_str(socket_address: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
    Ok(Peer(socket_address.to_string()))
  }
}

impl ToString for Peer {
  fn to_string(&self) -> String {
    self.0.to_string()
  }
}
