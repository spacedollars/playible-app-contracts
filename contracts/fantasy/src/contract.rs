#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg, WasmQuery, 
    Addr, Coin, Uint128
};
use cosmwasm_storage::to_length_prefixed;
use cosmwasm_bignumber::{Decimal256};

use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::msg::{
    InstantiateMsg, ExecuteMsg, QueryMsg, TokenMsg, TerrandMsg, AnchorMsg,
    LatestRandomResponse, ConfigResponse, StateResponse,
};
use crate::state::{
    ContractInfoResponse,
    CONTRACT_INFO, TOTAL_DEPOSIT, TOKEN_COUNT, LAST_ROUND, 
    total_deposit, increase_deposit, decrease_deposit,
    token_count, increment_token_count,
    token_addresses, token_addresses_read,
};
use crate::helpers::{
    encode_msg_execute,
    encode_raw_query,
    encode_msg_query,
};
use crate::querier::{deduct_tax};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fantasy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let anchor_contract = deps.api.addr_validate(&msg.anchor_addr)?;
    let terrand_contract = deps.api.addr_validate(&msg.terrand_addr)?;

    let info = ContractInfoResponse {
        stable_denom: msg.stable_denom,
        anchor_addr: anchor_contract,
        terrand_addr: terrand_contract,
        pack_len: msg.pack_len,
    };

    match msg.tokens {
        Some(m) => execute_add_token(deps.branch(), env, m)?,
        None => Response {
            messages: vec![],
            attributes: vec![],
            events: vec![],
            data: None,
        },
    };

    CONTRACT_INFO.save(deps.branch().storage, &info)?;
    TOTAL_DEPOSIT.save(deps.branch().storage, &0)?;
    TOKEN_COUNT.save(deps.branch().storage, &0)?;
    LAST_ROUND.save(deps.branch().storage, &0)?;
    
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
        ExecuteMsg::PurchasePack {} => execute_purchase(deps, env, info),
        ExecuteMsg::DepositStable {} => execute_deposit(deps, env, info),
        ExecuteMsg::RedeemStable {
            amount,
        } => execute_redeem(deps, env, info, amount),
        ExecuteMsg::AddToken {
            tokens
        } => execute_add_token(deps, env, tokens),
        ExecuteMsg::TokenTurnover {
            new_contract
        } => execute_token_turnover(deps, env, new_contract),
        ExecuteMsg::Test {} => execute_test(deps, env),
    }
}

pub fn execute_test(
    _deps: DepsMut,
    _env: Env,
) -> Result<Response, ContractError> {

    let response = Response {
        messages: vec![],
        attributes: vec![
            attr("action", "test"),
        ],
        events: vec![],
        data: None,
    };
    
    Ok(response)
}

pub fn execute_purchase(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo, 
) -> Result<Response, ContractError> {
    let sender = info.sender;

    // TODO: Generate N token ids based on the pack_len using Terrand
    let mut response = Response::new();

    // Load pack_len from the state
    let pack_len = query_contract_info(deps.as_ref()).unwrap().pack_len;
    let token_count = query_token_count(deps.as_ref()).unwrap();
   
    let mut mintable_token_list = vec![];

    for n in 0..token_count {
        if query_token_mintable(deps.as_ref(), n.to_string()).unwrap_or(false){    
                // Add athlete_id to the mintable list
                // Increment mintable tokens
                mintable_token_list.push(n);
        }
    }

    // Generate the list of athlete IDs to be minted
    // let hex_list = query_terrand(deps, env, pack_len).unwrap();
    let hex_list = match query_terrand(deps.branch(), env, pack_len) {
        Ok(list) => list,
        Err(error) => return Err(ContractError::Std(error)),
    };
    let mint_index_list = hex_to_athlete(deps.branch().as_ref(), hex_list.clone()).unwrap();

    for index in mint_index_list.iter() {
        let athlete_id = mintable_token_list[*index as usize].to_string();
        let token_address = query_token_address(deps.branch().as_ref(), athlete_id).unwrap();

        //TODO: Handle error from query_token_address. Ensure that the generated token ids are a subset of the token addresses
        
        let mint_msg = TokenMsg::Mint {
            owner: sender.clone().to_string(),
            rank: "B".to_string(),
        };

        let mint_res = encode_msg_execute(
            to_binary(&mint_msg).unwrap(),
            token_address.clone(),
            vec![]
        )?;

        // TODO: handle error from mint_res

        response.add_message(mint_res);
    }

    response.add_attribute("action", "purchase");
    response.add_attribute("from", &sender);
    
    Ok(response)
}

