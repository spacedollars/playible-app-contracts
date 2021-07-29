use cosmwasm_std::{
    from_binary, log, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, WasmQuery, StdError, StdResult, Storage, Uint128, Coin,
};
use cosmwasm_storage::to_length_prefixed;
use cosmwasm_bignumber::{Decimal256};

use cw2::set_contract_version;
use cw20::{Cw20HandleMsg};

use crate::error::ContractError;
use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, TokenMsg, TerrandMsg, AnchorMsg,
    ContractCountResponse, LatestRandomResponse, ConfigResponse, StateResponse,
};
use crate::state::{
    CONTRACT_INFO, LAST_ROUND, ContractInfoResponse,
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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = ContractInfoResponse {
        stable_denom: msg.stable_denom,
        anchor_addr: msg.anchor_addr,
        terrand_addr: msg.terrand_addr,
        pack_len: msg.pack_len,
    };

    match msg.tokens {
        Some(m) => handle_add_token(deps, env, m)?,
        None => Response {
            messages: vec![],
            attributes: vec![],
            events: vec![],
            data: None,
        },
    };

    CONTRACT_INFO.save(deps.storage, &info)?;
    TOTAL_DEPOSIT.save(deps.storage, &0)?;
    TOKEN_COUNT.save(deps.storage, &0)?;
    LAST_ROUND.save(deps.storage, &0)?;
    
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<HandleResponse> {
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo, 
) -> Result<Response, ContractError> {
    let sender = info.sender;

    // TODO: Generate N token ids based on the pack_len using Terrand
    let mut mint_responses = vec![];

    // Load pack_len from the state
    let pack_len = query_contract_info(deps.as_ref()).unwrap().pack_len;
    let token_count = query_contract_count(deps.as_ref()).unwrap().count);
   
    let mut mintable_token_list = vec![];

    for n in 0..token_count {
        if query_token_mintable(deps.as_ref(), n.to_string()).unwrap_or(false){    
                // Add athlete_id to the mintable list
                // Increment mintable tokens
                mintable_token_list.push(n);
        }
    }

    // Generate the list of athlete IDs to be minted
    //let hex_list = query_terrand(deps, env, pack_len).unwrap();
    let hex_list = match query_terrand(deps.as_ref(), env, pack_len) {
        Ok(list) => list,
        Err(error) => return Err(error),
    };
    let mint_index_list = hex_to_athlete(deps.as_ref(), hex_list.clone()).unwrap();

    for index in mint_index_list.iter() {
        let athlete_id = mintable_token_list[*index as usize].to_string();
        let token_address = query_token_address(deps.as_ref(), athlete_id).unwrap();

        //TODO: Handle error from query_token_address. Ensure that the generated token ids are a subset of the token addresses
        
        let mint_msg = TokenMsg::Mint {
            owner: sender.clone(),
            rank: "B".to_string(),
        };

        let mint_res = encode_msg_execute(
            to_binary(&mint_msg).unwrap(),
            token_address.clone(),
            vec![]
        )?;

        // TODO: handle error from mint_res

        mint_responses.push(mint_res);
    }
    
    Ok(Response {
        messages: mint_responses,
        attributes: vec![
            attr("action", "purchase"),
            attr("from", &sender),
        ],
        events: vec![],
        data: None,
    })
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = info.sender;

    let deposit_amount: Uint128 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == "uusd")
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    // coin deposit minus tax
    let coin_deposit = deduct_tax(
        deps, 
        Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount
        }
    )?;
    let contract = query_contract_info(deps.as_ref()).unwrap().anchor_addr;
    
    // execute anchor's deposit stable contract
    let deposit_msg = to_binary(&AnchorMsg::DepositStable{})?;
    let anchor_res = encode_msg_execute(
        deposit_msg,
        contract.clone(),
        vec![coin_deposit.clone()]
    )?;

    increase_deposit(deps, coin_deposit.amount.u128() as u64)?;
    
    Ok(Response {
        messages: vec![anchor_res],
        attributes: vec![
            attr("action", "deposit"),
            attr("from", &sender),
            attr("to", &contract),
            attr("deposit_amount", &coin_deposit.amount),
        ],
        events: vec![],
        data: None,
    })
}

