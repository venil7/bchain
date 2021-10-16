use async_std::channel::unbounded;
use async_std::task;
use bchain_db::database::create_db;
use bchain_domain::cli::Cli;
use bchain_domain::result::AppResult;
use log::info;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> AppResult<()> {
  dotenv::dotenv()?;
  pretty_env_logger::init();

  let cli = Cli::from_args();
  let _db = create_db(&cli.database)?;

  info!("connected!");

  let (s, r) = unbounded::<usize>();

  let s = s.clone();
  let r = r.clone();

  task::spawn(async move {
    info!("running this in thread");
    s.send(10).await.unwrap();
  })
  .await;

  let r = r.recv().await?;

  info!("--> {}", r);

  // let latest = db.latest_block()?;
  // if let Some(latest) = latest {
  //   info!("latest: {:?}", latest);
  //   let block = Block::new_from_previous(&latest);
  //   db.commit_block(block)?;
  // } else {
  //   warn!("no blocks, adding genesis");
  //   let genesis = Block::new();
  //   db.commit_block(genesis)?;
  // }

  Ok(())
}
