use crate::hash_digest::{AsBytes, Hashable};
use crate::public_key::PublicKey;
use crate::result::AppResult;
use crate::signature::Signature;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct Tx {
  amount: u64,
  sender: PublicKey,
  receiver: PublicKey,
  signature: Signature,
}

impl Tx {
  fn transaction_body(amount: u64, sender: &PublicKey, receiver: &PublicKey) -> Vec<u8> {
    let mut transaction_body = vec![];
    transaction_body.extend_from_slice(&amount.as_bytes());
    transaction_body.extend_from_slice(&sender.as_bytes());
    transaction_body.extend_from_slice(&receiver.as_bytes());
    transaction_body
  }

  pub fn new(wallet: &Wallet, receiver: PublicKey, amount: u64) -> AppResult<Tx> {
    let sender = wallet.public_key();
    let transaction_body = Tx::transaction_body(amount, &sender, &receiver);
    let signature = wallet.sign_hashable(&transaction_body)?;
    Ok(Tx {
      amount,
      sender,
      receiver,
      signature,
    })
  }

  pub fn verify_signature(&self) -> AppResult<()> {
    let transaction_body = Tx::transaction_body(self.amount, &self.sender, &self.receiver);
    self
      .sender
      .verify_signature(&transaction_body.hash_digest().to_vec(), &self.signature)
  }
}

impl AsBytes for Tx {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.extend_from_slice(&self.sender.as_bytes());
    res.extend_from_slice(&self.receiver.as_bytes());
    res.extend_from_slice(&self.amount.as_bytes());
    // do not hash signature!
    res
  }
}

impl Hashable for Tx {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{result::AppResult, wallet::Wallet};
  const RSAKEY_PEM: &str = "../rsakey.pem";

  #[async_std::test]
  async fn verify_transaction_serializaton() -> AppResult<()> {
    let wallet = Wallet::from_file(RSAKEY_PEM).await?;

    let tx = Tx::new(&wallet, wallet.public_key(), 1234)?;

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
    let tx = Tx::new(&wallet, wallet.public_key(), 1234)?;
    tx.verify_signature()?;
    Ok(())
  }
}
