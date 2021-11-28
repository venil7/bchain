use crate::result::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait Mine {
  async fn mine(&mut self, difficulty: usize) -> AppResult<()>;
}
