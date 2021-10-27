use nom::{
  bytes::complete::{is_not, tag},
  character::complete::space1,
  multi::separated_list1,
  sequence::preceded,
  IResult,
};

use super::UserCommand;

pub(crate) fn dial_command(input: &str) -> IResult<&str, UserCommand> {
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

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

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
