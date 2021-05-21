use bchain::peerfinder::NodeP2P;
use bchain::peerfinder::peer_finder::peerfinder_server::PeerfinderServer;

use bchain::cli::Cli;
use structopt::StructOpt;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::from_args();

  let addr = cli.listen.parse()?;
  let peer_finder = NodeP2P::default();

  Server::builder()
    .add_service(PeerfinderServer::new(peer_finder))
    .serve(addr)
    .await?;

  Ok(())
}
