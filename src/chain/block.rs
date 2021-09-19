use super::digest::HashDigest;
use crate::chain::digest::AsBytes;
use crate::chain::digest::Hashable;
use crate::chain::tx::Tx;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
  pub timestamp: i64,
  pub txs: HashMap<String, Tx>,
  pub parent_hash: Option<HashDigest>,
  pub nonce: Vec<u8>,
}

impl AsBytes for Block {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.append(&mut self.timestamp.as_bytes());
    let txs: Vec<Tx> = self.txs.iter().map(|(_, tx)| tx.clone()).collect();
    for tx in txs {
      res.append(&mut tx.as_bytes())
    }
    res.append(&mut self.parent_hash.as_bytes());
    res.append(&mut self.nonce.clone());
    res
  }
}

impl Hashable for Block {}

impl Block {
  pub fn new() -> Block {
    Block {
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: None,
      nonce: vec![],
    }
  }
  pub fn new_from_previous(previous_block: &Block) -> Block {
    Block {
      timestamp: chrono::Utc::now().timestamp(),
      txs: Default::default(),
      parent_hash: Some(previous_block.hash()),
      nonce: vec![],
    }
  }
  pub fn add_tx(&mut self, tx: &Tx) {
    let key = format!("{}", tx.receiver.to_address());
    self.txs.insert(key, tx.clone());
  }
}

impl Default for Block {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{chain::wallet::Wallet, result::AppResult};

  #[tokio::test]
  async fn bloc_equality_test() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;
    let tx = wallet.new_tx(&wallet.public_key()?, 0)?;
    let genesis = Block::new();
    let mut block = Block::new_from_previous(&genesis);
    block.add_tx(&tx);
    let hash1 = block.hash();
    let block: Block = serde_json::from_str(&serde_json::to_string(&block)?)?;
    let hash2 = block.hash();
    assert_eq!(hash1, hash2);
    Ok(())
  }
}
