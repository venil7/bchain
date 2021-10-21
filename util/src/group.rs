use async_std::prelude::*;
use async_std::stream::Stream;
use std::collections::HashMap;
use std::hash::Hash;

pub fn group<S, K, F>(stream: S, group_num: usize, get_index: F) -> impl Stream<Item = S::Item>
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

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use async_std::stream;

  #[async_std::test]
  async fn group_test_1() -> Result<()> {
    let iterable = vec![1, 1, 2, 3, 1, 2, 2, 3, 3];
    let stream = stream::from_iter(iterable);
    let recv1 = group(stream, 3, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1, 2, 3]);
    Ok(())
  }

  #[async_std::test]
  async fn group_test_2() -> Result<()> {
    let iterable = vec![1, 2, 3];
    let stream = stream::from_iter(iterable);
    let recv1 = group(stream, 1, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1, 2, 3]);
    Ok(())
  }

  #[async_std::test]
  async fn group_test_3() -> Result<()> {
    let iterable = vec![1];
    let stream = stream::from_iter(iterable);
    let recv1 = group(stream, 1, |&item| item);
    let res = recv1.collect::<Vec<_>>().await;

    assert_eq!(res, vec![1]);
    Ok(())
  }
}
