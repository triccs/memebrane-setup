use cosmwasm_std::{ entry_point, DepsMut, Env, MessageInfo, Response };
use cw2::set_contract_version;

use crate::msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::error::ContractError;

// Contract name and version used for migration.
const CONTRACT_NAME: &str = "LP_gauge";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract_address", env.contract.address)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateLPGauge { msg } => Ok(Response::new().add_message(msg).add_attribute("creator", info.clone().sender.to_string())),
        }
}