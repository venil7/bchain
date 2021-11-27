use crate::address::Address;
use crate::tx::Tx;
use async_std::task;
use async_trait::async_trait;
use bchain_util::hash_digest::{AsBytes, HashDigest, Hashable};
use bchain_util::mine::Mine;
use chrono::Utc;
use itertools::iterate;
use num::{BigUint, One, Zero};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::repeat;

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

#[async_trait]
impl Mine for Block {
  async fn mine(&mut self, difficulty: usize) {
    let block = self.clone();
    self.nonce = task::spawn(async move {
      let zero: BigUint = Zero::zero();
      let one: BigUint = One::one();
      let copies = repeat(block);
      let iterator = iterate(zero, move |n| n + &one)
        .map(|num| num.to_bytes_be())
        .zip(copies);
      let bridge = iterator.par_bridge();
      let solution =
        bridge.find_any(|(nonce, blk)| blk.nonce_matches_difficulty(nonce, difficulty));
      solution.unwrap().0
    })
    .await;
  }
}

impl Block {
  pub fn new<TXs>(txs: Option<TXs>) -> Block
  where
    TXs: IntoIterator<Item = Tx>,
  {
    let nonce: BigUint = Zero::zero();
    let timestamp = Utc::now().timestamp();
    let mut block = Block {
      id: 0,
      timestamp,
      txs: Default::default(),
      parent_hash: None,
      nonce: nonce.to_bytes_be(),
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
      parent_hash: Some(previous_block.hash_digest()),
      ..Default::default()
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

  pub fn nonce_matches_difficulty(&self, nonce: &[u8], difficulty: usize) -> bool {
    let mut block = self.clone();
    block.nonce = nonce.to_owned();
    block.hash_difficulty() >= difficulty
  }
}

impl Default for Block {
  fn default() -> Self {
    Self::new::<Vec<Tx>>(None)
  }
}

impl PartialOrd for Block {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match self.id.cmp(&other.id) {
      Ordering::Equal => Some(self.hash_digest().cmp(&other.hash_digest())),
      other => Some(other),
    }
  }
}

impl Display for Block {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Block #{} hash: {}", self.id, self.hash_digest())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::wallet::Wallet;
  use bchain_util::hash_digest::Hashable;
  use bchain_util::result::AppResult;

  const RSAKEY_PEM: &str = "../pem/rsakey.pem";

  #[async_std::test]
  async fn bloc_equality_test() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let genesis = Block::default();
    let mut block = Block::from_previous(&genesis);
    let tx = Tx::new(&wallet, &wallet.address(), 1234)?;
    block.add(&tx);
    let hash1 = block.hash_digest();
    let json = serde_json::to_string(&block)?;
    let block: Block = serde_json::from_str(&json)?;
    let hash2 = block.hash_digest();
    assert_eq!(hash1, hash2);
    Ok(())
  }

  #[async_std::test]
  async fn difficulty_test_1() -> AppResult<()> {
    let mut block = Block::default();
    block.mine(1).await;
    assert!(block.hash_difficulty() >= 1);
    Ok(())
  }

  #[async_std::test]
  async fn difficulty_test_2() -> AppResult<()> {
    let mut block = Block::default();
    block.mine(2).await;
    assert!(block.hash_difficulty() >= 2);
    Ok(())
  }

  // #[async_std::test]
  // async fn difficulty_test_3() -> AppResult<()> {
  //   let mut block = Block::default();
  //   block.mine(3).await;
  //   assert!(block.hash_difficulty() >= 3);
  //   Ok(())
  // }
}
