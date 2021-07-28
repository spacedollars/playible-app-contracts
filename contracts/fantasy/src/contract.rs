use cosmwasm_std::{
    from_binary, log, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, WasmQuery, StdError, StdResult, Storage, Uint128, Coin,
};
use cosmwasm_storage::to_length_prefixed;
use cosmwasm_bignumber::{Decimal256};

use cw2::set_contract_version;
use cw20::{Cw20HandleMsg};

use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, AnchorMsg, TerrandMsg, TokenMsg, ConfigResponse, 
    LatestRandomResponse, StateResponse, QueryMintMsg, ContractCountResponse,
};
use crate::state::{
    increase_deposit, last_round, last_round_read, total_deposit, total_deposit_read, state, state_read, State,
    token_addresses, token_addresses_read,
    get_contract_count, increment_contract_count,
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

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_contract_version(&mut deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = State {
        stable_denom: msg.stable_denom,
        anchor_addr: msg.anchor_addr,
        terrand_addr: msg.terrand_addr,
        pack_len: msg.pack_len,
    };

    match msg.tokens {
        Some(m) => handle_add_token(deps, env, m)?,
        None => HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        },
    };

    state(&mut deps.storage).save(&info)?;
    last_round(&mut deps.storage).save(&0)?;
    total_deposit(&mut deps.storage).save(&0)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::PurchasePack {} => handle_purchase(deps, env),
        HandleMsg::DepositStable {} => handle_deposit(deps, env),
        HandleMsg::RedeemStable {
            amount,
        } => handle_redeem(deps, env, amount),
        HandleMsg::AddToken {
            tokens
        } => handle_add_token(deps, env, tokens),
        HandleMsg::TokenTurnover {
            new_contract
        } => handle_token_turnover(deps, env, new_contract),
        HandleMsg::Test {} => handle_test(deps, env),
    }
}

pub fn handle_test<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {

    let response = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "test"),
        ],
        data: None,
    };
    
    Ok(response)
}


