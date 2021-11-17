use cosmwasm_std::{ StdError };
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("You need to send exactly {}{} to purchase a pack", amount, denom)]
    WrongAmount { amount: u64, denom: String },

    #[error("The current round has already been used. Please wait for the next round")]
    UsedRound {},

    #[error("Minting cannot exceed the cap")]
    Capped {},

    #[error("Token cannot be unlocked at this time")]
    Locked {},

    #[error("Token has reached its maximum number of usages")]
    UsageCapped {},

    #[error("Something went wrong")]
    DefaultErr {},

    #[error("Athlete ID does not exist")]
    DoesNotExist {},
}
