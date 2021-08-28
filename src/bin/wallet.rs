use bchain::chain::digest::Hashable;
use bchain::chain::tx::Tx;
use bchain::chain::wallet::Wallet;
use bchain::cli::Cli;
use bchain::result::AppResult;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> AppResult<()> {
  let cli = Cli::from_args();
  let wallet = Wallet::from_file(&cli.wallet).await?;

  let mut tx = Tx {
    sender: wallet.public_key()?,
    receiver: wallet.public_key()?,
    amount: 12345,
    signature: None,
  };
  tx.signature = Some(wallet.sign_hashable(&tx)?);

  let json = serde_json::to_string(&tx)?;
  let hash = tx.hash();

  println!("{}\n{}", json, hash);

  Ok(())
}
