use crate::chain::digest::AsBytes;
use crate::chain::public_key::PublicKey;
use crate::error::AppError;
use base58::FromBase58;
use base58::ToBase58;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(PublicKey);

impl Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let friendly = self.0.to_base58();
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

impl FromStr for Address {
  type Err = AppError;
  fn from_str(addr: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
    let bytes = addr.from_base58()?;
    let public_key = PublicKey::try_new(&bytes)?;
    Ok(Address(public_key))
  }
}
