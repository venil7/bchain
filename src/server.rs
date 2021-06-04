use crate::domain::gossip::Gossip;
use crate::domain::gossip::Peer;
use log::{error, info};
use serde_cbor;
use std::net::SocketAddr;

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;
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
    _sender_address: &SocketAddr,
    _bytes: &Vec<u8>,
  ) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
  async fn announce(
    socket: Arc<Mutex<UdpSocket>>,
    peer: &Peer,
    peers: &[Peer],
  ) -> Result<(), Box<dyn Error>> {
    let buf = serde_cbor::to_vec(&Gossip::AllPeers(peers.to_vec()))?;
    let socket = socket.lock().await;
    socket.send_to(&buf, peer.to_string()).await?;
    Ok(())
  }

  pub async fn listen<A: ToSocketAddrs>(&self, address: A) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(address).await?;
    let socket = Arc::new(Mutex::new(socket));
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

      tokio::spawn(async move {
        if let Err(err) = ServerP2p::handle_req(&sender_address, &bytes).await {
          info!("Error handling {:?}", err);
        }
      });
    }
  }
}
