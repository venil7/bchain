use nom::{bytes::complete::tag, character::complete::space0, sequence::preceded, IResult};

use super::UserCommand;

pub(crate) fn bootstrap_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/bootstrap"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Bootstrap))
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

  #[test]
  fn user_command_bootstrap_test() -> AppResult<()> {
    let input = "/bootstrap";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Bootstrap);
    Ok(())
  }
}
