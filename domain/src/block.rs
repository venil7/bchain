use crate::address::Address;
use crate::tx::Tx;
use bchain_util::hash_digest::{AsBytes, HashDigest, Hashable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

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
  pub fn new<TXs>(txs: Option<TXs>) -> Block
  where
    TXs: IntoIterator<Item = Tx>,
  {
    let mut block = Block {
      id: 0,
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: None,
      nonce: vec![],
    };
    if let Some(txs) = txs {
      for tx in txs {
        block.add(&tx);
      }
    }
    block
  }

  pub fn from_previous(previous_block: &Block) -> Block {
    Block {
      id: previous_block.id + 1,
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: Some(previous_block.hash_digest()),
      nonce: vec![],
    }
  }

  pub fn new_next(&self) -> Self {
    Self::from_previous(self)
  }

  pub fn add(&mut self, tx: &Tx) {
    let key = tx.hash_digest().to_string();
    self.txs.insert(key, tx.clone());
  }

  pub fn diff_for_address(&self, address: &Address) -> i64 {
    self
      .txs
      .values()
      .fold(0, |acc, tx| acc + tx.diff_for_address(address))
  }
}

impl Default for Block {
  fn default() -> Self {
    Self::new::<Vec<Tx>>(None)
  }
}

impl PartialOrd for Block {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    (self.id, self.timestamp).partial_cmp(&(other.id, other.timestamp))
  }
}

impl Display for Block {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Block #{} hash: {}", self.id, self.hash_digest())
  }
}

#[cfg(test)]
mod tests {
  use bchain_util::result::AppResult;

  use super::*;
  use crate::wallet::Wallet;

  const RSAKEY_PEM: &str = "../rsakey.pem";

  #[async_std::test]
  async fn bloc_equality_test() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let genesis = Block::default();
    let mut block = Block::from_previous(&genesis);
    let tx = Tx::new(&wallet, wallet.public_key(), 1234)?;
    block.add(&tx);
    let hash1 = block.hash_digest();
    let json = serde_json::to_string(&block)?;
    let block: Block = serde_json::from_str(&json)?;
    let hash2 = block.hash_digest();
    assert_eq!(hash1, hash2);
    Ok(())
  }
}