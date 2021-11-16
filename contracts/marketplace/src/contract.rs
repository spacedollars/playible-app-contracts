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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_addr = deps.api.addr_validate(&msg.admin_addr)?;

    let info = ContractInfoResponse {
        name: msg.name,
        admin_addr: admin_addr,
        stable_denom: msg.stable_denom,
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


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
    }
}

fn query_contract_info(
    deps: Deps,
) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
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
    let token_info = query_token_info(deps, contract_addr.clone(), token_id.clone(),).unwrap();
    let mut is_valid = false;
    
    // If the token owner matches provided owner address
    if token_info.owner == owner_addr.clone() {
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

fn query_token_info(
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