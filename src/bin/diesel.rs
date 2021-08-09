#[macro_use]
extern crate diesel_migrations;

use bchain::database::block::{Block, NewBlock};
use bchain::database::generated::blocks;
use bchain::error::AppResult;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::*;
use dotenv::dotenv;
use std::env;
use std::io::{Error as IoError, ErrorKind};

fn establish_connection() -> Result<SqliteConnection, IoError> {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  SqliteConnection::establish(&database_url)
    .or_else(|e| Err(IoError::new(ErrorKind::InvalidData, format!("{:?}", e))))
}

fn insert_block(conn: &SqliteConnection) -> AppResult<()> {
  let block = NewBlock {
    transactions: b"abcdef".to_vec(),
    created: chrono::Utc::now().naive_utc(),
  };
  diesel::insert_into(blocks::table)
    .values(&block)
    .execute(conn)?;

  Ok(())
}

fn print_users_holdings(conn: &SqliteConnection) -> AppResult<()> {
  let blocks = blocks::table.load::<Block>(conn)?;

  for b in blocks {
    println!("{:?}", b);
  }
  Ok(())
}

fn main() -> AppResult<()> {
  embed_migrations!();

  let conn = establish_connection()?;
  println!("connected!");
  // conn.execute("select * from blocks;")?;

  insert_block(&conn)?;
  print_users_holdings(&conn)?;
  Ok(())
}
