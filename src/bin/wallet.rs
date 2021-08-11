use bchain::chain::address::Address;
use bchain::chain::tx::Tx;
use bchain::chain::wallet::Wallet;
use bchain::cli::Cli;
use bchain::result::AppResult;
use std::str::FromStr;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> AppResult<()> {
  let cli = Cli::from_args();
  let wallet = Wallet::from_file(&cli.wallet).await?;

  let my_address = format!("{}", wallet.public_address());
  let tx = Tx {
    sender: wallet.public_key(),
    receiver: Address::from_str(&my_address)?,
    amount: 12345,
  };
  println!("--> {:?}", wallet.sign_hashable(&tx)?.len());

  Ok(())
}
