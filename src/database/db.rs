use crate::result::AppResult;
use async_std::task::spawn;
use diesel::prelude::*;
use diesel::SqliteConnection;
use std::io::{Error as IoError, ErrorKind};

struct Db {
  connection: diesel::SqliteConnection,
}

impl Db {
  pub async fn new(path: &'static str) -> AppResult<Self> {
    let connection = spawn(async move {
      SqliteConnection::establish(path)
        .or_else(|e| Err(IoError::new(ErrorKind::InvalidData, format!("{:?}", e))))
    })
    .await?;
    Ok(Db { connection })
  }
}
