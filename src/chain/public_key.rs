use crate::chain::address::Address;
use crate::chain::digest::AsBytes;
use crate::error::AppError;
use crate::result::AppResult;
use rsa::hash::Hash;
use rsa::{BigUint, PaddingScheme, PublicKey as _, RsaPublicKey};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;

pub const PUBLIC_KEY_LENGTH: usize = 0x100; //256
pub const PADDING: PaddingScheme = PaddingScheme::PKCS1v15Sign {
  hash: Some(Hash::SHA2_256),
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicKey(Vec<u8>, Vec<u8>);

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

impl Default for PublicKey {
  fn default() -> Self {
    PublicKey(vec![0u8; PUBLIC_KEY_LENGTH], vec![])
  }
}

impl AsBytes for PublicKey {
  fn as_bytes(&self) -> Vec<u8> {
    let mut res = vec![];
    res.append(&mut self.0.to_vec());
    res.append(&mut self.1.to_vec());
    res
  }
}

impl PublicKey {
  pub fn try_new(modulus: &[u8], exponent: &[u8]) -> AppResult<Self> {
    if modulus.len() == PUBLIC_KEY_LENGTH {
      Ok(PublicKey(modulus.to_vec(), exponent.to_vec()))
    } else {
      Err(Box::new(AppError::new(&format!(
        "PublicKey has to be {} chars long",
        PUBLIC_KEY_LENGTH
      ))))
    }
  }
  pub fn to_address(&self) -> Address {
    Address::from_public_key(self)
  }

  fn int_public_key(&self) -> AppResult<RsaPublicKey> {
    let modulus = BigUint::from_bytes_be(&self.0);
    let exponent = BigUint::from_bytes_be(&self.1);
    let int_public_key = RsaPublicKey::new(modulus, exponent)?;
    Ok(int_public_key)
  }

  pub fn verify_signature(&self, data: &[u8], sig: &[u8]) -> AppResult<()> {
    self.int_public_key()?.verify(PADDING, data, sig)?;
    Ok(())
  }
}
