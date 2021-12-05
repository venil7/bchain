use crate::address::Address;
use bchain_util::error::AppError;
use bchain_util::hash_digest::{AsBytes, Hashable};
use bchain_util::result::AppResult;
use rsa::hash::Hash;
use rsa::{BigUint, PaddingScheme, PublicKey as _, RsaPublicKey};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;

pub const PUBLIC_KEY_LENGTH: usize = 0x100; //256
pub const PADDING: PaddingScheme = PaddingScheme::PKCS1v15Sign {
  hash: Some(Hash::SHA2_256),
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct PublicKey(Vec<u8>);

impl Deref for PublicKey {
  type Target = Vec<u8>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for PublicKey {
  fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
    &mut self.0
  }
}

impl AsRef<[u8]> for PublicKey {
  fn as_ref(&self) -> &[u8] {
    &self.0[..]
  }
}

impl Default for PublicKey {
  fn default() -> Self {
    PublicKey(vec![0u8; PUBLIC_KEY_LENGTH])
  }
}

impl AsBytes for PublicKey {
  fn as_bytes(&self) -> Vec<u8> {
    self.0.to_vec()
  }
}

impl From<Address> for PublicKey {
  fn from(addr: Address) -> Self {
    addr.0
  }
}

impl Hashable for PublicKey {}

impl PublicKey {
  pub fn try_new(bytes: &[u8]) -> AppResult<Self> {
    if bytes.len() >= PUBLIC_KEY_LENGTH {
      Ok(PublicKey(bytes.to_vec()))
    } else {
      Err(AppError::msg(format!(
        "PublicKey has to be at least {} chars long",
        PUBLIC_KEY_LENGTH
      )))
    }
  }

  pub fn to_address(&self) -> Address {
    Address::new(self)
  }

  fn int_public_key(&self) -> AppResult<RsaPublicKey> {
    let modulus = BigUint::from_bytes_be(&self.0[0..PUBLIC_KEY_LENGTH]);
    let exponent = BigUint::from_bytes_be(&self.0[PUBLIC_KEY_LENGTH..]);
    let int_public_key = RsaPublicKey::new(modulus, exponent)?;
    Ok(int_public_key)
  }

  pub fn verify_signature(&self, data: &[u8], sig: &[u8]) -> AppResult<()> {
    self.int_public_key()?.verify(PADDING, data, sig)?;
    Ok(())
  }
}
