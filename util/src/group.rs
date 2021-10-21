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
        if let Some(&count) = state.get(&idx) {
          if count >= group_num - 1 {
            state.remove(&idx);
            Some(Some(item))
          } else {
            state.insert(idx, count + 1);
            Some(None)
          }
        } else {
          state.insert(idx, 1);
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
}
