use bchain_domain::cli::Cli;
use bchain_network::node::Node;
use bchain_util::result::AppResult;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> AppResult<()> {
  dotenv::dotenv()?;
  pretty_env_logger::init();

  let mut node = Node::new(&Cli::from_args()).await?;
  node.run().await?;

  Ok(())
}
