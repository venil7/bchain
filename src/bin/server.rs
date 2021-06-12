use bchain::cli::Cli;
use bchain::server::ServerP2p;
use std::error::Error;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();
  let cli = Cli::from_args();
  let mut server = ServerP2p::new(&cli.bootstrap);
  server.listen(&cli.listen).await?;

  Ok(())
}
