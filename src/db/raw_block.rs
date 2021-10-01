use std::convert::TryFrom;

use crate::{chain::block::Block, db::schema::blocks, error::AppError};
use chrono::NaiveDateTime;

#[derive(Queryable, Debug, Insertable, Clone, PartialEq)]
#[table_name = "blocks"]
pub struct RawBlock {
  pub id: i32,
  pub block: Vec<u8>,
  pub created: NaiveDateTime,
}

impl TryFrom<Block> for RawBlock {
  type Error = AppError;

  fn try_from(block: Block) -> Result<Self, Self::Error> {
    let str_json = serde_json::to_string(&block)?;
    let raw_block = RawBlock {
      id: block.id as i32,
      block: str_json.as_bytes().to_vec(),
      created: NaiveDateTime::from_timestamp(block.timestamp, 0),
    };
    Ok(raw_block)
  }
}
