use crate::{
  chain::{block::Block, hash_digest::HashDigest, tx::Tx},
  error::AppError,
};
use nom::{
  bytes::complete::tag, character::complete::space1, combinator::rest, sequence::preceded, IResult,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    if let Ok((_, frame)) = frame_message(msg) {
      Ok(frame)
    } else {
      Err(AppError::new("error parsing user command"))
    }
  }
}

pub fn frame_message(input: &str) -> IResult<&str, Frame> {
  let command = preceded(tag("/msg"), space1);
  let mut msg = preceded(command, rest);
  let (a, b) = msg(input)?;
  Ok((a, Frame::BchainRequest(BchainRequest::Msg(b.into()))))
}

#[cfg(test)]
mod tests {
  use structopt::clap::App;

  use crate::result::AppResult;

  use super::*;

  #[test]
  fn frame_message_positive_test() -> AppResult<()> {
    let input = "/msg some text here 111";
    let msg: Frame = input.parse()?;
    assert_eq!(
      msg,
      Frame::BchainRequest(BchainRequest::Msg("some text here 111".into()))
    );
    Ok(())
  }

  #[test]
  fn frame_message_negative_test() -> AppResult<()> {
    let input = "some text here 111";
    let msg = input.parse::<Frame>();
    assert_eq!(msg, Err(AppError::new("error parsing user command")));
    Ok(())
  }
}
