use crate::protocol::error::ParseError;
use crate::result::AppResult;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt};

const HEADER: u8 = b'{';
const FOOTER: u8 = b'}';

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
    pub async fn write<W>(&self, stream: &mut W) -> AppResult<()>
    where
        W: AsyncWrite + std::marker::Unpin,
    {
        let body = serde_cbor::to_vec(self)?;
        stream.write_u8(HEADER).await?;
        stream.write_u64(body.len() as u64).await?;
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
            "Expecting {} but got {:?}",
            FOOTER as char, closing_tag as char
        )));
    }
    Ok(())
}

fn get_u64(buf: &mut Cursor<&[u8]>) -> Result<u64, ParseError> {
    if !buf.has_remaining() {
        return Err(ParseError::Incomplete);
    }

    let start = buf.position() as usize;
    let end = buf.get_ref().len();
    if (end - start) < 8 {
        return Err(ParseError::Incomplete);
    }

    Ok(buf.get_u64())
}

fn get_num_bytes<'a>(buf: &mut Cursor<&'a [u8]>, num: u64) -> Result<&'a [u8], ParseError> {
    if !buf.has_remaining() {
        return Err(ParseError::Incomplete);
    }
    let start = buf.position() as usize;
    let end = buf.get_ref().len();
    if (num as usize) > (end - start) {
        return Err(ParseError::Incomplete);
    }
    buf.set_position(start as u64 + num);
    Ok(&buf.get_ref()[start..start + num as usize])
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
                "Expecting {} but got {:?}",
                HEADER as char, ch as char
            ))),
        }
    }

    pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, ParseError> {
        match get_u8(buf)? {
            HEADER => {
                let len = get_u64(buf)?;
                let bytes = get_num_bytes(buf, len)?;
                get_footer(buf)?;
                let frame = serde_cbor::from_slice(bytes)?;
                Ok(frame)
            }
            ch => Err(ParseError::Other(format!(
                "Expecting {} but got {}",
                HEADER as char, ch as char
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_num_bytes_2() -> AppResult<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut cur: Cursor<&[u8]> = Cursor::new(&data);
        let result = get_num_bytes(&mut cur, 2)?;
        Ok(assert_eq!(result.len(), 2))
    }
    #[test]
    fn test_get_num_bytes_10() -> AppResult<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut cur: Cursor<&[u8]> = Cursor::new(&data);
        let result = get_num_bytes(&mut cur, 10)?;
        Ok(assert_eq!(result.len(), 10))
    }

    #[test]
    fn test_get_num_bytes_11() -> AppResult<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut cur: Cursor<&[u8]> = Cursor::new(&data);
        let result = get_num_bytes(&mut cur, 11);
        Ok(assert_eq!(result.is_err(), true))
    }

    #[tokio::test]
    async fn test_write_frame() -> AppResult<()> {
        let mut vec = vec![0; 64];
        let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
        let frame = Frame::Gossip;
        Ok(frame.write(&mut cur).await?)
    }

    #[tokio::test]
    async fn test_frame_check() -> AppResult<()> {
        let mut vec = vec![0; 64];
        let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
        let frame = Frame::Blockchain(Blockchain::A);
        frame.write(&mut cur).await?;
        //
        let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec);
        Ok(Frame::check(&mut cur)?)
    }

    #[tokio::test]
    async fn test_frame_parse() -> AppResult<()> {
        let mut vec = vec![0; 64];
        let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
        let frame = Frame::Blockchain(Blockchain::A);
        frame.write(&mut cur).await?;
        //
        let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec);
        assert_eq!(Frame::parse(&mut cur)?, frame);

        Ok(())
    }

    #[tokio::test]
    async fn test_frame_parse_2() -> AppResult<()> {
        let mut vec = vec![0; 64];
        let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec);
        let frame1 = Frame::Blockchain(Blockchain::A);
        let frame2 = Frame::Gossip;
        frame1.write(&mut cur).await?;
        frame2.write(&mut cur).await?;
        //
        let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec);
        assert_eq!(Frame::parse(&mut cur)?, frame1);
        assert_eq!(Frame::parse(&mut cur)?, frame2);

        Ok(())
    }

    #[tokio::test]
    async fn test_frame_check_incomplete() -> AppResult<()> {
        let mut vec1 = vec![0; 64];
        let frame1 = Frame::Blockchain(Blockchain::A);
        let mut cur: Cursor<&mut [u8]> = Cursor::new(&mut vec1);
        frame1.write(&mut cur).await?;
        //
        let mut cur: Cursor<&[u8]> = Cursor::new(&mut vec1[0..3]);
        println!("{:?}", cur);
        let res = Frame::parse(&mut cur).unwrap_err();
        assert_eq!(res, ParseError::Incomplete);

        Ok(())
    }
}
