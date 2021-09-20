use structopt::StructOpt;

pub const DEFAULT_LISTEN: &str = "/ip4/0.0.0.0/tcp/0";
pub const DEFAULT_WALLET: &str = "./rsakey.pem";
pub const DEFAULT_DATABASE: &str = "./chain.sqlite";
pub const DEFAULT_NET: &str = "main";

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
  #[structopt(name = "listen", long = "--listen", default_value = DEFAULT_LISTEN)]
  pub listen: String,
  #[structopt(name = "wallet", long = "--wallet", default_value = DEFAULT_WALLET)]
  pub wallet: String,
  #[structopt(name = "database", long = "--database", default_value = DEFAULT_DATABASE)]
  pub database: String,
  #[structopt(name = "net", long = "--net", default_value = DEFAULT_NET)]
  pub net: String,
  #[structopt(name = "peers", long = "--peers")]
  pub peers: Vec<String>,
}
