use crate::chain::address::Address;
use crate::chain::digest::HashDigest;
use crate::chain::digest::Hashable;
use crate::chain::public_key::PublicKey;
use crate::error::AppError;
use crate::result::AppResult;
use pkcs8::PrivateKeyDocument;
use rsa::hash::Hash;
use rsa::PaddingScheme;
use rsa::PublicKeyParts;
use rsa::RSAPrivateKey;
use rsa::RSAPublicKey;
use std::ops::Deref;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
  private_key: RSAPrivateKey,
}

impl Deref for Wallet {
  type Target = RSAPrivateKey;
  fn deref(&self) -> &<Self as std::ops::Deref>::Target {
    &self.private_key
  }
}

impl Wallet {
  pub async fn from_file(fname: &str) -> AppResult<Wallet> {
    let file_content = from_file(fname).await?;
    let private_key_doc = from_pem(&file_content).await?;
    let (private_key, _) = from_priv_key(&private_key_doc).await?;
    if private_key.validate().is_ok() {
      Ok(Wallet { private_key })
    } else {
      Err(Box::new(AppError::new("key didnt validate")))
    }
  }

  pub fn public_key(&self) -> PublicKey {
    let public_key_bytes = self.to_public_key().n().to_bytes_be();
    let mut public_key = PublicKey::default();
    public_key[..256].clone_from_slice(&public_key_bytes[..256]);
    public_key
  }

  pub fn public_address(&self) -> Address {
    self.public_key().to_address()
  }

  pub fn sign_hashable<T: Hashable>(&self, s: &T) -> AppResult<HashDigest> {
    let digest = s.hash();
    let padding = PaddingScheme::PKCS1v15Sign {
      hash: Some(Hash::SHA2_256),
    };
    let signature = self.sign(padding, digest.deref())?;
    Ok(signature.into())
  }
}

async fn from_pem(s: &str) -> Result<PrivateKeyDocument, AppError> {
  let key = PrivateKeyDocument::from_pem(s)?;
  Ok(key)
}

async fn from_file(fname: &str) -> AppResult<String> {
  let mut file = File::open(fname).await?;
  let mut buf = vec![];
  file.read_to_end(&mut buf).await?;
  let str = std::str::from_utf8(&buf)?;
  Ok(str.to_owned())
}

async fn from_priv_key(key: &PrivateKeyDocument) -> AppResult<(RSAPrivateKey, RSAPublicKey)> {
  let private_key = RSAPrivateKey::from_pkcs8(key.as_ref())?;
  let public_key = RSAPublicKey::from(&private_key);
  Ok((private_key, public_key))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn load_wallet_from_pem_test() -> AppResult<()> {
    let _wallet = Wallet::from_file("./rsakey.pem").await?;
    Ok(())
  }
}
