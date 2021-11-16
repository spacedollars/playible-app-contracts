use cosmwasm_std::{ StdError, Uint128 };
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("You need to send exactly {}{} to purchase this token", amount, denom)]
    WrongAmount { amount: Uint128, denom: String },

    #[error("The collection address is invalid")]
    InvalidCollection {},

    #[error("The token owner address is invalid")]
    InvalidOwner {},

    #[error("The token buyer address is invalid")]
    InvalidBuyer {},

    #[error("The token buyer address does not match sender address")]
    BuyerMismatch {},
}
