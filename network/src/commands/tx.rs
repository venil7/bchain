use super::UserCommand;

use nom::{
  bytes::complete::tag,
  character::complete::{alphanumeric1, digit1, space1},
  combinator::eof,
  sequence::{preceded, tuple},
  IResult,
};

pub(crate) fn tx_command(input: &str) -> IResult<&str, UserCommand> {
  let command = preceded(tag("/tx"), space1);
  let mut command = preceded(command, tuple((alphanumeric1, space1, digit1, eof)));
  let (remainder, (recipient, _, amount, _)) = command(input)?;

  let recipient = recipient.parse();
  let amount = amount.parse();

  match (recipient, amount) {
    (Ok(recipient), Ok(amount)) => Ok((remainder, UserCommand::Tx(recipient, amount))),
    _ => Ok((remainder, UserCommand::Unrecognized)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;

  const ADDRESS: &str = "FzpuKhDdqVu7Q3E7bCJLHnWGGxgaPjN9pi9ScvJiLt1XnFdrP1RBUTzpVkAGN2mNcUtAFrCVF1x7PbnKJRCHcXs2nEusKLnuFKR6fA4vXZC92vMDoWip71eUy7yGfFcFNTF17oHUrvPAwxfu2NKFp2wb8xtYPV4vCHowKG2Bh3kT5DVxjmjzDuNVSU6StVX3Lx7nj5Wz7AkmHL9rszTPQuVpfpLWQwUSnLb2Q4XfUsTCpuCvnxQDaxE8wH8nw7xBZV5SL8v4idCrqQVjcEt5uddwBRyYgEiGJyysYjiWWdfpf7QeoG6Qj4C9ZYmXCRqRJxJAd1Gioey2iF4stkxxEmLurwrR8r7sma";

  #[test]
  fn user_command_tx_positive_test() -> AppResult<()> {
    let input = format!("/tx {} {}", ADDRESS, 123);
    let tx: UserCommand = input.parse()?;
    assert_eq!(tx, UserCommand::Tx(ADDRESS.parse()?, 123));
    Ok(())
  }

  #[test]
  fn user_command_tx_negative_test() -> AppResult<()> {
    let input = "/tx someaddress123 abc";
    let msg = input.parse::<UserCommand>()?;
    assert_eq!(msg, UserCommand::Unrecognized);
    Ok(())
  }
}
