use crate::chain::address::Address;
use crate::chain::digest::HashDigest;
use crate::chain::digest::Hashable;
use crate::error::AppError;
use pkcs8::PrivateKeyDocument;
use rsa::hash::Hash;
use rsa::PaddingScheme;
use rsa::PublicKeyParts;
use rsa::RSAPrivateKey;
use rsa::RSAPublicKey;
use sha2::Digest;
use sha2::Sha256;
use std::error::Error;
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
  pub async fn from_file(fname: &str) -> Result<Wallet, Box<dyn Error>> {
    let file_content = from_file(fname).await?;
    let private_key_doc = from_pem(&file_content).await?;
    let (private_key, _) = from_priv_key(&private_key_doc).await?;
    if private_key.validate().is_ok() {
      Ok(Wallet { private_key })
    } else {
      Err(Box::new(AppError::new("key didnt validate")))
    }
  }

  pub fn public_address(&self) -> Address {
    let public_key_bytes = self.to_public_key().n().to_bytes_be();
    let public_key_digest: Vec<u8> = Sha256::digest(&public_key_bytes).into_iter().collect();
    let mut digest: HashDigest = [0; 32];
    digest[..32].clone_from_slice(&public_key_digest[..32]);
    Address::from_digest(digest)
  }

  pub fn sign_hashable<T: Hashable>(&self, s: &T) -> Result<Vec<u8>, Box<dyn Error>> {
    let digest = s.hash();
    let padding = PaddingScheme::PKCS1v15Sign {
      hash: Some(Hash::SHA2_256),
    };
    let signature = self.sign(padding, &digest)?;
    Ok(signature)
  }
}

async fn from_pem(s: &str) -> Result<PrivateKeyDocument, AppError> {
  let key = PrivateKeyDocument::from_pem(s)?;
  Ok(key)
}

async fn from_file(fname: &str) -> Result<String, Box<dyn Error>> {
  let mut file = File::open(fname).await?;
  let mut buf = vec![];
  file.read_to_end(&mut buf).await?;
  let str = std::str::from_utf8(&buf)?;
  Ok(str.to_owned())
}

async fn from_priv_key(
  key: &PrivateKeyDocument,
) -> Result<(RSAPrivateKey, RSAPublicKey), Box<dyn Error>> {
  let private_key = RSAPrivateKey::from_pkcs8(key.as_ref())?;
  let public_key = RSAPublicKey::from(&private_key);
  Ok((private_key, public_key))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn load_wallet_from_pem_test() -> Result<(), Box<dyn Error>> {
    let _wallet = Wallet::from_file("./rsakey.pem").await?;
    Ok(())
  }
}
