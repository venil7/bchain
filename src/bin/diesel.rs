#[macro_use]
extern crate diesel_migrations;

use bchain::cli::Cli;
use bchain::db::database::Db;
use bchain::db::raw_block::NewRawBlock;
use bchain::result::AppResult;
use diesel_migrations::embed_migrations;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> AppResult<()> {
  embed_migrations!();
  dotenv::dotenv()?;
  let cli = Cli::from_args();

  let mut db = Db::new(&cli.database)?;
  embedded_migrations::run(db.raw_connection()?)?;

  println!("connected!");

  let block = NewRawBlock {
    block: b"some long string goes here".to_vec(),
    created: chrono::Utc::now().naive_utc(),
  };

  db.insert_block(&block)?;
  // conn.execute("select * from blocks;")?;

  // insert_block(&conn)?;
  // print_users_holdings(&conn)?;
  Ok(())
}
