use bchain::cli::Cli;
use bchain::error::AppError;
use bchain::full_node::FullNode;
use std::error::Error;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv::dotenv()?;
  env_logger::init();
  let cli = Cli::from_args();
  let addr = cli.listen.parse()?;

  let server_handle = tokio::spawn(async move {
    FullNode::run(addr).await?;
    Ok::<(), AppError>(())
  });

  server_handle.await??;

  Ok(())
}
