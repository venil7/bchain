use crate::chain::address::Address;
use crate::chain::hash_digest::Hashable;
use crate::chain::public_key::PublicKey;
use crate::error::AppError;
use crate::result::AppResult;
use pkcs8::{FromPrivateKey, PrivateKeyDocument, ToPrivateKey};
use rsa::{PublicKeyParts, RsaPrivateKey};
use std::convert::TryFrom;
use std::ops::Deref;

use super::public_key::PADDING;
use super::signature::Signature;

#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
  private_key: RsaPrivateKey,
}

impl Deref for Wallet {
  type Target = RsaPrivateKey;
  fn deref(&self) -> &<Self as std::ops::Deref>::Target {
    &self.private_key
  }
}

impl Wallet {
  pub async fn from_file(pem_file_path: &str) -> AppResult<Wallet> {
    let private_key = RsaPrivateKey::read_pkcs8_pem_file(pem_file_path)?;
    if private_key.validate().is_ok() {
      Ok(Wallet { private_key })
    } else {
      Err(Box::new(AppError::new("key didnt validate")))
    }
  }

  // pub fn public_key(&self) -> AppResult<PublicKey> {
  //   let internal_public_key = self.to_public_key();
  //   let modulus_bytes = internal_public_key.n().to_bytes_be();
  //   let exp_bytes = internal_public_key.e().to_bytes_be();
  //   let mut bytes = vec![];
  //   bytes.extend_from_slice(&modulus_bytes);
  //   bytes.extend_from_slice(&exp_bytes);
  //   let public_key = PublicKey::try_new(&bytes)?;
  //   Ok(public_key)
  // }

  pub fn public_key(&self) -> PublicKey {
    let internal_public_key = self.to_public_key();
    let modulus_bytes = internal_public_key.n().to_bytes_be();
    let exp_bytes = internal_public_key.e().to_bytes_be();
    let mut bytes = vec![];
    bytes.extend_from_slice(&modulus_bytes);
    bytes.extend_from_slice(&exp_bytes);
    let public_key = PublicKey::try_new(&bytes).expect("failed to get public key");
    public_key
  }

  // pub fn public_address(&self) -> AppResult<Address> {
  //   Ok(self.public_key()?.to_address())
  // }
  pub fn public_address(&self) -> Address {
    self.public_key().to_address()
  }

  pub fn sign_hashable<T: Hashable>(&self, s: &T) -> AppResult<Signature> {
    let digest = s.hash_digest();
    let signature_bytes = self.sign(PADDING, digest.deref())?;
    let signature = Signature::try_from(&signature_bytes[..])?;
    Ok(signature)
  }

  // pub fn new_tx(&self, receiever: &PublicKey, amount: u64) -> AppResult<Tx> {
  //   let mut tx = Tx {
  //     sender: self.public_key()?,
  //     receiver: receiever.clone(),
  //     amount,
  //     signature: None,
  //   };
  //   tx.signature = Some(self.sign_hashable(&tx)?);
  //   Ok(tx)
  // }

  // pub fn verify_tx_signature(&self, tx: &Tx) -> AppResult<()> {
  //   if let Some(ref signature) = tx.signature {
  //     tx.sender.verify_signature(&tx.hash().to_vec(), signature)?;
  //     return Ok(());
  //   }
  //   Err("Tx has to be signed".into())
  // }

  pub fn to_pkcs8_der(&self) -> AppResult<Vec<u8>> {
    let der: PrivateKeyDocument = self.private_key.to_pkcs8_der()?;
    Ok(der.as_ref().to_vec())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[async_std::test]
  async fn load_wallet_from_pem_test() -> AppResult<()> {
    let _wallet = Wallet::from_file("./rsakey.pem").await?;
    Ok(())
  }
}
