use crate::group::group;
use async_std::stream::Stream;
use bchain_domain::hash_digest::Hashable;
use num::Integer;

pub fn group_default<S, G>(stream: S, group_num: usize) -> impl Stream<Item = S::Item>
where
  S: Stream<Item = G>,
  G: Hashable,
{
  group(stream, group_num, |g| g.hash_digest())
}

pub fn peer_majority(peers: usize) -> usize {
  peers.div_ceil(&2)
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
    let recv1 = group_default(stream, 3);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec!["1", "2", "3"]);
    Ok(())
  }

  #[test]
  fn majority_test() -> Result<()> {
    let peers = 3;
    let majority = peer_majority(peers);

    assert_eq!(majority, 2);
    Ok(())
  }
}
