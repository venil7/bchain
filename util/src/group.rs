use async_std::prelude::*;
use async_std::stream::Stream;
use bchain_domain::hash_digest::Hashable;
use num::Integer;
use std::collections::HashMap;
use std::hash::Hash;

pub fn group_by<S, K, F>(stream: S, group_num: usize, get_index: F) -> impl Stream<Item = S::Item>
where
  S: Stream,
  K: Eq + Hash,
  F: 'static + Fn(&S::Item) -> K,
{
  assert!(group_num > 0);
  let ret = stream
    .scan(
      HashMap::default() as HashMap<K, usize>,
      move |state, item| {
        let idx = get_index(&item);
        let val = match state.get(&idx) {
          Some(v) => v + 1,
          None => 1,
        };
        if val >= group_num {
          state.remove(&idx);
          Some(Some(item))
        } else {
          state.insert(idx, val);
          Some(None)
        }
      },
    )
    .filter_map(|some: Option<_>| some);
  return ret;
}

pub fn group_default<S, G>(stream: S, group_num: usize) -> impl Stream<Item = S::Item>
where
  S: Stream<Item = G>,
  G: Hashable,
{
  group_by(stream, group_num, |g| g.hash_digest())
}

pub fn peer_majority(peers: usize) -> usize {
  peers.div_ceil(&2)
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use async_std::stream;

  #[async_std::test]
  async fn group_test_1() -> Result<()> {
    let iterable = vec![1, 1, 2, 3, 1, 2, 2, 3, 3];
    let stream = stream::from_iter(iterable);
    let recv1 = group_by(stream, 3, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1, 2, 3]);
    Ok(())
  }

  #[async_std::test]
  async fn group_test_2() -> Result<()> {
    let iterable = vec![1, 2, 3];
    let stream = stream::from_iter(iterable);
    let recv1 = group_by(stream, 1, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1, 2, 3]);
    Ok(())
  }

  #[async_std::test]
  async fn group_test_3() -> Result<()> {
    let iterable = vec![1];
    let stream = stream::from_iter(iterable);
    let recv1 = group_by(stream, 1, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1]);
    Ok(())
  }

  #[async_std::test]
  async fn group_default_test_1() -> Result<()> {
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
