use core::fmt::Display;
use sha2::Digest;
use sha2::Sha256;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HashDigest([u8; 32]);

impl Display for HashDigest {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", hex::encode(self.0))
  }
}

impl Deref for HashDigest {
  type Target = [u8; 32];

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
    HashDigest([0u8; 32])
  }
}

pub trait AsBytes {
  fn as_bytes(&self) -> Vec<u8>;
}

pub trait Hashable: AsBytes {
  fn hash(&self) -> HashDigest {
    let digest: Vec<u8> = Sha256::digest(&self.as_bytes()).into_iter().collect();
    let mut hash_digest = HashDigest::default();
    hash_digest[..32].clone_from_slice(&digest[..32]);
    hash_digest
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
  use crate::result::DynResult;

  #[test]
  fn string_as_bytes_test() -> DynResult<()> {
    let str = String::from("abc");
    let res = str.as_bytes();
    assert_eq!(res, "abc".as_bytes());
    Ok(())
  }

  #[test]
  fn string_as_hash_test() -> DynResult<()> {
    let str = String::from("abc");
    let res = str.hash();
    assert_eq!(
      format!("{}", res),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    Ok(())
  }
}
