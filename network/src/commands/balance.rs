use super::UserCommand;

use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::{alphanumeric1, multispace1},
  combinator::eof,
  sequence::preceded,
  IResult,
};

pub(crate) fn balance_command(input: &str) -> IResult<&str, UserCommand> {
  let command = tag("/balance");
  let address = alphanumeric1;
  let maybe_address = preceded(multispace1, alt((address, eof)));
  let mut command = preceded(command, maybe_address);
  let (remainder, addr) = command(input)?;

  match addr.parse() {
    Ok(addr) => Ok((remainder, UserCommand::Balance(Some(addr)))),
    _ => Ok((remainder, UserCommand::Balance(None))),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_domain::result::AppResult;

  #[test]
  fn user_comand_balance_test() -> AppResult<()> {
    let address = "FzpuKhDdqVu7Q3E7bCJLHnWGGxgaPjN9pi9ScvJiLt1XnFdrP1RBUTzpVkAGN2mNcUtAFrCVF1x7PbnKJRCHcXs2nEusKLnuFKR6fA4vXZC92vMDoWip71eUy7yGfFcFNTF17oHUrvPAwxfu2NKFp2wb8xtYPV4vCHowKG2Bh3kT5DVxjmjzDuNVSU6StVX3Lx7nj5Wz7AkmHL9rszTPQuVpfpLWQwUSnLb2Q4XfUsTCpuCvnxQDaxE8wH8nw7xBZV5SL8v4idCrqQVjcEt5uddwBRyYgEiGJyysYjiWWdfpf7QeoG6Qj4C9ZYmXCRqRJxJAd1Gioey2iF4stkxxEmLurwrR8r7sma";
    let input = format!("/balance {}", address);
    let msg: UserCommand = input.parse()?;
    assert_eq!(msg, UserCommand::Balance(Some(address.parse()?)));
    Ok(())
  }

  #[test]
  fn user_comand_balance_test_2() -> AppResult<()> {
    let input = "/balance ";
    let msg: UserCommand = input.parse()?;
    assert_eq!(msg, UserCommand::Balance(None));
    Ok(())
  }

  #[test]
  fn user_comand_balance_test_3() -> AppResult<()> {
    let input = "/balance $%^^&&*((";
    let msg: UserCommand = input.parse()?;
    assert_eq!(msg, UserCommand::Unrecognized);
    Ok(())
  }
}
