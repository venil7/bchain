use crate::{block::Block, tx::Tx};
use async_trait::async_trait;
use bchain_util::{mine::Mine, result::AppResult};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct TxPool {
  pool: HashSet<Tx>,
}

#[async_trait]
impl Mine for TxPool {
  async fn mine(&mut self, _difficulty: usize) -> AppResult<()> {
    Ok(())
  }
}

impl TxPool {
  pub fn add(&mut self, tx: Tx) -> AppResult<()> {
    self.pool.insert(tx);
    Ok(())
  }

  pub fn proposed_block(&self) -> Block {
    Block::default()
  }
}
