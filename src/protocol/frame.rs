use crate::chain::{block::Block, tx::Tx};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Frame {
  BchainRequest,
  BchainResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainRequest {
  GetBlock(usize),
  SubmitBlock(Block),
  SubmitTransaction(Tx),
}
