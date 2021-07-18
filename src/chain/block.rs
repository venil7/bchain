use crate::chain::address::Address;
use crate::chain::digest::AsBytes;
use crate::chain::digest::Hashable;

use byteorder::{LittleEndian, WriteBytesExt};
use std::mem;

impl AsBytes for f64 {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut bs = [0u8; mem::size_of::<f64>()];
    bs.as_mut()
      .write_f64::<LittleEndian>(*self)
      .expect("Unable to convert f64");
    bs.to_vec()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tx {
  pub from: Address,
  pub to: Address,
  pub amount: f64,
}

impl AsBytes for Tx {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.append(&mut self.from.as_bytes());
    res.append(&mut self.to.as_bytes());
    res.append(&mut self.amount.as_bytes());

    res
  }
}
impl Hashable for Tx {}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
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
