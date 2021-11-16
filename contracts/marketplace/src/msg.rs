use cosmwasm_std::{ Uint128 };
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw0::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract name 
    pub name: String,
    /// contract admin
    pub admin_addr: String,
    /// Stable coin denomination. 
    pub stable_denom: String,
    /// public key that can sign transaction messages
    pub public_key: String,
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
    /// Admin function: change admin address
    SetAdmin { new_addr: String },
    /// Admin function: change public key
    SetPublicKey { public_key: String },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the contract info for the Marketplace Contract
    ContractInfo {},
    /// Return the admin
    Admin {},
    /// Return the public key that is being used to validate messages with signatures
    PublicKey {},
}


/// CW721 Contract Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TokenMsg {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { 
        /// Burn Address (Fantasy Contract Address)
        recipient: String, 
        /// Token ID of the NFT to be transferred/burned
        token_id: String 
    },
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OwnerOfResponse {
    /// Owner of the token
    pub owner: String,
    /// If set this address is approved to transfer/send the token as well
    pub approvals: Vec<Approval>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: String,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}