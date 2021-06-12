use crate::domain::gossip::Gossip;
use crate::domain::gossip::Peer;
use log::{info, trace};
use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;

use std::collections::HashSet;
use std::error::Error;
use tokio::net::UdpSocket;

// #[derive(Debug, Clone)]
// pub struct AppErr;
// impl std::error::Error for AppErr {}
// impl std::fmt::Display for AppErr {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//     write!(f, "error")
//   }
// }

// impl From<Box<dyn Error>> for AppErr {
//   fn from(_: Box<dyn Error>) -> Self {
//     AppErr
//   }
// }

#[derive(Debug)]
pub enum Message {
  SendAllPeers(Peer, Vec<Peer>),
  SendDiff(Peer, Vec<Peer>),
}

#[derive(Debug)]
pub struct ServerP2p {
  peers: Rc<RefCell<HashSet<Peer>>>,
  socket: Rc<RefCell<Option<UdpSocket>>>,
}

impl ServerP2p {
  pub fn new(bootstrap_peers: &[String]) -> ServerP2p {
    let peers = bootstrap_peers.iter().cloned().map(Peer).collect();
    let peers = Rc::new(RefCell::new(peers));
    ServerP2p {
      peers,
      socket: Rc::new(RefCell::new(None)),
    }
  }

  pub async fn listen(&mut self, address: &str) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(address).await?;
    let local_addr = socket.local_addr();
    info!("listening on {:?}", local_addr);

    self.socket.replace(Some(socket));

    self.announce().await?;
    loop {
      if let Some(socket) = self.socket.borrow().as_ref() {
        let mut buf = [0; 1024];
        let (numbytes, sender_address) = socket.recv_from(&mut buf).await?;
        let bytes = (&buf[0..numbytes]).to_vec();
        self.handle_req(&sender_address, &bytes).await?;
      }
    }
  }

  async fn handle_req(
    &self,
    sender_address: &SocketAddr,
    bytes: &[u8],
  ) -> Result<(), Box<dyn Error>> {
    trace!("handle_req");
    let gossip = serde_cbor::from_slice::<Gossip>(bytes)?;
    match gossip {
      Gossip::AllPeers(ref all_peers) => self.handle_all_peers(all_peers, sender_address).await?,
      Gossip::DiffPeers(ref diff_peers) => {
        self.handle_diff_peers(diff_peers, sender_address).await?
      }
    }
    Ok(())
  }
  async fn handle_all_peers(
    &self,
    all_peers: &[Peer],
    sender_address: &SocketAddr,
  ) -> Result<(), Box<dyn Error>> {
    let sender = Peer::from(*sender_address);
    trace!("handle_all_peers");
    info!("all peers received from {}", sender_address);
    let all_peers = all_peers
      .into_iter()
      .chain(vec![&sender])
      .cloned()
      .collect::<Vec<Peer>>();
    let diff = self.peer_diff(&all_peers).await?;
    if !diff.is_empty() {
      self.send_diff(&Peer::from(*sender_address), &diff).await?;
      self.merge_peers(&all_peers).await?;
      self.announce().await?;
    }
    Ok(())
  }
  async fn handle_diff_peers(
    &self,
    diff_peers: &[Peer],
    sender_address: &SocketAddr,
  ) -> Result<(), Box<dyn Error>> {
    let sender = Peer::from(*sender_address);
    let diff_peers = diff_peers
      .into_iter()
      .chain(vec![&sender])
      .cloned()
      .collect::<Vec<Peer>>();
    trace!("handle_diff_peers");
    info!("diff peers received from {}", sender_address);
    self.merge_peers(&diff_peers).await?;
    Ok(())
  }

  async fn send_all(&self, peer: &Peer) -> Result<(), Box<dyn Error>> {
    trace!("send_all");
    if let Some(socket) = self.socket.borrow().as_ref() {
      let peers: Vec<Peer> = self.peers.borrow().iter().cloned().collect();
      let buf = serde_cbor::to_vec(&Gossip::AllPeers(peers))?;
      socket.send_to(&buf, peer.0.clone()).await?;
      info!("sending all known peers to {:?}", peer);
    }
    Ok(())
  }

  async fn send_diff(&self, peer: &Peer, diff_peers: &[Peer]) -> Result<(), Box<dyn Error>> {
    trace!("send_diff");
    if let Some(socket) = self.socket.borrow().as_ref() {
      let peers: Vec<Peer> = diff_peers.to_vec();
      let buf = serde_cbor::to_vec(&Gossip::DiffPeers(peers))?;
      socket.send_to(&buf, peer.0.clone()).await?;
      info!(
        "sending {} items diff peers to {:?}",
        diff_peers.len(),
        peer
      );
    }
    Ok(())
  }

  async fn peer_diff(&self, their_peers: &[Peer]) -> Result<Vec<Peer>, Box<dyn Error>> {
    trace!("peer_diff");
    let their_peers = their_peers.iter().cloned().collect::<HashSet<Peer>>();
    let diff = their_peers
      .difference(&self.peers.borrow())
      .cloned()
      .collect::<Vec<Peer>>();
    trace!("diff is {} long", diff.len());
    trace!("known peers {:?}", self.peers.borrow().iter());
    Ok(diff)
  }

  async fn merge_peers(&self, their_peers: &[Peer]) -> Result<(), Box<dyn Error>> {
    trace!("merge_peers");
    let their_peers = their_peers.iter().cloned().collect();
    let new_peers = { self.peers.borrow().union(&their_peers).cloned().collect() };
    self.peers.replace(new_peers);
    Ok(())
  }

  async fn announce(&self) -> Result<(), Box<dyn Error>> {
    trace!("announce");
    for peer in self.peers.borrow().iter() {
      self.send_all(&peer).await?;
    }
    Ok(())
  }
}
