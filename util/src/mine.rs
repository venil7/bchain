use crate::hash_digest::Hashable;
use async_trait::async_trait;

#[async_trait]
pub trait Mine: Hashable {
  async fn mine(&mut self, difficulty: usize);
}
