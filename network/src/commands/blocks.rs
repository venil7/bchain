use nom::{bytes::complete::tag, character::complete::space0, sequence::preceded, IResult};

use super::UserCommand;

pub(crate) fn blocks_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/blocks"), space0);
  let (remainder, _) = command(input)?;
  Ok((remainder, UserCommand::Blocks))
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

  #[test]
  fn user_command_bootstrap_test() -> AppResult<()> {
    let input = "/blocks ";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Blocks);
    Ok(())
  }
}
