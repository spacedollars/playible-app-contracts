use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::{Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    /// Stable coin denomination. 
    pub stable_denom: String,
    // anchor contract address for depositing the rewards
    pub anchor_addr: Addr,
    // terrand contract address for calling Oracle's DRand
    pub terrand_addr: Addr,
    /// contract address for the CW721 Athlete contract
    pub athlete_addr: Addr,
    /// contract admin
    pub admin_addr: Addr,
    /// number of NFT players to be pulled per pack
    pub pack_len: u64,
    /// price of each pack
    pub pack_price: u64,
    // Maximum number tokens to be minted for each rarity
    pub common_cap: u64,
    pub uncommon_cap: u64,
    pub rare_cap: u64,
    pub legendary_cap: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AthleteInfo {
    /// Symbol used for token_id generation
    pub symbol: String,
    /// Current number of minted tokens per rarity
    pub common_count: u64,
    pub uncommon_count: u64,
    pub rare_count: u64,
    pub legendary_count: u64,
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");
pub const TOTAL_DEPOSIT: Item<u64> = Item::new("total_deposit");
pub const ANCHOR_ADDR: Item<Addr> = Item::new("anchor_addr");
pub const TERRAND_ADDR: Item<Addr> = Item::new("terrand_addr");
pub const PACK_LEN: Item<u64>  = Item::new("pack_len");
pub const TOKEN_COUNT: Item<u64>  = Item::new("token_count");
pub const LAST_ROUND: Item<u64>  = Item::new("last_round");
pub const TOKEN_ADDRESSES_PREFIX: &[u8] = b"token_addresses";
pub const ATHLETE_LIST_PREFIX: &[u8] = b"athlete_list";

pub fn total_deposit(storage: &dyn Storage) -> StdResult<u64> {
    Ok(TOTAL_DEPOSIT.may_load(storage)?.unwrap_or_default())
}

pub fn increase_deposit(storage: &mut dyn Storage, amount: u64) -> StdResult<u64> {
    let val = total_deposit(storage)? + amount;
    TOTAL_DEPOSIT.save(storage, &val)?;
    Ok(val)
}

pub fn decrease_deposit(storage: &mut dyn Storage, amount: u64) -> StdResult<u64> {
    let val = total_deposit(storage)? - amount;
    TOTAL_DEPOSIT.save(storage, &val)?;
    Ok(val)
}

pub fn token_count(storage: &dyn Storage) -> StdResult<u64> {
    Ok(TOKEN_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_token_count(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = token_count(storage)? + 1;
    TOKEN_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn token_addresses(storage: &mut dyn Storage) -> Bucket<Addr> {
    bucket(storage, TOKEN_ADDRESSES_PREFIX)
}

pub fn token_addresses_read(storage: &dyn Storage) -> ReadonlyBucket<Addr> {
    bucket_read(storage, TOKEN_ADDRESSES_PREFIX)
}

pub fn athlete_list(storage: &mut dyn Storage) -> Bucket<AthleteInfo> {
    bucket(storage, ATHLETE_LIST_PREFIX)
}

pub fn athlete_list_read(storage: &dyn Storage) -> ReadonlyBucket<AthleteInfo> {
    bucket_read(storage, ATHLETE_LIST_PREFIX)
}