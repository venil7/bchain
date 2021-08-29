use base58::FromBase58;
use base58::ToBase58;
use sha2::Digest;
use sha2::Sha256;

use crate::chain::digest::AsBytes;
use crate::chain::digest::HashDigest;
use crate::chain::public_key::PublicKey;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

const ADDRESS_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(HashDigest);

impl Display for Address {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let friendly = self.0.to_base58();
    write!(f, "{}", friendly)
  }
}

impl Address {
  pub fn from_digest(digest: HashDigest) -> Address {
    Address(digest)
  }
  pub fn from_public_key(public_key: &PublicKey) -> Address {
    let public_key_digest: Vec<u8> = Sha256::digest(&public_key[..]).into_iter().collect();
    let mut digest = HashDigest::default();
    digest[..ADDRESS_LENGTH].clone_from_slice(&public_key_digest[..ADDRESS_LENGTH]);
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
    let bytes = addr.from_base58()?;
    Ok(Address(bytes.into()))
  }
}
