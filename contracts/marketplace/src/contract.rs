#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmQuery, WasmMsg, 
    Coin, Uint128, Addr
};

use cw2::set_contract_version;
// use cw20::{Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::msg::{ InstantiateMsg, ExecuteMsg, QueryMsg, TokenMsg, OwnerOfResponse };
use crate::state::{ ContractInfoResponse, CONTRACT_INFO };
// use crate::helpers::{
//     encode_msg_execute,
//     encode_raw_query,
//     encode_msg_query,
// };
// use crate::querier::{deduct_tax};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Length of a serialized compressed public key
const ECDSA_COMPRESSED_PUBKEY_LEN: usize = 33;
/// Length of a serialized uncompressed public key
const ECDSA_UNCOMPRESSED_PUBKEY_LEN: usize = 65;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_addr = deps.api.addr_validate(&msg.admin_addr)?;

    let public_key = base64::decode(&msg.public_key).unwrap();

    #[cfg(not(feature = "backtraces"))]
    check_pubkey(&public_key).map_err(|e| cosmwasm_std::StdError::ParseErr {
        target_type: "public key".to_string(),
        msg: format!("Parsing Public Key: {:?}", &e),
    })?;

    #[cfg(feature = "backtraces")]
    check_pubkey(&public_key).map_err(|e| cosmwasm_std::StdError::ParseErr {
        target_type: "public key".to_string(),
        msg: format!("Parsing Public Key: {:?}", &e),
        backtrace: Default::default(),
    })?;

    let info = ContractInfoResponse {
        name: msg.name,
        admin_addr: admin_addr,
        stable_denom: msg.stable_denom,
        public_key: msg.public_key
    };

    CONTRACT_INFO.save(deps.branch().storage, &info)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TempTransaction {
            contract_addr,
            owner_addr,
            token_id,
            buyer_addr,
            price
        } => execute_temp_transaction(deps, env, info, contract_addr, owner_addr, token_id, buyer_addr, price),
        ExecuteMsg::SetAdmin { 
            new_addr 
        } => set_admin_addr(deps, info, new_addr),
        ExecuteMsg::SetPublicKey { 
            public_key 
        } => set_public_key(deps, info, public_key),
    }
}

pub fn execute_temp_transaction(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
    owner_addr: String,
    token_id: String,
    buyer_addr: String,
    price: Uint128
) -> Result<Response, ContractError> {

    let collection = deps.api.addr_validate(&contract_addr)?;
    let owner = deps.api.addr_validate(&owner_addr)?;
    let buyer = deps.api.addr_validate(&buyer_addr)?;

    if !query_temp_is_valid(
        deps.as_ref(), 
        collection.clone(),
        owner.clone(),
        token_id.clone(),
        buyer.clone(),
        price.clone(),
        info.sender,
        info.funds
    )?{
        return Err(ContractError::InvalidMessage {});
    }

    // if info.sender != buyer.clone() {
    //     return Err(ContractError::BuyerMismatch{})
    // }

    // if info.funds.len() != 1 || 
    //     info.funds[0].denom != contract_info.stable_denom || 
    //     info.funds[0].amount != price.clone() {
    //     return Err(ContractError::WrongAmount{amount: price.clone(), denom: contract_info.stable_denom})
    // }

    let mut response = Response::new()
        .add_attribute("action", "temp_transaction")
        .add_attribute("collection", collection.clone())
        .add_attribute("owner", owner.clone())
        .add_attribute("token_id", token_id.clone())
        .add_attribute("buyer", buyer.clone())
        .add_attribute("price", price.clone());

    let transfer_msg = TokenMsg::TransferNft {
        recipient: buyer.clone().to_string(),
        token_id: token_id.clone()
    };

    response = response.add_message(WasmMsg::Execute {
        contract_addr: collection.clone().to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    });

    response = response.add_message(BankMsg::Send {
        to_address: owner.clone().to_string(),
        amount: vec![
            Coin {
                denom: "uusd".to_string(),
                amount: price.clone()
            }
        ],
    });

    Ok(response)
}

