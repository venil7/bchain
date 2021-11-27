use std::collections::HashSet;

use crate::tx::Tx;

#[derive(Debug, Default)]
pub struct TxPool {
  pool: HashSet<Tx>,
}
