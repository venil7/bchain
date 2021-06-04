use crate::domain::gossip::Gossip;
use crate::domain::gossip::Peer;
use log::{error, info};
use serde_cbor;
use std::net::SocketAddr;

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
// use tokio::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ServerP2p {
  peers: Arc<Mutex<HashSet<Peer>>>,
}

impl ServerP2p {
  pub fn new(bootstrap_peers: &[String]) -> ServerP2p {
    let peers = bootstrap_peers.iter().cloned().map(|s| Peer(s)).collect();
    ServerP2p {
      peers: Arc::new(Mutex::new(peers)),
    }
  }
  async fn handle_req(
    _socket: Arc<Mutex<UdpSocket>>,
    sender_address: &SocketAddr,
    bytes: &Vec<u8>,
  ) -> Result<(), Box<dyn Error>> {
    let gossip = serde_cbor::from_slice::<Gossip>(bytes)?;
    match gossip {
      Gossip::AllPeers(all_peers) => {
        ServerP2p::handle_all_peers(_socket, sender_address, &all_peers).await?
      }
      Gossip::DiffPeers(diff_peers) => {
        ServerP2p::handle_diff_peers(_socket, sender_address, &diff_peers).await?
      }
    }
    Ok(())
  }
  async fn handle_all_peers(
    _socket: Arc<Mutex<UdpSocket>>,
    sender_address: &SocketAddr,
    _all_peers: &Vec<Peer>,
  ) -> Result<(), Box<dyn Error>> {
    info!("all peers received from {}", sender_address);
    Ok(())
  }
  async fn handle_diff_peers(
    _socket: Arc<Mutex<UdpSocket>>,
    sender_address: &SocketAddr,
    _diff_peers: &Vec<Peer>,
  ) -> Result<(), Box<dyn Error>> {
    info!("diff peers received from {}", sender_address);
    Ok(())
  }
  async fn announce(
    socket: Arc<Mutex<UdpSocket>>,
    peer: &Peer,
    peers: &[Peer],
  ) -> Result<(), Box<dyn Error>> {
    info!("sending all known peers to {:?}", peer);

    let buf = serde_cbor::to_vec(&Gossip::AllPeers(peers.to_vec()))?;
    let socket = socket.lock().await;
    socket.send_to(&buf, peer.to_string()).await?;
    Ok(())
  }

  pub async fn listen(&self, address: &str) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(address).await?;
    let local_addr = socket.local_addr();
    let socket = Arc::new(Mutex::new(socket));
    info!("listening on {:?}", local_addr);
    let peers = { self.peers.lock().await.clone() };
    let peers: Vec<Peer> = peers.into_iter().collect();
    for peer in peers.iter() {
      if let Err(e) = ServerP2p::announce(socket.clone(), &peer, &peers).await {
        error!("Error announcing {:?}", e)
      }
    }
    loop {
      let (bytes, sender_address) = {
        let mut buf = [0; 128];
        let sock = socket.lock().await;
        let (numbytes, sender_address) = sock.recv_from(&mut buf).await?;
        let bytes = (&buf[0..numbytes]).to_vec();
        (bytes, sender_address)
      };

      let socket_clone = socket.clone();
      tokio::spawn(async move {
        if let Err(err) = ServerP2p::handle_req(socket_clone, &sender_address, &bytes).await {
          info!("Error handling {:?}", err);
        }
      });
    }
  }
}
