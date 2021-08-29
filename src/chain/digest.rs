use byteorder::{LittleEndian, WriteBytesExt};
use core::fmt::Display;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::convert::TryFrom;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::error::AppError;

const HASH_LENGTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HashDigest([u8; HASH_LENGTH]);

impl Display for HashDigest {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", hex::encode(self.0))
  }
}

impl Deref for HashDigest {
  type Target = [u8; HASH_LENGTH];

  fn deref(&self) -> &Self::Target {
    &&self.0
  }
}

impl DerefMut for HashDigest {
  fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
    &mut self.0
  }
}

impl Default for HashDigest {
  fn default() -> Self {
    HashDigest([0u8; HASH_LENGTH])
  }
}
impl AsBytes for HashDigest {
  fn as_bytes(&self) -> Vec<u8> {
    self.0.to_vec()
  }
}
impl<T> AsBytes for Option<T>
where
  T: AsBytes,
{
  fn as_bytes(&self) -> Vec<u8> {
    match self {
      Some(n) => n.as_bytes(),
      _ => vec![],
    }
  }
}

impl From<Vec<u8>> for HashDigest {
  fn from(vec: Vec<u8>) -> Self {
    assert!(vec.len() == HASH_LENGTH);
    let mut hash_digest = HashDigest::default();
    hash_digest[..HASH_LENGTH].clone_from_slice(&vec[..HASH_LENGTH]);
    hash_digest
  }
}

impl TryFrom<String> for HashDigest {
  type Error = AppError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    let v = hex::decode(value)?;
    Ok(v.into())
  }
}

pub trait AsBytes {
  fn as_bytes(&self) -> Vec<u8>;
}

impl AsBytes for u64 {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut bs = [0u8; mem::size_of::<u64>()];
    bs.as_mut()
      .write_u64::<LittleEndian>(*self)
      .expect("Unable to convert u64");
    bs.to_vec()
  }
}

pub trait Hashable: AsBytes {
  fn hash(&self) -> HashDigest {
    let digest: Vec<u8> = Sha256::digest(&self.as_bytes()).into_iter().collect();
    digest.into()
  }
}

impl AsBytes for String {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    self.as_bytes().to_vec()
  }
}

impl Hashable for String {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::result::AppResult;

  #[test]
  fn string_as_bytes_test() -> AppResult<()> {
    let str = String::from("abc");
    let res = str.as_bytes();
    assert_eq!(res, "abc".as_bytes());
    Ok(())
  }

  #[test]
  fn string_as_hash_test() -> AppResult<()> {
    let str = String::from("abc");
    let res = str.hash();
    assert_eq!(
      format!("{}", res),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    Ok(())
  }
}
