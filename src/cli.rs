use structopt::StructOpt;

pub const DEFAULT_LISTEN: &str = "0.0.0.0:5566";

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
  #[structopt(name = "listen", long = "--listen", default_value = DEFAULT_LISTEN)]
  pub listen: String,
  #[structopt(name = "peer", long = "--peer")]
  pub peer: String,
}