pub fn execute_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = info.sender;

    let deposit_amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == "uusd")
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    // coin deposit minus tax
    let coin_deposit = deduct_tax(
        deps.as_ref(), 
        Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount
        }
    )?;

    let anchor_contract = query_contract_info(deps.as_ref()).unwrap().anchor_addr;
    
    // execute anchor's deposit stable contract
    let deposit_msg = to_binary(&AnchorMsg::DepositStable{})?;
    let anchor_response = encode_msg_execute(
        deposit_msg,
        anchor_contract.clone(),
        vec![coin_deposit.clone()]
    )?;

    increase_deposit(deps.storage, coin_deposit.amount.u128() as u64)?;

    let mut response = Response::new();
    response.add_message(anchor_response);
    response.add_attribute("action", "deposit");
    response.add_attribute("from", &sender);
    response.add_attribute("to", &anchor_contract);
    response.add_attribute("deposit_amount", &coin_deposit.amount.to_string());
    
    Ok(response)
}

pub fn execute_redeem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let anchor_contract = query_contract_info(deps.as_ref()).unwrap().anchor_addr;

     // get exchange rate from anchor state
     let state_bin: Binary = encode_raw_query(
        deps.as_ref(), 
        Binary::from(to_length_prefixed(b"state")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let state_response: StateResponse = from_binary(&state_bin)?;
    let exchange_rate: Decimal256 = state_response.prev_exchange_rate.into();

    let contract_msg = to_binary(&AnchorMsg::RedeemStable{})?;
    let aust_amount = amount * (Decimal256::one() / exchange_rate).into();

    // get anchor usd (aust) contract address from anchor config
    let config_bin: Binary = encode_raw_query(
        deps.as_ref(),
        Binary::from(to_length_prefixed(b"config")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let config_response: ConfigResponse = from_binary(&config_bin)?;
    let aterra_contract = deps.api.addr_validate(&config_response.aterra_contract.to_string())?;

    // create a send message
    let msg = to_binary(&Cw20ExecuteMsg::Send{
        amount: aust_amount,
        contract: anchor_contract.to_string(),
        msg: contract_msg
    })?;

    let anchor_response = encode_msg_execute(
        msg,
        aterra_contract,
        vec![]
    )?;

    let mut response = Response::new();
    response.add_message(anchor_response);
    response.add_attribute("action", "receive");
    response.add_attribute("from", &sender);
    response.add_attribute("to", &anchor_contract);
    response.add_attribute("amount", &amount.to_string());
    response.add_attribute("aust_amount", &aust_amount.to_string());

    Ok(response)
}

pub fn execute_add_token(
    deps: DepsMut,
    _env: Env,
    tokens: Vec<String>,
) -> Result<Response, ContractError> {

    for token in tokens.iter() {
        let token_addr = deps.api.addr_validate(&token)?;
        let athlete_id = query_token_count(deps.as_ref()).unwrap();
        token_addresses(deps.storage).update(&athlete_id.to_string().as_bytes(), |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token_addr.clone()),
        })?;
        
        increment_token_count(deps.storage)?;
    }
    
    Ok(Response {
        messages: vec![],
        attributes: vec![
            attr("action", "add_token"),
        ],
        events: vec![],
        data: None,
    })
}

pub fn execute_token_turnover(
    deps: DepsMut,
    env: Env,
    new_contract: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let new_address = deps.api.addr_validate(&new_contract)?;
    let token_count = query_token_count(deps.as_ref()).unwrap();

    for athlete_id in 0..token_count {
        let contract_addr = query_token_address(deps.as_ref(), athlete_id.to_string()).unwrap();

        let update_msg = TokenMsg::UpdateMinter {
            minter: new_address.clone().to_string(),
        };

        let token_res = encode_msg_execute(
            to_binary(&update_msg).unwrap(),
            contract_addr.clone(),
            vec![]
        )?;

        // TODO: handle error from token_res
        response.add_message(token_res);
    }
    
    response.add_attribute("action", "token_turnover");
    response.add_attribute("from", &env.contract.address);
    response.add_attribute("to", &new_address.to_string());

    Ok(response)
}

pub fn update_last_round(
    deps: DepsMut,
    _env: Env,
    new_round: &u64,
) -> StdResult<u64> {
    LAST_ROUND.save(deps.storage, &new_round)?;
    Ok(*new_round)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::TotalDeposit {} => to_binary(&query_total_deposit(deps)?),
        QueryMsg::TokenContract {
            athlete_id
        } => to_binary(&query_token_address(deps, athlete_id)?),
        QueryMsg::IsTokenMintable {
            athlete_id
        } => to_binary(&query_token_mintable(deps, athlete_id)?),
        QueryMsg::TokenCount {} => to_binary(&query_token_count(deps)?),
        QueryMsg::LastRound {} => to_binary(&query_last_round(deps)?),
    }
}

