use std::convert::TryInto;
use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Decimal, DepsMut, Env, WasmMsg, WasmQuery, attr,
    Response, StdResult, Uint128, Reply, StdError, CosmosMsg, SubMsg, coins, QueryRequest, BankMsg,
};

/// Send all contract assets to the burn address
pub fn handle_balancer_reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response>{
    match msg.clone().result.into_result() {
        Ok(result) => {
        let mut attrs = vec![];

        //Get Balancer Pool denom from Response
        if let Some(b) = result.data {
            let res: MsgCreateBalancerPoolResponse = match b.try_into().map_err(ContractError::Std){
                Ok(res) => res,
                Err(err) => return Err(StdError::GenericErr { msg: String::from(err.to_string()) })
            };
            
            attrs.push(attr("pool_id", format!("{:?}", res.pool_id)));
        }

        //Query all assets in the contract
        let assets = deps.querier.query_all_balances(&env.contract.address)?;
        //Send all assets to the burn address
        let msg = BankMsg::Send {
            to_address: String::from("osmo1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmcn030"),
            amount: vec![assets],
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Bank(msg))
            .add_attributes(attrs)
            .add_attribute("assets_burnt", format!("{:?}", assets))
        )
    },
        Err(err) => return Err(StdError::GenericErr { msg: err }),
    }    
}
