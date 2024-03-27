use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

/// ## Description
/// This enum describes Assembly contract errors!
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val}")]
    CustomError { val: String },

    #[error("Invalid Asset: {asset}")]
    InvalidAsset { asset: String },

    #[error("Insufficient Funds: {asset} - {amount")]
    InsufficientFunds { asset: String, amount: Uint128 },
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}
