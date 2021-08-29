use crate::protocol::error::ParseError;
use crate::result::AppResult;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Frame {
  BchainRequest,
  BchainResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BchainRequest {
  GetBlock(usize),
}
