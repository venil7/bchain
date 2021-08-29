use super::digest::HashDigest;
use crate::chain::digest::AsBytes;
use crate::chain::digest::Hashable;
use crate::chain::tx::Tx;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
  pub timestamp: u64,
  pub txs: HashMap<String, Tx>,
  pub parent_hash: Option<HashDigest>,
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
    res
  }
}

impl Hashable for Block {}
