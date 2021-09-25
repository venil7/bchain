use libp2p::gossipsub::error::PublishError;
use std::error::Error;

pub type DynError = dyn Error + Send + Sync + 'static;

#[derive(Debug, PartialEq, Clone)]
pub struct AppError(String);
impl std::error::Error for AppError {}
impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", self.0)
  }
}

impl From<Box<DynError>> for AppError {
  fn from(err: Box<DynError>) -> Self {
    AppError(format!("{}", err))
  }
}

impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<serde_json::Error> for AppError {
  fn from(err: serde_json::Error) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<pkcs8::Error> for AppError {
  fn from(err: pkcs8::Error) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<hex::FromHexError> for AppError {
  fn from(err: hex::FromHexError) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<base58::FromBase58Error> for AppError {
  fn from(e: base58::FromBase58Error) -> Self {
    AppError(format!("{:?}", e))
  }
}
impl From<PublishError> for AppError {
  fn from(e: PublishError) -> Self {
    AppError(format!("{:?}", e))
  }
}

impl AppError {
  pub fn new(str: &str) -> Self {
    AppError(str.to_owned())
  }
}
