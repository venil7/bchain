use bchain_domain::{block::Block, hash_digest::HashDigest, tx::Tx};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Frame {
  BchainRequest(BchainRequest),
  BchainResponse(BchainResponse),
  Unrecognized,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainRequest {
  AskLatest,
  AskBlock(i64),
  SubmitBlock(Block),
  SubmitTx(Tx),
  Msg(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainResponse {
  Latest(i64),
  Block(Block),
  AcceptBlock(HashDigest),
  AcceptTx(HashDigest),
  Error(BchainError),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainError {
  Tx(HashDigest),
  Block(HashDigest),
  Generic(String),
}
