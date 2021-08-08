use crate::chain::digest::AsBytes;
use crate::chain::digest::Hashable;
use crate::chain::tx::Tx;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  pub timestamp: u64,
  pub txs: Vec<Tx>,
}

impl AsBytes for Block {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    for tx in &self.txs {
      res.append(&mut tx.as_bytes())
    }
    res
  }
}

impl Hashable for Block {}
