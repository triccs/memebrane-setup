
use cosmwasm_std::{attr, to_json_binary, CosmosMsg, DepsMut, Env, QueryRequest, Reply, Response, StdError, StdResult, WasmMsg, WasmQuery};
use cw721::{TokensResponse, Cw721QueryMsg as Sg721QueryMsg};
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

            let mut attrs = vec![
                attr("sg721_addr", sg721_addr),
                attr("base_minter_addr", base_minter_addr),
            ];
            //Query sg721 contract for Tokens & AllTokens as a test
            if let Ok(res) = deps.querier.query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.clone().sg721_addr,
                msg: to_json_binary(&Sg721QueryMsg::AllTokens { start_after: None, limit: None })?,
            })) {
                attrs.push(attr("all_tokens", format!("{:?}", res)));
            }
            if let Ok(res) = deps.querier.query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.clone().sg721_addr,
                msg: to_json_binary(&Sg721QueryMsg::Tokens { owner: env.contract.address.to_string(), start_after: None, limit: None })?,
            })) {
                attrs.push(attr("contract_tokens", format!("{:?}", res)));
            }

            Ok(Response::new()
                .add_attribute("sg721_addr", config.clone().sg721_addr)
                .add_attribute("base_minter_addr", config.clone().minter_addr)
            )
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