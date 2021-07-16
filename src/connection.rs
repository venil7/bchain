use crate::error::AppError;
use crate::protocol::Frame;
use crate::protocol::ParseError;
use bytes::Buf;
use bytes::BytesMut;
use std::io::Cursor;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub struct Connection {
  pub address: SocketAddr,
  stream: TcpStream,
  buffer: BytesMut,
}

impl Connection {
  pub fn new(stream: TcpStream, address: SocketAddr) -> Connection {
    Connection {
      address,
      stream,
      buffer: BytesMut::with_capacity(4_096),
    }
  }

  pub async fn read_frame(&mut self) -> Result<Option<Frame>, AppError> {
    loop {
      if let Some(frame) = self.parse_frame()? {
        return Ok(Some(frame));
      }
      if 0 == self.stream.read_buf(&mut self.buffer).await? {
        if self.buffer.is_empty() {
          return Ok(None);
        } else {
          return Err(AppError::new("connection reset by peer"));
        };
      }
    }
  }

  pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), AppError> {
    frame.write(&mut self.stream).await?;
    Ok(())
  }

  pub fn parse_frame(&mut self) -> Result<Option<Frame>, AppError> {
    let mut buf = Cursor::new(&self.buffer[..]);
    match Frame::check(&mut buf) {
      Ok(_) => {
        let len = buf.position() as usize;
        buf.set_position(0);
        let frame = Frame::parse(&mut buf)?;
        self.buffer.advance(len);
        Ok(Some(frame))
      }
      Err(ParseError::Incomplete) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }
}
