use bchain_domain::{block::Block, tx::Tx};
use bchain_util::hash_digest::{HashDigest, Hashable};
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
  Latest(Block),
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

impl ToString for BchainRequest {
  fn to_string(&self) -> String {
    match self {
      BchainRequest::SubmitBlock(block) => format!("SubmitBlock({})", block.hash_digest()),
      BchainRequest::SubmitTx(tx) => format!("SubmitTx({})", tx.hash_digest()),
      other => format!("{:?}", other),
    }
  }
}
