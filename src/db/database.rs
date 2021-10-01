use std::convert::TryInto;

use crate::chain::block::Block;
use crate::chain::hash_digest::Hashable;
use crate::cli::Cli;
use crate::db::raw_block::RawBlock;
use crate::db::schema::blocks;
use crate::error::AppError;
use crate::result::AppResult;
use diesel::prelude::*;
use diesel::result::Error::NotFound;
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

  pub fn commit_block(&mut self, block: Block) -> AppResult<()> {
    if let Some(latest) = self.latest_block()? {
      assert_eq!(latest.id + 1, block.id);
      assert_eq!(Some(latest.hash_digest()), block.parent_hash);
    }
    let raw_block: RawBlock = block.try_into()?;
    let query = diesel::insert_into(blocks::table).values(raw_block);
    query.execute(&self.connection)?;
    Ok(())
  }

  pub fn latest_block(&mut self) -> AppResult<Option<Block>> {
    let query = blocks::table
      .select(blocks::all_columns)
      .order(blocks::id.desc())
      .limit(1);

    match query.first::<RawBlock>(&self.connection) {
      Ok(res) => Ok(Some(res.try_into()?)),
      Err(NotFound) => Ok(None),
      Err(e) => Err(Box::new(AppError::new(&format!("{:?}", e)))),
    }
  }

  pub fn get_block(&mut self, id: i64) -> AppResult<Option<Block>> {
    let query = blocks::table
      .select(blocks::all_columns)
      .filter(blocks::id.eq(id as i32));

    match query.first::<RawBlock>(&self.connection) {
      Ok(res) => Ok(Some(res.try_into()?)),
      Err(NotFound) => Ok(None),
      Err(e) => Err(Box::new(AppError::new(&format!("{:?}", e)))),
    }
  }
}

pub fn create_db(cli: &Cli) -> AppResult<Db> {
  let db = Db::new(&cli.database)?;
  embedded_migrations::run(db.raw_connection()?)?;
  Ok(db)
}
