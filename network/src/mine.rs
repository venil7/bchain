use crate::protocol::{BchainRequest, BchainResponse};
use async_std::channel::{Receiver, Sender};
use async_std::stream::interval;
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
use std::time::Duration;

const EVERY_10_MINUTES: Duration = Duration::from_secs(60_10);

pub(crate) async fn mine(
  _wallet: Arc<RwLock<Wallet>>,
  _db: Arc<Mutex<Db>>,
  _pool: Arc<Mutex<TxPool>>,
  mut proposed_tx: Receiver<Tx>,
  mut proposed_blocks: Receiver<Block>,
  _bchain_request: Sender<BchainRequest>,
  bchain_response: Sender<BchainResponse>,
) -> AppResult<()> {
  let mut timer = interval(EVERY_10_MINUTES).fuse();
  loop {
    select! {
      _tick = timer.select_next_some() => {
        info!("mining cycle")
        // let proposed_block = {
        //   let mut pool = pool.lock().await;
        //     pool.mine(1).await?;
        //     pool.proposed_block()
        // };
        // bchain_request.send(BchainRequest::SubmitBlock(proposed_block)).await?;
      },
      tx = proposed_tx.select_next_some() => {
        let response = bchain_response.clone();
        handle_proposed_tx(tx, response).await?;
      },
      block = proposed_blocks.select_next_some() => {
        let response = bchain_response.clone();
        handle_proposed_block(block, response).await?;
      },
      complete => break,
    }
  }
  warn!("Mining loop exited");
  Ok(())
}

async fn handle_proposed_tx(tx: Tx, response: Sender<BchainResponse>) -> AppResult<()> {
  response
    .send(BchainResponse::AcceptTx(tx.hash_digest()))
    .await?;
  info!("miner received tx {:?}", tx.hash_digest());
  Ok(())
}

async fn handle_proposed_block(block: Block, response: Sender<BchainResponse>) -> AppResult<()> {
  response
    .send(BchainResponse::AcceptBlock(block.hash_digest()))
    .await?;
  info!("miner received block {:?}", block.hash_digest());
  Ok(())
}