fn query_contract_info(
    deps: Deps,
) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(deps.storage)
}

fn query_total_deposit(
    deps: Deps,
) -> StdResult<Uint128> {
    Ok(Uint128::from(total_deposit(deps.storage)?))
}

fn query_token_count(
    deps: Deps,
) -> StdResult<u64> {
    Ok(token_count(deps.storage)?)
}

fn query_last_round(
    deps: Deps,
) -> StdResult<u64> {
    LAST_ROUND.load(deps.storage)
}

fn query_token_address(
    deps: Deps,
    athlete_id: String
) -> StdResult<Addr> {
    token_addresses_read(deps.storage).load(athlete_id.as_bytes())
}

fn query_token_mintable(
    deps: Deps,
    athlete_id: String
) -> StdResult<bool> {
    let token_address = query_token_address(deps, athlete_id).unwrap();

    // Query token_address if mintable using the NFT contract's IsMintable{} query
    let msg = TokenMsg::IsMintable { rank: "B".to_string() };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_binary(&msg)?,
    };
    let is_mintable: bool = deps.querier.query(&wasm.into())?;

    Ok(is_mintable)
}

fn query_terrand(
    deps: DepsMut,
    env: Env,
    count: u64
) -> StdResult<Vec<String>> {
    // Load terrand_addr from the state
    let terrand_addr = query_contract_info(deps.as_ref()).unwrap().terrand_addr;
    let last_round = query_last_round(deps.as_ref()).unwrap();
    // String length to be returned by terrand should have 3 characters per athlete ID
    let string_len = count * 3;

    let msg = TerrandMsg::LatestDrand {};
    let wasm = encode_msg_query(
        to_binary(&msg).unwrap(),
        terrand_addr
    )?;
    
    let terrand_res: LatestRandomResponse = deps.querier.query(&wasm.into())?;
    let randomness_hash = hex::encode(terrand_res.randomness.as_slice());

    if terrand_res.round <= last_round {
        return Err(StdError::generic_err("The current round has already been used. Please wait for the next round."))
    }

    update_last_round(deps, env, &terrand_res.round)?;

    let n = randomness_hash
        .char_indices()
        .rev()
        .nth(string_len as usize - 1)
        .map(|(i, _)| i)
        .unwrap();
    let random_string = &randomness_hash[n..];
   
    // Splits random_string into a vector of 3 character strings 
    let hex_list = random_string.chars()
        .collect::<Vec<char>>()
        .chunks(3)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    Ok(hex_list)
}

fn hex_to_athlete(
    deps: Deps,
    hex_list: Vec<String>
) -> StdResult<Vec<u64>> {

    // Load contract_count from the state
    let token_count = query_token_count(deps).unwrap();

    let mut athlete_list: Vec<u64> = Vec::new();

    for hex in hex_list.iter(){
        // Convert hexadecimal to decimal
        let deci = u64::from_str_radix(hex, 16).unwrap();
        // Get the athlete IDs by using modulo
        let athlete_id = deci % token_count;

        athlete_list.push(athlete_id);
    }

    Ok(athlete_list)
}