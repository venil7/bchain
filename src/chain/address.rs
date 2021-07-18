use crate::chain::digest::AsBytes;
use crate::chain::digest::HashDigest;
use crate::error::AppError;
use std::fmt::Display;
use std::ops::DerefMut;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Address(HashDigest);

impl Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", self.0)
  }
}

impl Address {
  pub fn from_digest(digest: HashDigest) -> Address {
    Address(digest)
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
    let mut bytes = HashDigest::default();
    let _ = hex::decode_to_slice(addr, bytes.deref_mut())?;
    Ok(Address(bytes))
  }
}
