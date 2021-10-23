use crate::protocol::BchainRequest;
use async_std::channel::{Receiver, Sender};
use async_std::future::timeout;
use async_std::sync::{Mutex, RwLock};
use bchain_db::database::Db;
use bchain_domain::block::Block;
use bchain_domain::hash_digest::Hashable;
use bchain_domain::{result::AppResult, wallet::Wallet};
use bchain_util::group::{group_by, group_default};
use futures::prelude::*;
use log::info;
use std::{sync::Arc, time::Duration};

pub type NumPeersConsensus = (usize, usize);

const TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) async fn bootstrap_init(
  wallet: Arc<RwLock<Wallet>>,
  db: Arc<Mutex<Db>>,
) -> AppResult<()> {
  let genesis = {
    let wallet = wallet.read().await;
    let tx = wallet.new_tx(wallet.public_key(), 1000)?;
    Block::new(Some([tx]))
  };
  let mut db = db.lock().await;
  db.commit_as_genesis(&genesis)?;
  info!("Writing genesis block {}", genesis.hash_digest());
  Ok(())
}

pub(crate) async fn request_latest_block_id(
  (_, majority): &NumPeersConsensus,
  network_requests: Sender<BchainRequest>,
  network_latest: Receiver<i64>,
) -> AppResult<Option<i64>> {
  network_requests.send(BchainRequest::AskLatest).await?;
  let mut network_latest_block_stream = group_by(network_latest, *majority, |&i| i);
  let network_latest_block_id = timeout(TIMEOUT, network_latest_block_stream.next());
  Ok(network_latest_block_id.await?)
}

pub(crate) async fn request_specific_block(
  id: i64,
  (_, majority): &NumPeersConsensus,
  network_requests: Sender<BchainRequest>,
  network_blocks: Receiver<Block>,
) -> AppResult<Option<Block>> {
  network_requests.send(BchainRequest::AskBlock(id)).await?;
  let network_block_stream =
    group_default(network_blocks, *majority).filter(|block| future::ready(block.id == id));
  let mut pinned_stream = Box::pin(network_block_stream);
  let next = pinned_stream.next();
  let network_block = timeout(TIMEOUT, next);
  Ok(network_block.await?)
}
