use once_cell::sync::Lazy;
use std::env::var;
use structopt::StructOpt;

static DEFAULT_DATABASE: Lazy<String> =
  Lazy::new(|| var("DATABASE").unwrap_or_else(|_| "chain.sqlite".into()));
static DEFAULT_NET: Lazy<String> =
  Lazy::new(|| var("NET").unwrap_or_else(|_| "chain.sqlite".into()));
static DEFAULT_WALLET: Lazy<String> =
  Lazy::new(|| var("WALLET").unwrap_or_else(|_| "pem/rsakey.pem".into()));
static DEFAULT_LISTEN: Lazy<String> =
  Lazy::new(|| var("LISTEN").unwrap_or_else(|_| "/ip4/0.0.0.0/tcp/0".into()));

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
  #[structopt(name = "listen", long = "--listen", default_value = &DEFAULT_LISTEN)]
  pub listen: String,
  #[structopt(name = "wallet", long = "--wallet", default_value = &DEFAULT_WALLET)]
  pub wallet: String,
  #[structopt(name = "database", long = "--db", default_value = &DEFAULT_DATABASE)]
  pub database: String,
  #[structopt(name = "net", long = "--net", default_value = &DEFAULT_NET)]
  pub net: String,
  #[structopt(name = "peers", long = "--peers")]
  pub peers: Vec<String>,
  #[structopt(name = "delay", long = "--delay", default_value = "1")]
  pub delay: usize,
  #[structopt(name = "init", long = "--init")]
  pub init: bool,
}

impl Cli {
  pub fn delay(&self) -> usize {
    if self.delay > 10 {
      10
    } else {
      self.delay
    }
  }
}
