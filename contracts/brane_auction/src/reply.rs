
use cosmwasm_std::{to_json_binary, CosmosMsg, DepsMut, Env, Reply, Response, StdError, StdResult, WasmMsg};
use sg721::ExecuteMsg as Sg721ExecuteMsg;

use crate::state::{CONFIG, WINNING_BIDDER};

pub fn handle_collection_reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result.into_result() {
        Ok(result) => {
            
            let mint_event = result
                .events
                .iter()
                .find(|e| {
                    e.attributes
                        .iter()
                        .any(|attr| attr.key == "sg721_address")
                })
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find mint event") }
                })?;

            let sg721_addr = mint_event
                .attributes
                .iter()
                .find(|attr| attr.key == "sg721_address")
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find mint address") }
                })?
                .value
                .clone();

            
            
                let base_minter_event = result
                .events
                .iter()
                .find(|e| {
                    e.attributes
                        .iter()
                        .any(|attr| attr.value == "crates.io:sg-base-minter")
                })
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find base minter event") }
                })?;

            let base_minter_addr = base_minter_event
                .attributes
                .iter()
                .find(|attr| attr.key == "_contract_address")
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find base minter address") }
                })?
                .value
                .clone();

            //Load config
            let mut config = CONFIG.load(deps.storage)?;
            //Update base_minter address
            config.minter_addr = base_minter_addr.clone();
            //Update mint address
            config.sg721_addr = sg721_addr.clone();
            //Save config
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new()
            .add_attribute("mint_addr", config.clone().minter_addr))
        },
        
        Err(err) => {
            //Its reply on success only
            Ok(Response::new().add_attribute("error", err))
        }
    }
}

    
pub fn handle_mint_reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result.into_result() {
        Ok(result) => {            
            
            let mint_event = result
                .events
                .iter()
                .find(|e| {
                    e.attributes
                        .iter()
                        .any(|attr| attr.key == "token_id")
                })
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find mint event") }
                })?;

            let token_id = mint_event
                .attributes
                .iter()
                .find(|attr| attr.key == "token_id")
                .ok_or_else(|| {
                    StdError::GenericErr { msg: String::from("unable to find token IDs") }
                })?
                .value
                .clone();

            //Load config
            let config = CONFIG.load(deps.storage)?;
            //Load winning bidder
            let winning_bidder = WINNING_BIDDER.load(deps.storage)?;
            //Remove winning bidder
            WINNING_BIDDER.remove(deps.storage);
            
            ///Transfer newly minted NFT to the bidder
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.clone().sg721_addr,
                msg: to_json_binary(&Sg721ExecuteMsg::TransferNft::<Option<String>, Option<String>> { 
                    recipient: winning_bidder.clone(),
                    token_id: token_id.clone(),
                    })?,
                funds: vec![],
            });

            Ok(Response::new()
            .add_message(msg)
            .add_attribute("token_id", token_id)
            .add_attribute("new_owner", winning_bidder)
            )
        },
        
        Err(err) => {
            //Its reply on success only
            Ok(Response::new().add_attribute("error", err))
        }
    }    
}