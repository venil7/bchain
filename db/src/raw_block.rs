use crate::schema::blocks;
use bchain_domain::block::Block;
use bchain_domain::error::AppError;
use chrono::NaiveDateTime;
use std::convert::TryFrom;

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

impl TryFrom<RawBlock> for Block {
  type Error = AppError;

  fn try_from(raw_block: RawBlock) -> Result<Self, Self::Error> {
    let str_json = String::from_utf8(raw_block.block).unwrap();
    let block: Block = serde_json::from_str(&str_json)?;
    assert_eq!(raw_block.id, block.id as i32);
    Ok(block)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_domain::{block::Block, result::AppResult, tx::Tx, wallet::Wallet};

  const RSAKEY_PEM: &str = "../rsakey.pem";

  #[async_std::test]
  async fn to_raw_and_back() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let genesis = Block::new();
    let mut block = Block::new_from_previous(&genesis);
    let tx = Tx::new(&wallet, wallet.public_key(), 1234)?;
    block.add(&tx);
    let raw = RawBlock::try_from(block.clone())?;
    let block1 = Block::try_from(raw)?;
    assert_eq!(block, block1);
    Ok(())
  }
}
