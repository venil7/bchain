use super::digest::{AsBytes, Hashable};
use super::public_key::PublicKey;
use super::signature::Signature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct Tx {
  pub sender: PublicKey,
  pub receiver: PublicKey,
  pub amount: u64,
  pub signature: Option<Signature>,
}

impl AsBytes for Tx {
  fn as_bytes(&self) -> std::vec::Vec<u8> {
    let mut res = vec![];
    res.append(&mut self.sender.as_bytes());
    res.append(&mut self.receiver.as_bytes());
    res.append(&mut self.amount.as_bytes());
    // do not hash signature!
    res
  }
}

impl Hashable for Tx {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{chain::wallet::Wallet, result::AppResult};

  #[tokio::test]
  async fn verify_transaction_serializaton() -> AppResult<()> {
    let wallet = Wallet::from_file("./rsakey.pem").await?;

    let tx = wallet.new_tx(&wallet.public_key()?, 1234)?;

    let json = serde_json::to_string(&tx)?;
    let hash = tx.hash();

    let tx1: Tx = serde_json::from_str(&json)?;
    let hash1 = tx1.hash();

    assert_eq!(hash, hash1);

    Ok(())
  }
}
