use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,
    /// Describes the rank of the NFT 
    pub rank: String,
    /// Determines whether or not the NFT is locked for Fantasy Sports
    pub is_locked: bool,
    /// Determines the unlock date after the NFT has been locked
    pub unlock_date: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("nft_info");
pub const MINTER: Item<Addr> = Item::new("minter");
pub const BASE_COUNT: Item<u64> = Item::new("base_tokens");
pub const SILVER_COUNT: Item<u64> = Item::new("silver_tokens");
pub const GOLD_COUNT: Item<u64>  = Item::new("gold_tokens");

// Stored as (granter, operator) giving operator full control over granter's account
pub const OPERATORS: Map<(&Addr, &Addr), Expiration> = Map::new("operators");

pub fn base_tokens(storage: &dyn Storage) -> StdResult<u64> {
    Ok(BASE_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_base_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = base_tokens(storage)? + 1;
    BASE_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn silver_tokens(storage: &dyn Storage) -> StdResult<u64> {
    Ok(SILVER_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_silver_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = silver_tokens(storage)? + 1;
    SILVER_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn gold_tokens(storage: &dyn Storage) -> StdResult<u64> {
    Ok(GOLD_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_gold_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = gold_tokens(storage)? + 1;
    GOLD_COUNT.save(storage, &val)?;
    Ok(val)
}

pub struct TokenIndexes<'a> {
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Vec<u8>, Vec<u8>), TokenInfo>,
}

impl<'a> IndexList<TokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo>> + '_> {
        let v: Vec<&dyn Index<TokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a>() -> IndexedMap<'a, &'a str, TokenInfo, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        owner: MultiIndex::new(
            |d, k| (Vec::from(d.owner.as_ref()), k),
            "tokens",
            "tokens__owner",
        ),
    };
    IndexedMap::new("tokens", indexes)
}
