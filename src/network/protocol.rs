use std::str::FromStr;

use crate::{
  chain::{block::Block, hash_digest::HashDigest, tx::Tx},
  error::AppError,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Frame {
  BchainRequest(BchainRequest),
  BchainResponse(BchainResponse),
  Unrecognized,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainRequest {
  GetBlock(usize),
  SubmitBlock(Block),
  SubmitTx(Tx),
  Msg(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainResponse {
  Block(Block),
  BlockAccepted(HashDigest),
  TxAccepted(HashDigest),
  Error(BchainError),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainError {
  Tx(HashDigest),
  Block(HashDigest),
  Generic(String),
}

impl FromStr for Frame {
  type Err = AppError;

  fn from_str(msg: &str) -> Result<Self, Self::Err> {
    // todo!()
    Ok(Frame::BchainRequest(BchainRequest::Msg(msg.into())))
  }
}
