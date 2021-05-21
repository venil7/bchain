use bchain::cli::Cli;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::HashSet;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::UdpSocket;
// use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Peer {
  uuid: uuid::Uuid,
  address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Gossip {
  Announce(Peer),
  Welcome { peers: Vec<Peer> },
}

async fn announce(
  socket: Arc<Mutex<UdpSocket>>,
  address: &SocketAddr,
  uuid: &uuid::Uuid,
  listen: &str,
) -> Result<(), Box<dyn Error>> {
  println!("Bootstrapping with {:?}", address);
  let announce = Gossip::Announce(Peer {
    uuid: uuid.clone(),
    address: listen.to_owned(),
  });
  let buf = serde_cbor::to_vec(&announce)?;
  let socket = socket.lock().await;
  let _ = socket.send_to(&buf, address).await?;
  println!("Sent to {:?}", address);

  Ok(())
}
async fn handle_req(
  // _socket: Arc<Mutex<UdpSocket>>,
  src_address: &SocketAddr,
  bytes: &[u8],
) -> Result<(), Box<dyn Error>> {
  println!("received from {:?}", src_address);
  let gossip = serde_cbor::from_slice::<Gossip>(bytes)?;
  match gossip {
    Gossip::Announce(peer) => println!("---> {:?}, {:?}", peer, src_address),
    Gossip::Welcome { peers } => println!("---> {:?}", peers),
  }
  Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::from_args();
  let uuid = uuid::Uuid::new_v4();
  let mut _db: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

  let socket = Arc::new(Mutex::new(UdpSocket::bind(&cli.listen).await?));
  println!("Listening {:?}", cli.listen);

  for bootstrap_address in cli.bootstrap {
    let addr = bootstrap_address.parse::<SocketAddr>()?;
    let listen = cli.listen.clone();
    announce(socket.clone(), &addr, &uuid, &listen).await?;
  }

  loop {
    let mut buf = [0; 128];

    println!("before block");
    let (numbytes, sender_address) = {
      let sock = socket.lock().await;
      let (numbytes, sender_address) = sock.recv_from(&mut buf).await?;
      (numbytes, sender_address)
    };
    let bbuf = (&buf[0..numbytes]).to_vec();
    // let sock = socket.clone();
    tokio::spawn(async move {
      println!("spawning new handler block");
      if let Err(err) = handle_req(/*sock,*/ &sender_address, &bbuf).await {
        println!("Error: {:?}", err);
      }
    });
    println!("after block");
  }
}
