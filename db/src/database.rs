use crate::raw_block::RawBlock;
use crate::schema::blocks;
use bchain_domain::block::Block;
use bchain_util::error::AppError;
use bchain_util::hash_digest::Hashable;
use bchain_util::result::AppResult;
use diesel::prelude::*;
use diesel::result::Error::NotFound;
use diesel::SqliteConnection;
use diesel_migrations::embed_migrations;
use log::info;
use std::cmp::max;
use std::convert::TryInto;

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

  pub fn commit_block(&mut self, block: &Block) -> AppResult<()> {
    if let Some(latest) = self.latest_block()? {
      assert_eq!(latest.id + 1, block.id);
      assert_eq!(Some(latest.hash_digest()), block.parent_hash);
    }
    let raw_block: RawBlock = block.try_into()?;
    let query = diesel::insert_into(blocks::table).values(raw_block);
    query.execute(&self.connection)?;
    info!("Commited {}", block);
    Ok(())
  }

  pub fn commit_as_genesis(&mut self, block: &Block) -> AppResult<()> {
    let query = diesel::delete(blocks::table);
    query.execute(&self.connection)?;
    self.commit_block(block)
  }

  pub fn latest_block(&mut self) -> AppResult<Option<Block>> {
    let query = blocks::table
      .select(blocks::all_columns)
      .order(blocks::id.desc())
      .limit(1);

    match query.first::<RawBlock>(&self.connection) {
      Ok(res) => Ok(Some(res.try_into()?)),
      Err(NotFound) => Ok(None),
      Err(e) => Err(AppError::msg(format!("{:?}", e))),
    }
  }

  pub fn recent_blocks(&mut self, num_blocks: i64) -> AppResult<Vec<Block>> {
    let mut blocks = vec![];
    let latest = self.latest_block()?;
    if let Some(block) = latest {
      let latest_id = block.id;
      blocks.push(block);
      if latest_id > 0 {
        let rest = ((max(0, latest_id - num_blocks))..latest_id).rev();
        for id in rest {
          blocks.push(self.get_block(id)?.unwrap());
        }
      }
    }

    Ok(blocks)
  }

  pub fn get_block(&mut self, id: i64) -> AppResult<Option<Block>> {
    let query = blocks::table
      .select(blocks::all_columns)
      .filter(blocks::id.eq(id as i32));

    match query.first::<RawBlock>(&self.connection) {
      Ok(res) => Ok(Some(res.try_into()?)),
      Err(NotFound) => Ok(None),
      Err(e) => Err(AppError::msg(format!("{:?}", e))),
    }
  }
}

pub fn create_db(path: &str) -> AppResult<Db> {
  let db = Db::new(path)?;
  info!("Using block chain database {}", path);
  embedded_migrations::run(db.raw_connection()?)?;
  Ok(db)
}
