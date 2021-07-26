use cosmwasm_std::Binary;
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,

    // Maximum number of base tokens
    pub base_cap: u64,
    // Maximum number of silver tokens
    pub silver_cap: u64,
    // Maximum number of gold tokens
    pub gold_cap: u64,
}

/// This is like Cw721ExecuteMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Converts Base NFTs to Silver Rank
    UpgradeToken {
        /// Desired rank to upgrade to
        rank: String,
        /// NFTs to burn
        tokens: Vec<String>,
    },
    /// Change the minter for the token, can only be called by the current minter
    UpdateMinter {
        /// Address of the new minter
        minter: String,
    },

    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg),

    /// Locks an NFT token to be played for Fantasy Sports, can only be called by the NFT owner
    LockToken {
        /// Unique ID of the NFT
        token_id: String,
    },

    /// Checks and unlocks an NFT token if it can be unlocked, can only be called by the NFT owner 
    UnlockToken {
        /// Unique ID of the NFT
        token_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Describes the rank of the NFT 
    pub rank: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Return the owner of the given token, error if token does not exist
    /// Return type: OwnerOfResponse
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    /// Return type: `ApprovedForAllResponse`
    ApprovedForAll {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract: `ContractInfoResponse`
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: `NftInfoResponse`
    NftInfo {
        token_id: String,
    },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients: `AllNftInfo`
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// Total number of tokens issued
    BaseTokens {},
    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: TokensResponse.
    OwnerBaseTokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    /// Return type: TokensResponse.
    AllBaseTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    SilverTokens {},
    OwnerSilverTokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllSilverTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    GoldTokens {}, 
    OwnerGoldTokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllGoldTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    // Return the minter
    Minter {},
    /// Returns a boolean determining if the token is mintable
    IsMintable {
        rank: String,
    },
    /// Checks if a locked NFT can be unlocked
    CanUnlockToken {
        token_id: String,
    },
}

/// Shows who can mint these tokens
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterResponse {
    pub minter: String,
}
