use bchain_domain::address::Address;
use bchain_util::error::AppError;
use nom::branch::alt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use self::{
  balance::balance_command, blocks::blocks_command, bootstrap::bootstrap_command,
  dial::dial_command, message::message_command, peers::peers_command, tx::tx_command,
};

pub mod balance;
pub mod blocks;
pub mod bootstrap;
pub mod dial;
pub mod message;
pub mod peers;
pub mod tx;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserCommand {
  Peers,
  Blocks,
  Bootstrap,
  Msg(String),
  Unrecognized,
  Dial(Vec<String>),
  Balance(Option<Address>),
  Tx(Address, u64),
}

impl FromStr for UserCommand {
  type Err = AppError;

  fn from_str(msg: &str) -> Result<Self, Self::Err> {
    if let Ok((_, cmd)) = alt((
      tx_command,
      dial_command,
      peers_command,
      blocks_command,
      message_command,
      balance_command,
      bootstrap_command,
    ))(msg)
    {
      Ok(cmd)
    } else {
      Ok(UserCommand::Unrecognized)
    }
  }
}
