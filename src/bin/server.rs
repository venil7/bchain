use bchain::cli::Cli;
use bchain::server::ServerP2p;
use env_logger;
use std::error::Error;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();
  let cli = Cli::from_args();
  let server = ServerP2p::new(&cli.bootstrap);
  server.listen(&cli.listen).await?;

  Ok(())
}
