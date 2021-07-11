use bchain::cli::Cli;
use bchain::server::FullNode;
use std::error::Error;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv::dotenv()?;
  env_logger::init();
  let cli = Cli::from_args();
  let addr = cli.listen.parse()?;
  FullNode::run(addr).await?;

  Ok(())
}
