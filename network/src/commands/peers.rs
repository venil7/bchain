use super::UserCommand;

use nom::{bytes::complete::tag, character::complete::space0, sequence::preceded, IResult};

pub(crate) fn peers_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/peers"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Peers))
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

  #[test]
  fn user_command_peers_test() -> AppResult<()> {
    let input = "/peers";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Peers);
    Ok(())
  }
}
