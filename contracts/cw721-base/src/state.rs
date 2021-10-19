use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage};

use cw721::{ContractInfoResponse, CustomMsg, Cw721, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub struct Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub minter: Item<'a, Addr>,
    pub common_count: Item<'a, u64>,
    pub uncommon_count: Item<'a, u64>,
    pub rare_count: Item<'a, u64>,
    pub legendary_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,

    pub(crate) _custom_response: PhantomData<C>,
}

// This is a signal, the implementations are in other files
impl<'a, T, C> Cw721<T, C> for Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
}

impl<T, C> Default for Cw721Contract<'static, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "minter",
            "common_tokens",
            "uncommon_tokens",
            "rare_tokens",
            "legendary_tokens",
            "operators",
            "tokens",
            "tokens__owner",
        )
    }
}

impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn new(
        contract_key: &'a str,
        minter_key: &'a str,
        common_count_key: &'a str,
        uncommon_count_key: &'a str,
        rare_count_key: &'a str,
        legendary_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            minter: Item::new(minter_key),
            common_count: Item::new(common_count_key),
            uncommon_count: Item::new(uncommon_count_key),
            rare_count: Item::new(rare_count_key),
            legendary_count: Item::new(legendary_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            _custom_response: PhantomData,
        }
    }

    pub fn common_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.common_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_common_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.common_count(storage)? + 1;
        self.common_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn uncommon_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.uncommon_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_uncommon_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.uncommon_count(storage)? + 1;
        self.uncommon_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn rare_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.rare_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_rare_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.rare_count(storage)? + 1;
        self.rare_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn legendary_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.legendary_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_legendary_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.legendary_count(storage)? + 1;
        self.legendary_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// Describes the rarity of the NFT 
    pub rarity: String,
    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
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

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    // pk goes to second tuple element
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), TokenInfo<T>>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(d: &TokenInfo<T>, k: Vec<u8>) -> (Addr, Vec<u8>) {
    (d.owner.clone(), k)
}
