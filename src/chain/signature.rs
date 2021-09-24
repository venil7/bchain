use crate::chain::hash_digest::AsBytes;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ops::Deref;
use std::ops::DerefMut;

const SIGNATURE_LENGTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature(Vec<u8>);

impl Deref for Signature {
  type Target = Vec<u8>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Signature {
  fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
    &mut self.0
  }
}

impl Default for Signature {
  fn default() -> Self {
    Signature(vec![0u8; SIGNATURE_LENGTH])
  }
}

impl AsBytes for Signature {
  fn as_bytes(&self) -> Vec<u8> {
    self.0.to_vec()
  }
}

impl TryFrom<&[u8]> for Signature {
  type Error = AppError;

  fn try_from(vec: &[u8]) -> Result<Self, AppError> {
    if vec.len() == SIGNATURE_LENGTH {
      Ok(Signature(vec.to_vec()))
    } else {
      Err(AppError::new(&format!(
        "Signature has to be {} chars long",
        SIGNATURE_LENGTH
      )))
    }
  }
}
