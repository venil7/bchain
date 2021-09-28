use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::{alphanumeric1, digit1, space1},
  combinator::{eof, rest},
  sequence::{preceded, tuple},
  IResult,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::AppError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserCommand {
  Msg(String),
  Tx { recipient: String, amount: u64 },
  Unrecognized,
}

impl FromStr for UserCommand {
  type Err = AppError;

  fn from_str(msg: &str) -> Result<Self, Self::Err> {
    if let Ok((_, cmd)) = alt((message_command, tx_command))(msg) {
      Ok(cmd)
    } else {
      Ok(UserCommand::Unrecognized)
    }
  }
}

pub fn message_command(input: &str) -> IResult<&str, UserCommand> {
  let command = preceded(tag("/msg"), space1);
  let mut command = preceded(command, rest);
  let (remainder, message) = command(input)?;
  Ok((remainder, UserCommand::Msg(message.into())))
}

pub fn tx_command(input: &str) -> IResult<&str, UserCommand> {
  let command = preceded(tag("/tx"), space1);
  let mut command = preceded(command, tuple((alphanumeric1, space1, digit1, eof)));
  let (remainder, (recipient, _, amount, _)) = command(input)?;
  Ok((
    remainder,
    UserCommand::Tx {
      recipient: recipient.into(),
      amount: amount.parse().expect("failed to parse amount"),
    },
  ))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::result::AppResult;

  #[test]
  fn user_command_message_positive_test() -> AppResult<()> {
    let input = "/msg some text here 111";
    let msg: UserCommand = input.parse()?;
    assert_eq!(msg, UserCommand::Msg("some text here 111".into()));
    Ok(())
  }

  #[test]
  fn user_command_message_negative_test() -> AppResult<()> {
    let input = "some text here 111";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Unrecognized);
    Ok(())
  }

  #[test]
  fn user_command_tx_positive_test() -> AppResult<()> {
    let input = "/tx someaddress123 123";
    let msg: UserCommand = input.parse()?;
    assert_eq!(
      msg,
      UserCommand::Tx {
        recipient: "someaddress123".into(),
        amount: 123
      }
    );
    Ok(())
  }

  #[test]
  fn user_command_tx_negative_test() -> AppResult<()> {
    let input = "/tx someaddress123 123.45";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Unrecognized);
    Ok(())
  }
}
