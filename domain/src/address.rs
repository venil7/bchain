use crate::public_key::PublicKey;
use bchain_util::error::AppError;
use bchain_util::hash_digest::{AsBytes, Hashable};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(PublicKey);

impl Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let friendly = bs58::encode(&self.0).into_string();
    write!(f, "{}", friendly)
  }
}

impl Address {
  pub fn new(public_key: &PublicKey) -> Address {
    Address(public_key.clone())
  }
}

impl AsBytes for Address {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    self.0.to_vec()
  }
}

impl Hashable for Address {}
// impl ShortDisplay for Address {}

impl FromStr for Address {
  type Err = AppError;
  fn from_str(addr: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
    match bs58::decode(addr).into_vec() {
      Ok(bytes) => Ok(Address(PublicKey::try_new(&bytes)?)),
      Err(e) => Err(AppError::msg(format!("{:?}", e))),
    }
  }
}

#[cfg(test)]
mod tests {
  use bchain_util::result::AppResult;

  use super::*;

  #[test]
  fn address_parse_test() -> AppResult<()> {
    let input = "FzpuKhDdqVu7Q3E7bCJLHnWGGxgaPjN9pi9ScvJiLt1XnFdrP1RBUTzpVkAGN2mNcUtAFrCVF1x7PbnKJRCHcXs2nEusKLnuFKR6fA4vXZC92vMDoWip71eUy7yGfFcFNTF17oHUrvPAwxfu2NKFp2wb8xtYPV4vCHowKG2Bh3kT5DVxjmjzDuNVSU6StVX3Lx7nj5Wz7AkmHL9rszTPQuVpfpLWQwUSnLb2Q4XfUsTCpuCvnxQDaxE8wH8nw7xBZV5SL8v4idCrqQVjcEt5uddwBRyYgEiGJyysYjiWWdfpf7QeoG6Qj4C9ZYmXCRqRJxJAd1Gioey2iF4stkxxEmLurwrR8r7sma";
    let address: Address = input.parse()?;
    assert_eq!(format!("{}", address), input);
    Ok(())
  }
}
