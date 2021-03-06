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
    /// contract admin
    pub admin_addr: Addr,
    /// contract address for the CW721 Athlete contract
    pub athlete_addr: Addr,
    /// contract address for the Marketplace contract
    pub marketplace_addr: Addr,
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
pub const LAST_ROUND: Item<u64>  = Item::new("last_round");
pub const ATHLETE_LIST_PREFIX: &[u8] = b"athlete_list";
pub const ATHLETE_COUNT: Item<u64>  = Item::new("athlete_count");

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

pub fn athlete_count(storage: &dyn Storage) -> StdResult<u64> {
    Ok(ATHLETE_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_athlete_count(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = athlete_count(storage)? + 1;
    ATHLETE_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn athlete_list(storage: &mut dyn Storage) -> Bucket<AthleteInfo> {
    bucket(storage, ATHLETE_LIST_PREFIX)
}

pub fn athlete_list_read(storage: &dyn Storage) -> ReadonlyBucket<AthleteInfo> {
    bucket_read(storage, ATHLETE_LIST_PREFIX)
}