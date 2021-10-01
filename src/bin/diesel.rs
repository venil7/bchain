use bchain::chain::block::Block;
use bchain::cli::Cli;
use bchain::db::database::create_db;
use bchain::result::AppResult;
use log::{info, warn};
use structopt::StructOpt;

#[async_std::main]
async fn main() -> AppResult<()> {
  dotenv::dotenv()?;
  pretty_env_logger::init();

  let cli = Cli::from_args();
  let mut db = create_db(&cli)?;

  info!("connected!");

  let latest = db.latest_block()?;
  if let Some(latest) = latest {
    info!("latest: {:?}", latest);
    let block = Block::new_from_previous(&latest);
    db.commit_block(block)?;
  } else {
    warn!("no blocks, adding genesis");
    let genesis = Block::new();
    db.commit_block(genesis)?;
  }

  Ok(())
}
