use crate::schema::generated::blocks;
use chrono::NaiveDateTime;

#[derive(Queryable, Debug)]
pub struct Block {
  pub id: i32,
  pub transactions: Vec<u8>,
  pub created: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "blocks"]
pub struct NewBlock {
  pub transactions: Vec<u8>,
  pub created: NaiveDateTime,
}
