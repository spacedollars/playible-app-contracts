use cosmwasm_std::{ Uint128 };
use cw20::{Cw20ReceiveMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract name 
    pub name: String,
    /// contract admin
    pub admin_addr: String,
    /// Stable coin denomination. 
    pub stable_denom: String,
}  


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Mock function for validating a signed message from front-end
    TempTransaction {
        contract_addr: String,
        owner_addr: String,
        token_id: String,
        buyer_addr: String,
        price: Uint128
    }, 
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the contract info for the Marketplace Contract
    ContractInfo {},
    /// Mock function for querying if a signed message is valid
    TempIsValid {
        contract_addr: String,
        owner_addr: String,
        token_id: String,
        buyer_addr: String,
        price: Uint128
    },
}


/// CW721 Contract Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TokenMsg {
    TransferNft { 
        /// Burn Address (Fantasy Contract Address)
        recipient: String, 
        /// Token ID of the NFT to be transferred/burned
        token_id: String 
    },
    NftInfo {
        /// Token ID of the NFT to be queried
        token_id: String,
    },
}