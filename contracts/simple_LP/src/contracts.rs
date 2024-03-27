use cosmwasm_std::{
    entry_point, has_coins, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, 
    QueryRequest, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg, WasmQuery
};
use cw2::set_contract_version;

use crate::msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::reply::handle_balancer_reply;
use crate::state::{CONFIG, OWNERSHIP_TRANSFER};

// Contract name and version used for migration.
const CONTRACT_NAME: &str = "simple_LP";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//CONSTANTS
const BALANCER_POOL_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        paired_asset: msg.paired_asset,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("method", "instantiate")
        .add_attribute("config", format!("{:?}", config))
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
        ExecuteMsg::LP {} => LP(deps, env, info),
        ExecuteMsg::UpdateConfig { owner, paired_asset } => update_config(deps, info, owner, paired_asset),
        }
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<Addr>,
    paired_asset: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    //Assert Authority
    if info.sender != config.owner {
        //Check if ownership transfer is in progress & transfer if so
        let new_owner = OWNERSHIP_TRANSFER.load(deps.storage)?;
        if info.sender == new_owner {
            config.owner = info.sender;
        } else {
            return Err(ContractError::Unauthorized { });
        }
    }    
    if let Some(owner) = owner {
        let valid_addr = deps.api.addr_validate(&owner)?;

        //Set owner transfer state
        OWNERSHIP_TRANSFER.save(deps.storage, &valid_addr)?; 
        attrs.push(attr("owner_transfer", valid_addr));
    }

    if let Some(paired_asset) = paired_asset {
        config.paired_asset = paired_asset;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("config", format!("{:?}", config))
    )
}

fn LP(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    //Assert the contract has the required funds
    let paired_asset_balance = match deps.querier.query_balance(&env.contract.address, &config.paired_asset){
        Ok(balance) => {
            if balance.amount.is_zero() {
                return Err(ContractError::InsufficientFunds { asset: config.clone().paired_asset, amount: Uint128::zero() }),
            }

            balance.amount
        },
        Err(_) => return Err(ContractError::InsufficientFunds { asset: config.clone().paired_asset, amount: Uint128::zero() }),
    };
    let uosmo_balance = match deps.querier.query_balance(&env.contract.address, &String::from("uosmo")){
        Ok(balance) => {
            if balance.amount.is_zero() {
                return Err(ContractError::InsufficientFunds { asset: String::from("uosmo"), amount: Uint128::zero() }),
            }
            
            balance.amount
        },
        Err(_) => return Err(ContractError::InsufficientFunds { asset: String::from("uosmo"), amount: Uint128::zero() }),
    };
    //Needs 100 USDC to pay for the LP
    match deps.querier.query_balance(&env.contract.address, &String::from("ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4")){
        Ok(balance) => {
            if balance.amount < Uint128::from(100_000_000u128) {
                return Err(ContractError::InsufficientFunds { asset: String::from("usdc"), amount: Uint128::new(balance.amount) }),
            }
        },
        Err(_) => return Err(ContractError::InsufficientFunds { asset: String::from("usdc"), amount: Uint128::zero() }),
    };

    //Create the LP
    let msg = MsgCreateBalancerPool {
        sender: env.contract.address.to_string(),
        pool_params: Some(PoolParams {
            swap_fee: String::from("0.000000000000000000"), //0% in sdk.Dec 18 places
            exit_fee: String::from("0"),
            smooth_weight_change_params: None,
        }),
        pool_assets: vec![
            PoolAsset { 
                token: Some(Coin { denom: config.clone().paired_asset, amount: paired_asset_balance }), 
                weight: String::from("50") 
            },
            PoolAsset { 
                token: Some(Coin { denom: config.clone().osmo_denom, amount: uosmo_balance }), 
                weight: String::from("50") 
            }
        ],
        future_pool_governor: config.clone().owner,
    };
    let sub_msg = SubMsg::reply_on_success(msg, BALANCER_POOL_REPLY_ID);

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("method", "create_LP")
    )
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        BALANCER_POOL_REPLY_ID => handle_balancer_reply(deps, env, msg),
        id => Err(StdError::generic_err(format!("invalid reply id: {}", id))),
    }
}