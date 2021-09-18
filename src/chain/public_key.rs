use crate::chain::address::Address;
use crate::chain::digest::AsBytes;
use crate::error::AppError;
use crate::result::AppResult;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::ops::DerefMut;

const PUBLIC_KEY_LENGTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl PublicKey {
  pub fn try_new(v: &[u8]) -> AppResult<Self> {
    if v.len() == PUBLIC_KEY_LENGTH {
      Ok(PublicKey(v.to_vec()))
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
}
