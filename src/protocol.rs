use crate::error::AppError;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt};

const HEADER: u8 = b'{';
const FOOTER: u8 = b'}';

#[derive(Debug)]
pub enum ParseError {
  Incomplete,
  Other(String),
}
impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    // let _ = debug!("{}", self);
    write!(f, "{:?}", self)
  }
}
impl Error for ParseError {}

impl From<ParseError> for AppError {
  fn from(pe: ParseError) -> Self {
    AppError::new(&format!("{}", pe))
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Blockchain {
  A,
  B,
  C,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Frame {
  Gossip,
  Blockchain(Blockchain),
}

impl Frame {
  pub async fn write<W>(&self, stream: &mut W) -> Result<(), Box<dyn Error>>
  where
    W: AsyncWrite + std::marker::Unpin,
  {
    let body = serde_cbor::to_vec(self)?;
    stream.write_u8(HEADER).await?;
    stream.write_i64(body.len() as i64).await?;
    stream.write_all(&body).await?;
    stream.write_u8(FOOTER).await?;
    stream.flush().await?;
    Ok(())
  }
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
  let closing_tag = buf.get_u8();
  if closing_tag != FOOTER {
    return Err(ParseError::Other(format!(
      "protocol error: expecting }}, got {:?}",
      closing_tag
    )));
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
  src.set_position(start as u64 + num);
  Ok(&src.get_ref()[start..start + num as usize])
}

impl Frame {
  pub fn check(buf: &mut Cursor<&[u8]>) -> Result<(), ParseError> {
    match get_u8(buf)? {
      HEADER => {
        let len = get_u64(buf)?;
        get_num_bytes(buf, len)?;
        get_footer(buf)?;
        Ok(())
      }
      ch => Err(ParseError::Other(format!(
        "protocol error: expecting {{, got {:?}",
        ch
      ))),
    }
  }

  pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, Box<dyn Error>> {
    match get_u8(buf)? {
      HEADER => {
        let len = get_u64(buf)?;
        let bytes = get_num_bytes(buf, len)?;
        get_footer(buf)?;
        let frame = serde_cbor::from_slice(bytes)?;
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
    Ok(assert_eq!(result.len(), 2))
  }
  #[test]
  fn test_get_num_bytes_10() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut cur: Cursor<&[u8]> = Cursor::new(&data);
    let result = get_num_bytes(&mut cur, 10)?;
    Ok(assert_eq!(result.len(), 10))
  }

  #[test]
  fn test_get_num_bytes_11() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut cur: Cursor<&[u8]> = Cursor::new(&data);
    let result = get_num_bytes(&mut cur, 11);
    Ok(assert_eq!(result.is_err(), true))
  }

  #[tokio::test]
  async fn test_write_frame() -> Result<(), Box<dyn Error>> {
    let mut vec = vec![0; 64];
    let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
    let frame = Frame::Gossip;
    Ok(frame.write(&mut cur).await?)
  }

  #[tokio::test]
  async fn test_frame_check() -> Result<(), Box<dyn Error>> {
    let mut vec = vec![0; 64];
    let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
    let frame = Frame::Blockchain(Blockchain::A);
    frame.write(&mut cur).await?;
    //
    let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec);
    Ok(Frame::check(&mut cur)?)
  }
  #[tokio::test]
  async fn test_frame_parse() -> Result<(), Box<dyn Error>> {
    let mut vec = vec![0; 64];
    let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
    let frame = Frame::Blockchain(Blockchain::A);
    frame.write(&mut cur).await?;
    //
    let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec);
    assert_eq!(Frame::parse(&mut cur)?, frame);

    Ok(())
  }
}
