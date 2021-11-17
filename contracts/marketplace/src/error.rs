use cosmwasm_std::{ StdError, Uint128 };
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The provided message is invalid")]
    InvalidMessage {},

    #[error("You need to send exactly {}{} to purchase this token", amount, denom)]
    InsufficientFunds { amount: Uint128, denom: String },

    #[error("The token buyer address does not match sender address")]
    BuyerMismatch {},

    #[error("The token is not owned by the provided owner address")]
    InvalidToken {},

    #[error("Signature doesn't match")]
    BadSignature {},

    #[error("Invalid Secp256k1 Pubkey Format")]
    InvalidSecp256k1PubkeyFormat {},
}
