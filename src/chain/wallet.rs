use crate::chain::address::Address;
use crate::chain::digest::Hashable;
use crate::chain::public_key::PublicKey;
use crate::error::AppError;
use crate::result::AppResult;
use pkcs8::{FromPrivateKey, PrivateKeyDocument, ToPrivateKey};
use rsa::{PublicKeyParts, RsaPrivateKey};
use std::convert::TryFrom;
use std::ops::Deref;

use super::public_key::PADDING;
use super::signature::Signature;
use super::tx::Tx;

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

  pub fn public_key(&self) -> AppResult<PublicKey> {
    let internal_public_key = self.to_public_key();
    let modulus_bytes = internal_public_key.n().to_bytes_be();
    let exp_bytes = internal_public_key.e().to_bytes_be();
    let public_key = PublicKey::try_new(&modulus_bytes[..], &exp_bytes[..])?;
    Ok(public_key)
  }

  pub fn public_address(&self) -> AppResult<Address> {
    Ok(self.public_key()?.to_address())
  }

  fn sign_hashable<T: Hashable>(&self, s: &T) -> AppResult<Signature> {
    let digest = s.hash();
    let signature_bytes = self.sign(PADDING, digest.deref())?;
    let signature = Signature::try_from(&signature_bytes[..])?;
    Ok(signature)
  }

  pub fn new_tx(&self, receiever: &PublicKey, amount: u64) -> AppResult<Tx> {
    let mut tx = Tx {
      sender: self.public_key()?,
      receiver: receiever.clone(),
      amount,
      signature: None,
    };
    tx.signature = Some(self.sign_hashable(&tx)?);
    Ok(tx)
  }

  pub fn verify_tx_signature(&self, tx: &Tx) -> AppResult<()> {
    if let Some(ref signature) = tx.signature {
      tx.sender.verify_signature(&tx.hash().to_vec(), signature)?;
      return Ok(());
    }
    Err("Tx has to be signed".into())
  }

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

  #[async_std::test]
  async fn verify_legit_tx() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;
    let tx = wallet.new_tx(&wallet.public_key()?, 1234)?;
    wallet.verify_tx_signature(&tx)?;
    Ok(())
  }

  #[async_std::test]
  async fn verify_legit_tx_from_another_wallet() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;
    let wallet1 = Wallet::from_file("./rsakey1.pem").await?;
    let tx = wallet.new_tx(&wallet1.public_key()?, 1234)?;
    wallet.verify_tx_signature(&tx)?;
    wallet1.verify_tx_signature(&tx)?;
    Ok(())
  }
}
