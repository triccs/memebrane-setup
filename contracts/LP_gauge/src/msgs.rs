use cosmwasm_schema::cw_serde;

use osmosis_std::types::osmosis::incentives::MsgCreateGauge;


#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateLPGauge { msg: MsgCreateGauge },
    // UpdateConfig {
    // },
}

// #[cw_serde]
// pub enum QueryMsg {
//     /// Return contract config
//     Config {},
// }

// #[cw_serde]
// pub struct Config {
//     /// Contract owner
//     pub owner: Addr,
//     /// Asset to pair with OSMO
//     pub paired_asset: String,
// }

#[cw_serde]
pub struct MigrateMsg {}