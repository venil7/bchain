use crate::address::Address;
use crate::public_key::PublicKey;
use crate::signature::Signature;
use crate::wallet::Wallet;
use bchain_util::hash_digest::{AsBytes, Hashable};
use bchain_util::result::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tx {
  amount: u64,
  timestamp: i64,
  sender: Address, //PublicKey::default() for coinbase tx
  receiver: Address,
  signature: Signature,
}

impl Tx {
  fn transaction_body(amount: u64, sender: &Address, receiver: &Address) -> Vec<u8> {
    let mut transaction_body = vec![];
    transaction_body.extend_from_slice(&amount.as_bytes());
    transaction_body.extend_from_slice(&sender.as_bytes());
    transaction_body.extend_from_slice(&receiver.as_bytes());
    transaction_body
  }

  pub fn new_coinbase(wallet: &Wallet, amount: u64) -> AppResult<Tx> {
    let sender = Address::default();
    let receiver = wallet.address();
    let transaction_body = Tx::transaction_body(amount, &sender, &receiver);
    let signature = wallet.sign_hashable(&transaction_body)?;
    let timestamp = Utc::now().timestamp();
    Ok(Tx {
      amount,
      timestamp,
      sender,
      receiver,
      signature,
    })
  }

  pub fn new(wallet: &Wallet, receiver: &Address, amount: u64) -> AppResult<Tx> {
    let sender = wallet.address();
    let transaction_body = Tx::transaction_body(amount, &sender, receiver);
    let signature = wallet.sign_hashable(&transaction_body)?;
    let receiver = receiver.clone();
    let timestamp = Utc::now().timestamp();
    Ok(Tx {
      amount,
      timestamp,
      sender,
      receiver,
      signature,
    })
  }

  pub fn verify_signature(&self) -> AppResult<()> {
    let transaction_body = Tx::transaction_body(self.amount, &self.sender, &self.receiver);
    let pub_key: PublicKey = self.sender.clone().into();
    pub_key.verify_signature(&transaction_body.hash_digest().to_vec(), &self.signature)
  }

  pub fn diff_for_address(&self, address: &Address) -> i64 {
    if address == &self.sender {
      0 - (self.amount as i64)
    } else if address == &self.receiver {
      self.amount as i64
    } else {
      0
    }
  }
}

impl AsBytes for Tx {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.extend_from_slice(&self.amount.as_bytes());
    res.extend_from_slice(&self.timestamp.as_bytes());
    res.extend_from_slice(&self.sender.as_bytes());
    res.extend_from_slice(&self.receiver.as_bytes());
    // do not hash signature
    // signature used to sign bytes, so cant be included!
    res
  }
}

impl Hashable for Tx {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::wallet::Wallet;
  const RSAKEY_PEM: &str = "../pem/rsakey.pem";

  #[async_std::test]
  async fn verify_transaction_serializaton() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;

    let tx = Tx::new(&wallet, &wallet.address(), 1234)?;

    let json = serde_json::to_string(&tx)?;
    let hash = tx.hash_digest();

    let tx1: Tx = serde_json::from_str(&json)?;
    let hash1 = tx1.hash_digest();

    assert_eq!(hash, hash1);

    Ok(())
  }

  #[async_std::test]
  async fn verify_legit_tx() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let tx = Tx::new(&wallet, &wallet.address(), 1234)?;
    tx.verify_signature()?;
    Ok(())
  }

  #[async_std::test]
  async fn reject_illegitimate_tx() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;
    let mut tx = Tx::new(&wallet, &wallet.address(), 1234)?;
    tx.signature = Default::default(); // assigning wrong signature
    let sig_verify_result = tx.verify_signature();
    assert!(sig_verify_result.is_err());
    Ok(())
  }
}
