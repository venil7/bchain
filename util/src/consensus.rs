use crate::group::group;
use async_std::stream::Stream;
use bchain_domain::hash_digest::Hashable;

pub fn collate<S, G>(stream: S, group_num: i32) -> impl Stream<Item = S::Item>
where
  S: Stream<Item = G>,
  G: Hashable,
{
  group(stream, group_num, |g| g.hash_digest())
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use async_std::prelude::*;
  use async_std::stream;

  #[async_std::test]
  async fn to_raw_and_back() -> Result<()> {
    let iterable = vec!["1", "1", "2", "3", "1", "2", "2", "3", "3"];
    let stream = stream::from_iter(iterable);
    let recv1 = collate(stream, 3);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec!["1", "2", "3"]);
    Ok(())
  }
}
