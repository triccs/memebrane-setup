use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};

use crate::msgs::ExecuteMsg;

/// AuctionContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AuctionContract(pub Addr);

impl AuctionContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }
}
