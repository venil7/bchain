use bchain_domain::error::AppError;
use nom::{
  branch::alt,
  bytes::complete::{is_not, tag},
  character::complete::{alphanumeric1, digit1, space0, space1},
  combinator::{eof, rest},
  multi::separated_list1,
  sequence::{preceded, tuple},
  IResult,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserCommand {
  Msg(String),
  Tx { recipient: String, amount: u64 },
  Blocks,
  Dial(Vec<String>),
  Peers,
  Bootstrap,
  Unrecognized,
}

impl FromStr for UserCommand {
  type Err = AppError;

  fn from_str(msg: &str) -> Result<Self, Self::Err> {
    if let Ok((_, cmd)) = alt((
      message_command,
      tx_command,
      peers_command,
      dial_command,
      bootstrap_command,
      blocks_command,
    ))(msg)
    {
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

pub fn peers_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/peers"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Peers))
}

pub fn bootstrap_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/bootstrap"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Bootstrap))
}

pub fn dial_command(input: &str) -> IResult<&str, UserCommand> {
  let command = preceded(tag("/dial"), space1);
  let non_whitespace = is_not(" \t\r\n");
  let mut command = preceded(command, separated_list1(space1, non_whitespace));
  let (remainder, peers) = command(input)?;
  let peers = peers
    .iter()
    .map(|&s| String::from(s))
    .collect::<Vec<String>>();
  Ok((remainder, UserCommand::Dial(peers)))
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

pub fn blocks_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/blocks"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Blocks))
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_domain::result::AppResult;

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

  #[test]
  fn user_command_peers_test() -> AppResult<()> {
    let input = "/peers";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Peers);
    Ok(())
  }

  #[test]
  fn user_command_bootstrap_test() -> AppResult<()> {
    let input = "/bootstrap";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Bootstrap);
    Ok(())
  }

  #[test]
  fn user_command_dial_test() -> AppResult<()> {
    let input = "/dial  abc 123 :://";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(
      msg,
      UserCommand::Dial(vec!["abc".into(), "123".into(), ":://".into()])
    );
    Ok(())
  }
}
