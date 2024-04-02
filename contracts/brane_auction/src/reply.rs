use cosmwasm_std::{DepsMut, Env, Reply, Response, StdError, StdResult};

use crate::state::CONFIG;



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

            //Load config
            let mut config = CONFIG.load(deps.storage)?;
            //Update mint address
            config.minter_addr = sg721_addr.clone();
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