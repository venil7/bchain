use crate::schema::blocks;
use bchain_domain::block::Block;
use bchain_util::error::AppError;
use chrono::NaiveDateTime;
use std::convert::TryFrom;

#[derive(Queryable, Debug, Insertable, Clone, PartialEq)]
#[table_name = "blocks"]
pub struct RawBlock {
  pub id: i32,
  pub block: Vec<u8>,
  pub created: NaiveDateTime,
}

impl TryFrom<&Block> for RawBlock {
  type Error = AppError;

  fn try_from(block: &Block) -> Result<Self, Self::Error> {
    let str_json = serde_json::to_string(block)?;
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
  use bchain_domain::{block::Block, tx::Tx, wallet::Wallet};
  use bchain_util::result::AppResult;

  const RSAKEY_PEM: &str = "../pem/rsakey.pem";

  #[async_std::test]
  async fn to_raw_and_back() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let genesis = Block::default();
    let tx = Tx::new(&wallet, &wallet.address(), 1234)?;
    let block = Block::from_previous(&genesis, Some([tx]));
    let raw = RawBlock::try_from(&block)?;
    let block1 = Block::try_from(raw)?;
    assert_eq!(block, block1);
    Ok(())
  }
}
