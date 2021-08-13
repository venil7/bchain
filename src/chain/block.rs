use super::digest::HashDigest;
use crate::chain::digest::AsBytes;
use crate::chain::digest::Hashable;
use crate::chain::tx::Tx;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  pub timestamp: u64,
  pub parent_hash: Option<HashDigest>,
  pub txs: Vec<Tx>,
}

// impl Bock {
//   pub fn produce_child
// }

impl AsBytes for Block {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.append(&mut self.timestamp.as_bytes());
    res.append(&mut self.parent_hash.as_bytes());
    for tx in &self.txs {
      res.append(&mut tx.as_bytes())
    }
    res
  }
}

impl Hashable for Block {}
