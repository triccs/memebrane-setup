use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use cosmwasm_std::{Addr, Coin};

use crate::msgs::Config;

pub const CONFIG: Item<Config> = Item::new("config");
pub const OWNERSHIP_TRANSFER: Item<Addr> = Item::new("ownership_transfer");