pub fn execute_redeem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let anchor_contract = query_contract_info(deps.as_ref()).unwrap().anchor_addr;

     // get exchange rate from anchor state
     let state_bin: Binary = encode_raw_query(
        deps, 
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
        deps,
        Binary::from(to_length_prefixed(b"config")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let config_response: ConfigResponse = from_binary(&config_bin)?;
    let aterra_contract = deps.api.human_address(&config_response.aterra_contract)?;

    // create a send message
    let msg = to_binary(&Cw20HandleMsg::Send{
        amount: aust_amount,
        contract: anchor_contract,
        msg: Some(contract_msg)
    })?;

    let anchor_response = encode_msg_execute(
        msg,
        aterra_contract,
        vec![]
    )?;

    Ok(Response {
        messages: vec![anchor_response],
        attributes: vec![
            attr("action", "receive"),
            attr("from", &sender),
            attr("to", &anchor_contract),
            attr("amount", &amount),
            attr("aust_amount", &aust_amount),
        ];,
        events: vec![],
        data: None,
    })
}

pub fn execute_add_token(
    deps: DepsMut,
    _env: Env,
    tokens: Vec<HumanAddr>,
) -> Result<Response, ContractError> {

    for token in tokens.iter() {
        let athlete_id = query_token_count(deps.as_ref()).unwrap().count;
        token_addresses(deps.storage).update(&athlete_id.to_string().as_bytes(), |old| match old {
            Some(_) => Err(StdError::generic_err("athlete_id already claimed")),
            None => Ok(token.clone()),
        })?;
        
        increment_token_count(deps.storage)?;
    }
    
    Ok(response = Response {
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
    _env: Env,
    new_contract: HumanAddr,
) -> Result<Response, ContractError> {

    let token_count = query_contract_count(deps.as_ref()).unwrap().count;
    let mut token_responses = vec![];

    for athlete_id in 0..token_count {
        let contract_addr = query_token_address(deps.as_ref(), athlete_id.to_string()).unwrap();

        let update_msg = TokenMsg::UpdateMinter {
            minter: new_contract.clone(),
        };

        let token_res = encode_msg_execute(
            to_binary(&update_msg).unwrap(),
            contract_addr.clone(),
            vec![]
        )?;

        // TODO: handle error from token_res
        token_responses.push(token_res);
    }
    
    Ok(Response {
        messages: token_responses,
        attributes: vec![
            attr("action", "token_turnover"),
        ],
        events: vec![],
        data: None,
    })
}

pub fn update_last_round(
    deps: DepsMut,
    _env: Env,
    new_round: &u64,
) -> StdResult<u64> {
    let prev_round = LAST_ROUND.load(deps.storage)?;
    LAST_ROUND.save(deps.storage, &new_round)?;

    Ok(Response {
        messages: vec![],
        attributes: vec![
            attr("action", "update_last_round"),
            attr("prev_round", prev_round),
            attr("new_round", new_round),
        ],
        events: vec![],
        data: None,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
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
    Ok(Uint128(total_deposit(deps.storage).load()? as u128))
}

fn query_last_round(
    deps: Deps,
) -> StdResult<u64> {
    LAST_ROUND.load(deps.storage)
}

fn query_token_count(
    deps: Deps,
) -> StdResult<TokenCountResponse> {
    let count = token_count(deps.storage)?;
    Ok(TokenCountResponse { count })
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

    // TODO: Query token_address if mintable using the NFT contract's IsMintable{} query
    let msg = QueryMintMsg::IsMintable { rank: "B".to_string() };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address,
        msg: to_binary(&msg)?,
    };
    let is_mintable: bool = deps.querier.query(&wasm.into())?;

    Ok(is_mintable)
}

fn query_terrand(
    deps: Deps,
    env: Env,
    count: u128
) -> StdResult<Vec<String>> {
    // Load terrand_addr from the state
    let terrand_addr = query_contract_info(deps).unwrap().terrand_addr;
    let last_round = query_last_round(deps).unwrap();
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
    let contract_count = query_token_count(deps).unwrap().count;

    let mut athlete_list: Vec<u64> = Vec::new();

    for hex in hex_list.iter(){
        // Convert hexadecimal to decimal
        let deci = u64::from_str_radix(hex, 16).unwrap();
        // Get the athlete IDs by using modulo
        let athlete_id = deci % contract_count;

        athlete_list.push(athlete_id);
    }

    Ok(athlete_list)
}