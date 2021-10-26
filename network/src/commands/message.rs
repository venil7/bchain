use super::UserCommand;

use nom::{
  bytes::complete::tag, character::complete::space1, combinator::rest, sequence::preceded, IResult,
};

pub(crate) fn message_command(input: &str) -> IResult<&str, UserCommand> {
  let command = preceded(tag("/msg"), space1);
  let mut command = preceded(command, rest);
  let (remainder, message) = command(input)?;
  Ok((remainder, UserCommand::Msg(message.into())))
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
}
