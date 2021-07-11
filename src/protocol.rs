use crate::error::AppError;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Cursor;

#[derive(Debug)]
pub enum ParseError {
  Incomplete,
  Other(String),
}
impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "parse-error")
  }
}
impl Error for ParseError {}

// impl From<Box<dyn Error>> for ParseError {
//   fn from(err: Box<dyn Error>) -> Self {
//     ParseError::Other(format!("{:?}", err))
//   }
// }

impl From<ParseError> for AppError {
  fn from(pe: ParseError) -> Self {
    AppError::new(&format!("{}", pe))
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Blockchain {
  A,
  B,
  C,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Frame {
  Gossip,
  Blockchain(Blockchain),
}

impl Default for Frame {
  fn default() -> Self {
    Frame::Gossip
  }
}

fn get_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, ParseError> {
  if !buf.has_remaining() {
    return Err(ParseError::Incomplete);
  }

  Ok(buf.get_u8())
}

fn get_footer(buf: &mut Cursor<&[u8]>) -> Result<(), ParseError> {
  if !buf.has_remaining() {
    return Err(ParseError::Incomplete);
  }
  if buf.get_ref()[0] != b'}' {
    return Err(ParseError::Other("protocol error".into()));
  }
  Ok(())
}

fn get_u64(buf: &mut Cursor<&[u8]>) -> Result<u64, ParseError> {
  if !buf.has_remaining() {
    return Err(ParseError::Incomplete);
  }

  Ok(buf.get_u64())
}

fn get_num_bytes<'a>(src: &mut Cursor<&'a [u8]>, num: u64) -> Result<&'a [u8], ParseError> {
  let start = src.position() as usize;
  let end = src.get_ref().len();
  if (num as usize) > (end - start) {
    return Err(ParseError::Incomplete);
  }

  Ok(&src.get_ref()[start..start + num as usize])
}

impl Frame {
  pub fn check(buf: &mut Cursor<&[u8]>) -> Result<(), ParseError> {
    match get_u8(buf)? {
      b'{' => {
        let len = get_u64(buf)?;
        get_num_bytes(buf, len)?;
        get_footer(buf)?;
        Ok(())
      }
      _ => Err(ParseError::Other("protocol error".into())),
    }
  }

  pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, Box<dyn Error>> {
    match get_u8(buf)? {
      b'{' => {
        let len = get_u64(buf)?;
        let bytes = get_num_bytes(buf, len)?;
        let frame = serde_cbor::from_slice(bytes)?;
        get_footer(buf)?;
        Ok(frame)
      }
      _ => Err(Box::new(AppError::new("protocol error"))),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_num_bytes_2() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut cur: Cursor<&[u8]> = Cursor::new(&data);
    let result = get_num_bytes(&mut cur, 2)?;
    assert_eq!(result.len(), 2);
    Ok(())
  }
  #[test]
  fn test_get_num_bytes_10() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut cur: Cursor<&[u8]> = Cursor::new(&data);
    let result = get_num_bytes(&mut cur, 10)?;
    assert_eq!(result.len(), 10);
    Ok(())
  }

  #[test]
  fn test_get_num_bytes_11() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut cur: Cursor<&[u8]> = Cursor::new(&data);
    let result = get_num_bytes(&mut cur, 11);
    assert_eq!(result.is_err(), true);
    Ok(())
  }
}
