use cosmwasm_std::{Addr, Decimal, Uint128};
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct InstantiateMsg {
    pub paired_asset: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    LP { },
    UpdateConfig {
        paired_asset: Option<String>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    /// Return contract config
    Config {},
}

#[cw_serde]
pub struct Config {
    /// Contract owner
    pub owner: Addr,
    /// Asset to pair with OSMO
    pub paired_asset: String,
}

#[cw_serde]
pub struct MigrateMsg {}