pub fn set_admin_addr(
    mut deps: DepsMut,
    info: MessageInfo,
    new_addr: String,
) -> Result<Response, ContractError> {

    let contract_info = query_contract_info(deps.as_ref()).unwrap();
    let new_address = deps.api.addr_validate(&new_addr)?;
    let old_address = contract_info.admin_addr;

    if info.sender != old_address.clone() {
        return Err(ContractError::Unauthorized {});
    }

    let update = ContractInfoResponse {
        name: contract_info.name,
        admin_addr: new_address.clone(),
        stable_denom: contract_info.stable_denom,
        public_key: contract_info.public_key
    };

    CONTRACT_INFO.save(deps.branch().storage, &update)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin_addr")
        .add_attribute("from", old_address.clone())
        .add_attribute("to", new_addr.clone()))
}

pub fn set_public_key(
    mut deps: DepsMut,
    info: MessageInfo,
    public_key: String,
) -> Result<Response, ContractError> {

    let contract_info = query_contract_info(deps.branch().as_ref()).unwrap();

    if info.sender != contract_info.admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    let update = ContractInfoResponse {
        name: contract_info.name,
        admin_addr: contract_info.admin_addr,
        stable_denom: contract_info.stable_denom,
        public_key: public_key.clone()
    };

    CONTRACT_INFO.save(deps.branch().storage, &update)?;

    Ok(Response::new()
        .add_attribute("action", "set_public_key")
        .add_attribute("sender", info.sender)
        .add_attribute("public_key", public_key.clone()))
}

fn check_pubkey(data: &[u8]) -> Result<(), ContractError> {

    let ok = match data.first() {
        Some(0x02) | Some(0x03) => data.len() == ECDSA_COMPRESSED_PUBKEY_LEN,
        Some(0x04) => data.len() == ECDSA_UNCOMPRESSED_PUBKEY_LEN,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(ContractError::InvalidSecp256k1PubkeyFormat {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::Admin {} => to_binary(&query_admin(deps)?),
        QueryMsg::PublicKey {} => to_binary(&query_public_key(deps)?),
    }
}

fn query_contract_info(
    deps: Deps,
) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
}

fn query_admin(
    deps: Deps,
) -> StdResult<String> {
    Ok(query_contract_info(deps).unwrap().admin_addr.to_string())
}

fn query_public_key(
    deps: Deps,
) -> StdResult<String> {
    Ok(query_contract_info(deps).unwrap().public_key)
}

fn query_temp_is_valid(
    deps: Deps,
    contract_addr: Addr,
    owner_addr: Addr,
    token_id: String,
    buyer_addr: Addr,
    price: Uint128,
    sender: Addr,
    funds: Vec<Coin>
) -> StdResult<bool> {

    let contract_info = query_contract_info(deps).unwrap();
    let token_owner = query_token_owner(deps, contract_addr.clone(), token_id.clone(),).unwrap();
    let mut is_valid = false;
    
    // If the token owner matches provided owner address
    if token_owner.owner == owner_addr.clone() {
        // If token buyer address matches sender address
        if sender != buyer_addr.clone() {
            // If the provided funds matches the listed sale price
            if funds.len() != 1 || 
               funds[0].denom != contract_info.stable_denom || 
               funds[0].amount != price.clone() {
                is_valid = true;
            }
        }
    }

    Ok(is_valid)
}

fn query_token_owner(
    deps: Deps,
    contract_addr: Addr,
    token_id: String
) -> StdResult<OwnerOfResponse> {

    let msg = TokenMsg::OwnerOf { token_id: token_id, include_expired: None };
    let wasm = WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&msg)?,
    };

    let all_nft_info = deps.querier.query::<OwnerOfResponse>(&wasm.into())?;

    Ok(all_nft_info)
}