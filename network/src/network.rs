use crate::protocol::BchainRequest;
use async_std::channel::{Receiver, Sender};
use async_std::future::timeout;
use async_std::sync::{Mutex, RwLock};
use bchain_db::database::Db;
use bchain_domain::address::Address;
use bchain_domain::block::Block;
use bchain_domain::wallet::Wallet;
use bchain_util::group::group_default;
use bchain_util::hash_digest::Hashable;
use bchain_util::result::AppResult;
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
    let tx = wallet.new_coinbase_tx(1_000_000)?;
    Block::new(Some([tx]))
  };
  let mut db = db.lock().await;
  db.commit_as_genesis(&genesis)?;
  info!("Writing genesis block {}", genesis.hash_digest());
  Ok(())
}

pub(crate) async fn request_latest_block(
  (_, majority): &NumPeersConsensus,
  network_requests: Sender<BchainRequest>,
  network_latest: Receiver<Block>,
) -> AppResult<Option<Block>> {
  network_requests.send(BchainRequest::AskLatest).await?;
  let mut network_latest_block_stream = group_default(network_latest, *majority);
  let network_latest_block = timeout(TIMEOUT, network_latest_block_stream.next());
  Ok(network_latest_block.await?)
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

pub(crate) async fn local_balance(address: &Address, db: Arc<Mutex<Db>>) -> AppResult<i64> {
  let mut id = 0;
  let mut balance = 0;
  loop {
    let block = db.lock().await.get_block(id)?;
    if let Some(block) = block {
      balance += block.diff_for_address(address);
      id += 1;
    } else {
      break;
    }
  }
  Ok(balance)
}
