use crate::error::AppError;
use crate::error::DynError;
use crate::network::connection::Connection;
use crate::protocol::frame::Frame;
use crate::result::AppResult;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

type FrameMessage = (Frame, oneshot::Sender<Frame>);

#[derive(Default)]
struct Peers(String);

pub struct FullNode {
    _peers: Arc<Mutex<Peers>>,
}

impl FullNode {
    fn new() -> Self {
        let _peers = Arc::new(Mutex::new(Peers::default()));
        FullNode { _peers }
    }

    pub async fn run(addr: SocketAddr) -> AppResult<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("listening on {}", addr);
        let (sender, mut receiver) = mpsc::channel::<FrameMessage>(32);

        tokio::spawn(async move {
            let mut node = FullNode::new();
            info!("ready to handle");
            node.handle_frames(&mut receiver).await?;
            Ok::<(), AppError>(())
        });

        loop {
            let (stream, address) = listener.accept().await?;
            let sender = sender.clone();
            tokio::spawn(async move {
                let mut connection = Connection::new(stream, address);
                if let Err(e) = FullNode::handle_connecton(&mut connection, sender).await {
                    error!("{}", e)
                }
            });
        }
    }

    async fn handle_frames(
        &mut self,
        receiver: &mut mpsc::Receiver<FrameMessage>,
    ) -> AppResult<()> {
        while let Some((frame, responder)) = receiver.recv().await {
            info!("received: {:?}", frame);
            responder.send(frame).unwrap();
        }
        Ok(())
    }

    async fn handle_connecton(
        connection: &mut Connection,
        frame_handling_queue: mpsc::Sender<FrameMessage>,
    ) -> AppResult<()> {
        info!("connection from {}", connection.address);
        while let Some(frame) = connection.read_frame().await? {
            let (tx, frame_handling_respond) = oneshot::channel();
            frame_handling_queue.send((frame, tx)).await?;
            let response_frame = frame_handling_respond.await?;
            connection.write_frame(&response_frame).await?;
        }
        Ok(())
    }
}
