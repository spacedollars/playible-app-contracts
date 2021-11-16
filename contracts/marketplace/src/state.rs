use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::{Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    /// contract name 
    pub name: String,
    /// contract admin
    pub admin_addr: Addr,
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");