use bchain::cli::DEFAULT_LISTEN;
use bchain::connection::Connection;
use bchain::protocol::{Blockchain, Frame};
use log::info;
use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv::dotenv()?;
  env_logger::init();
  let stream = TcpStream::connect(DEFAULT_LISTEN).await?;
  let addr = stream.local_addr()?;
  let mut connection = Connection::new(stream, addr);
  info!("Connection established to {}", addr);
  connection
    .write_frame(&Frame::Blockchain(Blockchain::A))
    .await?;
  let response_frame = connection.read_frame().await?;
  info!("received back {:?}", response_frame);
  Ok(())
}
