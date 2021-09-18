use bchain::chain::wallet::Wallet;
use bchain::cli::Cli;
use bchain::result::AppResult;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> AppResult<()> {
  let cli = Cli::from_args();
  let wallet = Wallet::from_file(&cli.wallet).await?;

  println!("{}", wallet.public_address()?);

  Ok(())
}
