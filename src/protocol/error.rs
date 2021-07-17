use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
  Incomplete,
  Other(String),
}
impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "{:?}", self)
  }
}

impl Error for ParseError {}

impl From<serde_cbor::Error> for ParseError {
  fn from(err: serde_cbor::Error) -> Self {
    ParseError::Other(format!("{}", err))
  }
}
