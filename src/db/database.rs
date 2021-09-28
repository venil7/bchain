use crate::cli::Cli;
use crate::db::raw_block::NewRawBlock;
use crate::db::schema::blocks;
use crate::result::AppResult;
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_migrations::embed_migrations;

embed_migrations!();

pub struct Db {
  connection: SqliteConnection,
}

impl Db {
  pub fn raw_connection(&self) -> AppResult<&SqliteConnection> {
    Ok(&self.connection)
  }

  pub fn new(path: &str) -> AppResult<Self> {
    let connection = SqliteConnection::establish(path)?;
    Ok(Db { connection })
  }

  pub fn insert_block(&mut self, block: &NewRawBlock) -> AppResult<()> {
    diesel::insert_into(blocks::table)
      .values(block)
      .execute(&self.connection)?;
    Ok(())
  }
}

pub fn create_db(cli: &Cli) -> AppResult<Db> {
  let db = Db::new(&cli.database)?;
  embedded_migrations::run(db.raw_connection()?)?;
  Ok(db)
}