pub fn handle_purchase<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    // TODO: Generate N token ids based on the pack_len using Terrand
    let mut mint_responses = vec![];

    
    // Load pack_len from the state
    let pack_len: u128 = query_state(&deps).unwrap().pack_len.u128();
    let token_count: u128 = query_contract_count(&deps).unwrap().count.into();
   
    let mut mintable_token_list = vec![];

    for n in 0..token_count {
        if query_token_mintable(&deps, n.to_string()).unwrap_or(false){    
                // Add athlete_id to the mintable list
                // Increment mintable tokens
                mintable_token_list.push(n);
        }
    }

    // Generate the list of athlete IDs to be minted
    //let hex_list = query_terrand(deps, env, pack_len).unwrap();
    let hex_list = match query_terrand(deps, env, pack_len) {
        Ok(list) => list,
        Err(error) => return Err(error),
    };
    let mint_index_list = hex_to_athlete(&deps, hex_list.clone()).unwrap();

    for index in mint_index_list.iter() {
        let athlete_id = mintable_token_list[*index as usize].to_string();
        let token_address = query_token_address(&deps, athlete_id).unwrap();

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

    let response = HandleResponse {
        messages: mint_responses,
        log: vec![
            log("action", "purchase"),
            log("from", &sender),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn handle_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    let deposit_amount: Uint128 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == "uusd")
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    // coin deposit minus tax
    let coin_deposit = deduct_tax(
        &deps, 
        Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount
        }
    )?;
    let contract = query_state(&deps).unwrap().anchor_addr;
    
    // execute anchor's deposit stable contract
    let deposit_msg = to_binary(&AnchorMsg::DepositStable{})?;
    let anchor_res = encode_msg_execute(
        deposit_msg,
        contract.clone(),
        vec![coin_deposit.clone()]
    )?;

    increase_deposit(&mut deps.storage, coin_deposit.amount.u128() as u64)?;

    let response = HandleResponse {
        messages: vec![anchor_res],
        log: vec![
            log("action", "deposit"),
            log("from", &sender),
            log("to", &contract),
            log("deposit_amount", &coin_deposit.amount),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn handle_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;
    let anchor_contract = query_state(&deps).unwrap().anchor_addr;

     // get exchange rate from anchor state
     let state_bin: Binary = encode_raw_query(
        &deps, 
        Binary::from(to_length_prefixed(b"state")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let state_response: StateResponse = from_binary(&state_bin)?;
    let exchange_rate: Decimal256 = state_response.prev_exchange_rate.into();

    let contract_msg = to_binary(&AnchorMsg::RedeemStable{})?;
    let aust_amount = amount * (Decimal256::one() / exchange_rate).into();

    let logs = vec![
        log("action", "receive"),
        log("from", &sender),
        log("to", &anchor_contract),
        log("amount", &amount),
        log("aust_amount", &aust_amount),
    ];

    // get anchor usd (aust) contract address from anchor config
    let config_bin: Binary = encode_raw_query(
        &deps,
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

    let res = HandleResponse {
        messages: vec![anchor_response],
        log: logs,
        data: None,
    };
    
    Ok(res)
}

pub fn handle_add_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    tokens: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {

    for token in tokens.iter() {
        let contract_count = query_contract_count(&deps).unwrap();
        let athlete_id = contract_count.count;
        token_addresses(&mut deps.storage).update(&athlete_id.to_string().as_bytes(), |old| match old {
            Some(_) => Err(StdError::generic_err("athlete_id already claimed")),
            None => Ok(token.clone()),
        })?;
        
        increment_contract_count(&mut deps.storage)?;
    }

    let response = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "add_token"),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn handle_token_turnover<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    new_contract: HumanAddr,
) -> StdResult<HandleResponse> {

    let contract_count = query_contract_count(&deps).unwrap();
    let mut token_responses = vec![];

    for athlete_id in 0..contract_count.count {
        let contract_addr = query_token_address(&deps, athlete_id.to_string()).unwrap();

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

    let response = HandleResponse {
        messages: token_responses,
        log: vec![
            log("action", "token_turnover"),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn update_last_round<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: &Env,
    new_round: &u64,
) -> StdResult<u64> {
    last_round(&mut deps.storage).save(&new_round)?;
    Ok(*new_round)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::TotalDeposit {} => to_binary(&query_total_deposit(deps)?),
        QueryMsg::LastRound {} => to_binary(&query_last_round(deps)?),
        QueryMsg::TokenContract {
            athlete_id
        } => to_binary(&query_token_address(deps, athlete_id)?),
        QueryMsg::IsTokenMintable {
            athlete_id
        } => to_binary(&query_token_mintable(deps, athlete_id)?),
        QueryMsg::ContractCount {} => to_binary(&query_contract_count(deps)?),
    }
}

fn query_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<State> {
    state_read(&deps.storage).load()
}

fn query_total_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint128> {
    Ok(Uint128(total_deposit_read(&deps.storage).load()? as u128))
}

fn query_last_round<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<u64> {
    Ok(last_round_read(&deps.storage).load()?)
}

fn query_contract_count<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ContractCountResponse> {
    let count = get_contract_count(&deps.storage)?;
    Ok(ContractCountResponse { count })
}

fn query_token_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    athlete_id: String
) -> StdResult<HumanAddr> {
    token_addresses_read(&deps.storage).load(athlete_id.as_bytes())
}

fn query_token_mintable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    athlete_id: String
) -> StdResult<bool> {
    let token_address = query_token_address(&deps, athlete_id).unwrap();

    // TODO: Query token_address if mintable using the NFT contract's IsMintable{} query
    let msg = QueryMintMsg::IsMintable { rank: "B".to_string() };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address,
        msg: to_binary(&msg)?,
    };
    let is_mintable: bool = deps.querier.query(&wasm.into())?;

    Ok(is_mintable)
}

fn query_terrand<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    count: u128
) -> StdResult<Vec<String>> {
    // Load terrand_addr from the state
    let terrand_addr = query_state(&deps).unwrap().terrand_addr;
    let last_round = query_last_round(&deps).unwrap();
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

    update_last_round(deps, &env, &terrand_res.round)?;

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

fn hex_to_athlete<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    hex_list: Vec<String>
) -> StdResult<Vec<u64>> {

    // Load contract_count from the state
    let contract_count = query_contract_count(&deps).unwrap().count;

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