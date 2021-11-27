use async_std::channel::{Receiver, Sender};
use async_std::sync::{Mutex, RwLock};
use bchain_db::database::Db;
use bchain_domain::block::Block;
use bchain_domain::tx::Tx;
use bchain_domain::tx_pool::TxPool;
use bchain_domain::wallet::Wallet;
use bchain_util::hash_digest::Hashable;
use bchain_util::result::AppResult;
use futures::{prelude::*, select};
use log::{info, warn};
use std::sync::Arc;

use crate::protocol::BchainResponse;

pub(crate) async fn mine(
  _wallet: Arc<RwLock<Wallet>>,
  _db: Arc<Mutex<Db>>,
  _pool: Arc<Mutex<TxPool>>,
  mut proposed_tx: Receiver<Tx>,
  mut proposed_blocks: Receiver<Block>,
  _response: Sender<BchainResponse>,
) -> AppResult<()> {
  loop {
    select! {
      tx = proposed_tx.select_next_some().fuse() => {
          info!("miner received tx {:?}", tx.hash_digest());
      },
      block = proposed_blocks.select_next_some().fuse() => {
        info!("miner received block {:?}", block.hash_digest());
      },
      complete => break,
    }
  }
  warn!("mine loop exited");
  Ok(())
}
