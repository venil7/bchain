use crate::database::block::NewBlock;
use crate::database::generated::blocks;
use crate::result::AppResult;
use diesel::prelude::*;
use diesel::SqliteConnection;

pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn raw_connection<'a>(self: &'a Self) -> AppResult<&'a SqliteConnection> {
        Ok(&self.connection)
    }

    pub fn new(path: &str) -> AppResult<Self> {
        let connection = SqliteConnection::establish(&path)?;
        Ok(Db { connection })
    }

    pub fn insert_block(&mut self, block: &NewBlock) -> AppResult<()> {
        diesel::insert_into(blocks::table)
            .values(block)
            .execute(&self.connection)?;
        Ok(())
    }
}
