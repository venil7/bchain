use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
  #[structopt(name = "listen", long = "--listen", default_value = "0.0.0.0:5566")]
  pub listen: String,
  #[structopt(name = "bootstrap", long = "--bootstrap")]
  pub bootstrap: Vec<String>,
}
