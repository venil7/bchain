use crate::address::Address;
use crate::public_key::PublicKey;
use crate::tx::Tx;
use bchain_util::hash_digest::Hashable;
use bchain_util::result::AppResult;
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
      Err(anyhow::Error::msg("key didnt validate"))
    }
  }

  pub fn new_tx(&self, receiver: &Address, amount: u64) -> AppResult<Tx> {
    Tx::new(self, receiver, amount)
  }

  pub fn new_coinbase_tx(&self, amount: u64) -> AppResult<Tx> {
    Tx::new_coinbase(self, amount)
  }

  pub fn public_key(&self) -> PublicKey {
    let internal_public_key = self.to_public_key();
    let modulus_bytes = internal_public_key.n().to_bytes_be();
    let exp_bytes = internal_public_key.e().to_bytes_be();
    let mut bytes = vec![];
    bytes.extend_from_slice(&modulus_bytes);
    bytes.extend_from_slice(&exp_bytes);
    PublicKey::try_new(&bytes).expect("failed to get public key")
  }

  pub fn address(&self) -> Address {
    self.public_key().to_address()
  }

  pub fn sign_hashable<T: Hashable>(&self, s: &T) -> AppResult<Signature> {
    let digest = s.hash_digest();
    let signature_bytes = self.sign(PADDING, digest.deref())?;
    let signature = Signature::try_from(&signature_bytes[..])?;
    Ok(signature)
  }

  pub fn to_pkcs8_der(&self) -> AppResult<Vec<u8>> {
    let der: PrivateKeyDocument = self.private_key.to_pkcs8_der()?;
    Ok(der.as_ref().to_vec())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bchain_util::result::AppResult;
  const RSAKEY_PEM: &str = "../pem/rsakey.pem";

  #[async_std::test]
  async fn load_wallet_from_pem_test() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await;
    assert!(wallet.is_ok());
    Ok(())
  }

  #[async_std::test]
  async fn load_wallet_from_pem_test_() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let address = wallet.address();
    let address1: Address = format!("{}", address).parse()?;
    assert_eq!(address, address1);
    Ok(())
  }
}
