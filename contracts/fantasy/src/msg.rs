use cosmwasm_std::{Binary, CanonicalAddr, Uint128, Timestamp};
use cw20::{Cw20ReceiveMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Uint256, Decimal256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Stable coin denomination. 
    pub stable_denom: String,
    // anchor contract address for depositing the rewards
    pub anchor_addr: String,
    // terrand contract address for calling Oracle's DRand
    pub terrand_addr: String,
    // Number of Player NFTs to be pulled per pack
    pub pack_len: u64,
    // Price of each pack
    pub pack_price: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct TokenExtension {
    /// Determines whether or not the NFT is locked for Fantasy Sports
    pub is_locked: bool,
    /// Determines the unlock date after the NFT has been locked
    pub unlock_date: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftInfoResponse {
    /// Universal Resource Identifier link of the NFT
    pub token_uri: Option<String>,
    /// Describes the rarity of the NFT 
    pub rarity: String,
    /// Additional Metadata of Fantasy Athlete tokens
    pub extension: TokenExtension,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// For testing stuff
    Test {}, 
    /// Purchase an athlete token pack
    PurchasePack {},
    /// Deposit Stablecoins into the contract to receive an athlete token
    DepositStable {},
    /// Redeem Stablecoins (UST) from Anchor
    RedeemStable {
        //amount in uusd to be redeemed from Anchor
        amount: Uint128,
    },
    /// Add athlete token contract address
    AddToken {
        token: String,
    },
    /// Performs the turnover of tokens to another instance of Fantasy contract
    TokenTurnover {
        new_contract: String
    },
    /// Locks an NFT token to be played for Fantasy Sports, can only be called by the NFT owner
    LockToken {
        /// Athlete ID of the NFT
        athlete_id: String,
        /// Unique ID of the NFT
        token_id: String,
        /// Time before a token can be unlocked
        duration: String,
    },
    /// Checks and unlocks an NFT token if it can be unlocked, can only be called by the NFT owner 
    UnlockToken {
        /// Athlete ID of the NFT
        athlete_id: String,
        /// Unique ID of the NFT
        token_id: String,
    },
    /// Exchanges an Athlete token with the same rarity for a higher rarity token
    UpgradeSameToken {
        /// Describes the rarity of the NFT 
        rarity: String,
        /// Athlete ID of the NFTs to be burned
        athlete_id: String,
        /// NFTs to burn
        tokens: Vec<String>,
    },
    /// Exchanges any Athlete tokens of the same rarity for a random higher rarity token
    UpgradeRandToken {
        /// Describes the rarity of the NFT 
        rarity: String,
        /// Athlete IDs of the NFTs to be burned
        athlete_ids: Vec<String>,
        /// NFTs to burn
        tokens: Vec<String>,
        /// Seed to be used for minting a random new token
        rand_seed: String
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the contract info for the Fantasy Contract
    ContractInfo {},
    /// Returns the price for purchasing a pack
    PackPrice {},
    /// Returns the contract address of the corresponding token id
    TokenContract {
        athlete_id: String,
    },
    /// Returns the total deposited stable coin amount to Anchor
    TotalDeposit {},
    /// Returns a boolean if the token is mintable using the Athlete Contract's IsMintable{} Query
    IsTokenMintable {
        athlete_id: String,
    },
    /// Returns the total number of Athlete Contracts saved 
    TokenCount {},
    /// Returns the last round used for Terrand
    LastRound {},
    /// Checks if a locked NFT can be unlocked
    CanUnlockToken {
        /// Athlete ID of the NFT
        athlete_id: String,
        /// Token ID of the NFT to be queried
        token_id: String,
    },
}

/// CW721 Contract Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TokenMsg {
    Mint {
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal Resource Identifier link of the NFT
        token_uri: Option<String>,
        /// Describes the rarity of the NFT 
        rarity: String,
        /// Additional Metadata of Fantasy Athlete tokens
        extension: TokenExtension
    },
    UpdateToken {
        /// Token ID of the NFT to be updated
        token_id: String,
        /// URI link of the NFT image
        token_uri: Option<String>,
        /// Additional Metadata of Fantasy Athlete tokens
        extension: TokenExtension
    },
    UpdateMinter {
        /// Address of the new minter
        minter: String,
    },
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
    IsMintable {
        /// Describes the rarity of the NFT 
        rarity: String,
    },
}

/// Terrand Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TerrandMsg {
    LatestDrand {}
}

/// Terrand Responses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LatestRandomResponse {
    pub round: u64,
    pub randomness: Binary,
    pub worker: String,
}

/// Anchor Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMsg {
    Config {},
    DepositStable {},
    Receive(Cw20ReceiveMsg),
    RedeemStable {},
}

/// Anchor Responses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    pub last_interest_updated: u64,
    pub last_reward_updated: u64,
    pub global_interest_index: Decimal256,
    pub global_reward_index: Decimal256,
    pub anc_emission_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub contract_addr: CanonicalAddr,
    pub owner_addr: CanonicalAddr,
    pub aterra_contract: CanonicalAddr,
    pub interest_model: CanonicalAddr,
    pub distribution_model: CanonicalAddr,
    pub overseer_contract: CanonicalAddr,
    pub collector_contract: CanonicalAddr,
    pub distributor_contract: CanonicalAddr,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}