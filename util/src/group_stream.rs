use async_std::prelude::*;
use async_std::stream::Stream;
use std::collections::HashMap;
use std::hash::Hash;

pub fn group<S, K, F>(stream: S, group_num: i32, get_index: F) -> impl Stream<Item = S::Item>
where
    S: Stream,
    K: Eq + Hash,
    F: 'static + Fn(&S::Item) -> K,
{
    let ret = stream
        .scan(HashMap::default() as HashMap<K, i32>, move |state, item| {
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
        })
        .filter_map(|some: Option<_>| some);
    return ret;
}
