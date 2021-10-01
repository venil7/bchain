use super::hash_digest::HashDigest;
use crate::chain::hash_digest::AsBytes;
use crate::chain::hash_digest::Hashable;
use crate::chain::tx::Tx;
use crate::db::raw_block::RawBlock;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
  pub id: i64,
  pub timestamp: i64,
  pub txs: HashMap<String, Tx>,
  pub parent_hash: Option<HashDigest>,
  pub nonce: Vec<u8>,
}

impl AsBytes for Block {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.extend_from_slice(&self.id.as_bytes());
    res.extend_from_slice(&self.timestamp.as_bytes());
    let txs: Vec<Tx> = self.txs.iter().map(|(_, tx)| tx.clone()).collect();
    for tx in txs {
      res.extend_from_slice(&tx.as_bytes())
    }
    res.extend_from_slice(&self.parent_hash.as_bytes());
    res.extend_from_slice(&self.nonce.clone());
    res
  }
}

impl Hashable for Block {}

impl Block {
  pub fn new() -> Block {
    Block {
      id: 0,
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: None,
      nonce: vec![],
    }
  }

  pub fn new_from_previous(previous_block: &Block) -> Block {
    Block {
      id: previous_block.id + 1,
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: Some(previous_block.hash_digest()),
      nonce: vec![],
    }
  }

  pub fn add(&mut self, tx: &Tx) {
    let key = tx.hash_digest().to_string();
    self.txs.insert(key, tx.clone());
  }
}

impl Default for Block {
  fn default() -> Self {
    Self::new()
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
  use crate::{chain::wallet::Wallet, result::AppResult};

  #[async_std::test]
  async fn bloc_equality_test() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;
    let genesis = Block::new();
    let mut block = Block::new_from_previous(&genesis);
    let tx = Tx::new(&wallet, wallet.public_key(), 1234)?;
    block.add(&tx);
    let hash1 = block.hash_digest();
    let json = serde_json::to_string(&block)?;
    let block: Block = serde_json::from_str(&json)?;
    let hash2 = block.hash_digest();
    assert_eq!(hash1, hash2);
    Ok(())
  }

  #[async_std::test]
  async fn to_raw_and_back() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;
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
