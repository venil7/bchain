use nom::{bytes::complete::tag, character::complete::space0, sequence::preceded, IResult};

use super::UserCommand;

const HELP_TEXT: &str = "
/peers - display peers
/blocks - list blocks
/bootstrap - run bootstrap again
/msg <some msg> - send message to peers
/dial <addr1> [<addr2>] - dial peer by address
/balance [address] - balance for address, own address used if not specified
/tx <addr> <amount> - send transaction to network 
/help - this help
";

pub(crate) fn help_command(input: &str) -> IResult<&str, UserCommand> {
  let mut command = preceded(tag("/help"), space0);
  let (remainder, _) = command(input)?;
  let help = UserCommand::Help(HELP_TEXT);
  Ok((remainder, help))
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

  #[test]
  fn user_command_help_test() -> AppResult<()> {
    let input = "/help  ";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Help(HELP_TEXT));
    Ok(())
  }
}
