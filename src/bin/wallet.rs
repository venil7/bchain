use bchain::chain::address::Address;
use bchain::chain::tx::Tx;
use bchain::chain::wallet::Wallet;
use bchain::result::DynResult;
use std::str::FromStr;

#[tokio::main]
async fn main() -> DynResult<()> {
  let wallet = Wallet::from_file("./rsakey.pem").await?;

  let my_address = format!("{}", wallet.public_address());
  let tx = Tx {
    sender: wallet.public_key(),
    receiver: Address::from_str(&my_address)?,
    amount: 12345,
  };
  println!("--> {:?}", wallet.sign_hashable(&tx)?.len());

  Ok(())
}
