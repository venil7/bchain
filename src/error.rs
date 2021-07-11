use std::error::Error;

#[derive(Debug, Clone)]
pub struct AppError(String);

impl std::error::Error for AppError {}
impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", self.0)
  }
}

impl From<Box<dyn Error>> for AppError {
  fn from(err: Box<dyn Error>) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self {
    AppError(format!("{}", err))
  }
}
impl From<serde_cbor::Error> for AppError {
  fn from(err: serde_cbor::Error) -> Self {
    AppError(format!("{}", err))
  }
}

impl AppError {
  pub fn new(str: &str) -> Self {
    AppError(str.to_owned())
  }
}
