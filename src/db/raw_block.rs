use crate::db::schema::blocks;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct RawBlock {
  pub id: i32,
  pub block: Vec<u8>,
  pub created: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "blocks"]
pub struct NewRawBlock {
  pub block: Vec<u8>,
  pub created: NaiveDateTime,
}
