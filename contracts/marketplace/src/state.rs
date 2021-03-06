use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    /// contract name 
    pub name: String,
    /// contract admin
    pub admin_addr: Addr,
    /// Stable coin denomination. 
    pub stable_denom: String,
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");
pub const PUBLIC_KEY: Item<String> = Item::new("public_key");