#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmQuery, WasmMsg, 
    Addr, Coin, Uint128, Timestamp
};
use cosmwasm_storage::to_length_prefixed;
// use cosmwasm_bignumber::{Decimal256};

use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::msg::{
    InstantiateMsg, ExecuteMsg, QueryMsg, TokenMsg, TerrandMsg, AnchorMsg,
    LatestRandomResponse, ConfigResponse, StateResponse, 
    TokenExtension, NftInfoResponse
};
use crate::state::{
    ContractInfoResponse,
    CONTRACT_INFO, TOTAL_DEPOSIT, TOKEN_COUNT, LAST_ROUND, 
    total_deposit, increase_deposit,
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
    _env: Env,
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
        pack_price: msg.pack_price,
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
        ExecuteMsg::Test {} => execute_test(deps, env),
        ExecuteMsg::PurchasePack {
            rand_seed
        } => execute_purchase(deps, env, info, rand_seed),
        ExecuteMsg::DepositStable {} => execute_deposit(deps, env, info),
        ExecuteMsg::RedeemStable {
            amount,
        } => execute_redeem(deps, env, info, amount),
        ExecuteMsg::AddToken {
            token
        } => execute_add_token(deps, env, token),
        ExecuteMsg::TokenTurnover {
            new_contract
        } => execute_token_turnover(deps, env, new_contract),
        ExecuteMsg::LockToken {
            athlete_id,
            token_id,
            duration
        } => execute_lock_token(deps, env, info, athlete_id, token_id, duration),
        ExecuteMsg::UnlockToken {
            athlete_id,
            token_id,
        } => execute_unlock_token(deps, env, info, athlete_id, token_id),
        ExecuteMsg::UpgradeSameToken {
            rarity,
            athlete_id,
            tokens
        } => execute_upgrade_same_token(deps, env, info, rarity, athlete_id, tokens),
        ExecuteMsg::UpgradeRandToken {
            rarity,
            athlete_ids,
            tokens,
            rand_seed
        } => execute_upgrade_rand_token(deps, env, info, rarity, athlete_ids, tokens, rand_seed),
    }
}

pub fn execute_test(
    _deps: DepsMut,
    _env: Env,
) -> Result<Response, ContractError> {

    Ok(Response::new()
        .add_attribute("action", "test")
        .add_attribute("message", "hey I have a message")
    )
}

pub fn execute_purchase(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo, 
    rand_seed: String
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let contract_info = query_contract_info(deps.as_ref()).unwrap();

    if info.funds.len() != 1 || 
        info.funds[0].denom != contract_info.stable_denom || 
        info.funds[0].amount != Uint128::from(contract_info.pack_price)
    {
        return Err(ContractError::WrongAmount{amount:contract_info.pack_price, denom:contract_info.stable_denom})
    }

    // Load pack_len from the state
    let pack_len = query_contract_info(deps.as_ref()).unwrap().pack_len;
    let token_count = query_token_count(deps.as_ref()).unwrap();
   
    let mut mintable_token_list = vec![];

    for n in 0..token_count {
        if query_token_mintable(deps.as_ref(), n.to_string(), "C".to_string()).unwrap_or(false){    
                // Add athlete_id to the mintable list
                // Increment mintable tokens
                mintable_token_list.push(n);
        }
    }

    // Generate the list of athlete IDs to be minted
    // let hex_list = query_terrand(deps, env, pack_len).unwrap();
    // 
    // let hex_list = match query_terrand(deps.branch(), env, pack_len) {
    //     Ok(list) => list,
    //     Err(error) => return Err(ContractError::Std(error)),
    // };
    // let last_round = query_last_round(deps.branch().as_ref()).unwrap();
    
    //let mint_index_list = [0, 0, 0, 0, 0];
    let mint_index_list = hex_to_athlete(deps.branch().as_ref(), rand_seed.clone()).unwrap();
    let last_round = 420;
    let mut response = Response::new()
        .add_attribute("action", "purchase")
        .add_attribute("from", &sender);
    
    // Default NFT rarity is Common
    for index in mint_index_list.iter() {
        let athlete_id = mintable_token_list[*index as usize].to_string();
        let token_address = query_token_address(deps.branch().as_ref(), athlete_id).unwrap();
        
        let mint_msg = TokenMsg::Mint {
            owner: sender.clone().to_string(),
            token_uri: None,
            rarity: "C".to_string(),
            extension: TokenExtension {
                is_locked: false,
                unlock_date: None,
                usage: 3,
            }
        };

        response = response.add_message(WasmMsg::Execute {
            contract_addr: token_address.clone().to_string(),
            msg: to_binary(&mint_msg).unwrap(),
            funds: vec![],
        });

        /*let mint_res = encode_msg_execute(
            to_binary(&mint_msg).unwrap(),
            token_address.clone(),
            vec![]
        )?;*/

        // TODO: handle error from mint_res
        //response = response.add_message(mint_res);
    }
    response = response.add_attribute("last_round", last_round.to_string());
    
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
    
    Ok(Response::new()
        .add_message(anchor_response)
        .add_attribute("action", "deposit")
        .add_attribute("from", &sender)
        .add_attribute("to", &anchor_contract)
        .add_attribute("deposit_amount", &coin_deposit.amount.to_string()))
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

    //TODO: Declare this as a static variable, which is incredibly difficult to do for some reason
    let decimal_fraction: Uint128 = Uint128::from(1_000_000_000_000_000_000 as u128);

    // transform binary response to state response
    let state_response: StateResponse = from_binary(&state_bin)?;
    let exchange_rate: Decimal = state_response.prev_exchange_rate.into();

    let contract_msg = to_binary(&AnchorMsg::RedeemStable{})?;
    let aust_amount = amount.multiply_ratio(
        decimal_fraction,
        decimal_fraction * exchange_rate + decimal_fraction
    );

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

    Ok(Response::new()
        .add_message(anchor_response)
        .add_attribute("action", "receive")
        .add_attribute("from", &sender)
        .add_attribute("to", &anchor_contract)
        .add_attribute("amount", &amount.to_string())
        .add_attribute("aust_amount", &aust_amount.to_string()))
}

pub fn execute_add_token(
    deps: DepsMut,
    _env: Env,
    token: String,
) -> Result<Response, ContractError> {
    let token_addr = deps.api.addr_validate(&token)?;
    let athlete_id = query_token_count(deps.as_ref()).unwrap();

    token_addresses(deps.storage).update::<_, ContractError>(&athlete_id.to_string().as_bytes(), |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token_addr.clone()),
    })?;
    
    increment_token_count(deps.storage)?;
    
    Ok(Response::new()
        .add_attribute("action", "add_tokens")
        .add_attribute("athlete_id", athlete_id.to_string())
    )
}

pub fn execute_token_turnover(
    deps: DepsMut,
    env: Env,
    new_contract: String,
) -> Result<Response, ContractError> {
    let new_address = deps.api.addr_validate(&new_contract)?;
    let token_count = query_token_count(deps.as_ref()).unwrap();
    let mut response = Response::new()
        .add_attribute("action", "token_turnover")
        .add_attribute("from", &env.contract.address)
        .add_attribute("to", &new_address.to_string());

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
        response = response.add_message(token_res);
    }

    Ok(response)
}

pub fn execute_lock_token(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    athlete_id: String,
    token_id: String,
    duration: String
) -> Result<Response, ContractError> {
    let token_address = query_token_address(deps.as_ref(), athlete_id.clone()).unwrap();
    let mut token = query_token_info(deps.as_ref(), athlete_id.clone(), token_id.clone()).unwrap();

    // TODO: Add token ownership authentication when executing this function
    // Use AllNftInfoResponse instead of NftInfoResponse from CW721 package
    // if info.sender != token.owner {
    //     return Err(ContractError::Unauthorized {});
    // }

    if !query_use_token(deps.as_ref(), athlete_id.clone(), token_id.clone())?{
        return Err(ContractError::UsageCapped {});
    }

    let curr_date = env.block.time;
    if duration.clone().eq("hour"){
        token.extension.unlock_date = Some(curr_date.plus_seconds(3_600));
    } else if duration.clone().eq("day"){
        token.extension.unlock_date = Some(curr_date.plus_seconds(86_400));
    } else if duration.clone().eq("week"){
        token.extension.unlock_date = Some(curr_date.plus_seconds(604_800));
    } else {
        token.extension.unlock_date = Some(curr_date.plus_seconds(60));
    }

    let update_msg = TokenMsg::UpdateToken {
        token_id: token_id.clone(),
        token_uri: token.token_uri,
        extension: TokenExtension {
            is_locked: true,
            unlock_date: token.extension.unlock_date,
            usage: token.extension.usage - 1
        }
    };
    
    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_binary(&update_msg).unwrap(),
            funds: vec![],
        })
        .add_attribute("action", "lock_token")
        .add_attribute("token_id", token_id.clone())
        .add_attribute("duration", duration.clone()))
}

pub fn execute_unlock_token(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    athlete_id: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let token_address = query_token_address(deps.as_ref(), athlete_id.clone()).unwrap();
    let token = query_token_info(deps.as_ref(), athlete_id.clone(), token_id.clone()).unwrap();

    // TODO: Add token ownership authentication when executing this function
    // Use AllNftInfoResponse instead of NftInfoResponse from CW721 package
    // if info.sender != token.owner {
    //     return Err(ContractError::Unauthorized {});
    // }

    if !query_unlock_token(deps.as_ref(), env, athlete_id.clone(), token_id.clone())?{
        return Err(ContractError::Locked {});
    }

    let update_msg = TokenMsg::UpdateToken {
        token_id: token_id.clone(),
        token_uri: token.token_uri,
        extension: TokenExtension {
            is_locked: false,
            unlock_date: None,
            usage: token.extension.usage
        }
    };
    
    Ok(Response::new()
        .add_message(WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_binary(&update_msg).unwrap(),
            funds: vec![],
        })
        .add_attribute("action", "unlock_token")
        .add_attribute("token_id", token_id.clone()))
}

pub fn execute_upgrade_same_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rarity: String,
    athlete_id: String, 
    tokens: Vec<String>
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let token_address = query_token_address(deps.as_ref(), athlete_id).unwrap();

    let mut response = Response::new()
        .add_attribute("action", "upgrade_same_token")
        .add_attribute("from", sender.clone());

    // Burn fodder tokens
    for token in tokens.iter() {
        let burn_msg = TokenMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: token.clone()
        };

        response = response.add_message(WasmMsg::Execute {
            contract_addr: token_address.clone().to_string(),
            msg: to_binary(&burn_msg).unwrap(),
            funds: vec![],
        });
    }

    let mut usage_cap = 3;
    if rarity.eq("U"){
        usage_cap = 5;
    } else if rarity.eq("R"){
        usage_cap = 10;
    } else if rarity.eq("L"){
        usage_cap = 999_999_999;
    }
    
    // Mint higher rarity token
    let mint_msg = TokenMsg::Mint {
        owner: sender.clone().to_string(),
        token_uri: None,
        rarity: rarity,
        extension: TokenExtension {
            is_locked: false,
            unlock_date: None,
            usage: usage_cap,
        }
    };

    response = response.add_message(WasmMsg::Execute {
        contract_addr: token_address.clone().to_string(),
        msg: to_binary(&mint_msg).unwrap(),
        funds: vec![],
    });
    
    Ok(response)
}

pub fn execute_upgrade_rand_token(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rarity: String,
    athlete_id: Vec<String>,
    tokens: Vec<String>,
    rand_seed: String
) -> Result<Response, ContractError> {
    let sender = info.sender;

    let mut token_address: Vec<String> = vec![];
    for id in athlete_id.iter() {
        let addr = query_token_address(deps.as_ref(), id.to_string()).unwrap().to_string();
        token_address.push(addr)
    }

    let mut response = Response::new()
        .add_attribute("action", "upgrade_rand_token")
        .add_attribute("from", sender.clone());
    
    // Burn fodder tokens
    for index in 0..2 {
        let burn_msg = TokenMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: tokens[index].clone()
        };

        response = response.add_message(WasmMsg::Execute {
            contract_addr: token_address[index].to_string(),
            msg: to_binary(&burn_msg).unwrap(),
            funds: vec![],
        });
    }

    // Create list of mintable tokens
    let token_count = query_token_count(deps.as_ref()).unwrap();
    let mut mintable_list = vec![];
    for n in 0..token_count {
        if query_token_mintable(deps.as_ref(), n.to_string(), rarity.clone()).unwrap_or(false){    
                // Add athlete_id to the mintable list
                // Increment mintable tokens
                mintable_list.push(n);
        }
    }

    // Select random Athlete Token/Address from the mintable list
    let index_list = hex_to_athlete(deps.branch().as_ref(), rand_seed).unwrap();
    let athlete_id = mintable_list[index_list[0] as usize].to_string();
    let mint_address = query_token_address(deps.branch().as_ref(), athlete_id).unwrap();

    let mut usage_cap = 3;
    if rarity.clone().eq("U"){
        usage_cap = 5;
    } else if rarity.clone().eq("R"){
        usage_cap = 10;
    } else if rarity.clone().eq("L"){
        usage_cap = 999_999_999;
    }

    // Mint higher rarity token
    let mint_msg = TokenMsg::Mint {
        owner: sender.clone().to_string(),
        token_uri: None,
        rarity: rarity.clone(),
        extension: TokenExtension {
            is_locked: false,
            unlock_date: None,
            usage: usage_cap,
        }
    };
    
    response = response.add_message(WasmMsg::Execute {
        contract_addr: mint_address.clone().to_string(),
        msg: to_binary(&mint_msg).unwrap(),
        funds: vec![],
    });
    
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::PackPrice {} => to_binary(&query_pack_price(deps)?),
        QueryMsg::TotalDeposit {} => to_binary(&query_total_deposit(deps)?),
        QueryMsg::TokenContract {
            athlete_id
        } => to_binary(&query_token_address(deps, athlete_id)?),
        QueryMsg::IsTokenMintable {
            athlete_id,
            rarity,
        } => to_binary(&query_token_mintable(deps, athlete_id, rarity)?),
        QueryMsg::TokenCount {} => to_binary(&query_token_count(deps)?),
        QueryMsg::LastRound {} => to_binary(&query_last_round(deps)?),
        QueryMsg::CanUnlockToken {
            athlete_id,
            token_id
        } => to_binary(&query_unlock_token(deps, env, athlete_id, token_id)?),
        QueryMsg::CanUseToken {
            athlete_id,
            token_id
        } => to_binary(&query_use_token(deps, athlete_id, token_id)?),
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

fn query_pack_price(deps: Deps) -> StdResult<u64> {
    Ok(CONTRACT_INFO.load(deps.storage)?.pack_price)
}

fn query_token_address(
    deps: Deps,
    athlete_id: String
) -> StdResult<Addr> {
    token_addresses_read(deps.storage).load(athlete_id.as_bytes())
}

fn query_token_mintable(
    deps: Deps,
    athlete_id: String,
    rarity: String
) -> StdResult<bool> {
    let token_address = query_token_address(deps, athlete_id).unwrap();

    // Query token_address if mintable using the NFT contract's IsMintable{} query
    let msg = TokenMsg::IsMintable { rarity: rarity };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_binary(&msg)?,
    };
    let is_mintable: bool = deps.querier.query(&wasm.into())?;

    Ok(is_mintable)
}

fn query_token_info(
    deps: Deps,
    athlete_id: String,
    token_id: String
) -> StdResult<NftInfoResponse> {
    let token_address = query_token_address(deps, athlete_id).unwrap();

    let msg = TokenMsg::NftInfo { token_id: token_id };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address.to_string(),
        msg: to_binary(&msg)?,
    };

    let token = deps.querier.query::<NftInfoResponse>(&wasm.into())?;

    let nft_info = NftInfoResponse {
        token_uri: token.token_uri,
        rarity: token.rarity,
        extension: TokenExtension {
            is_locked: token.extension.is_locked,
            unlock_date: token.extension.unlock_date,
            usage: token.extension.usage,
        }
    };

    Ok(nft_info)
}

fn query_unlock_token(
    deps: Deps,
    env: Env,
    athlete_id: String,
    token_id: String
) -> StdResult<bool> {
    let token = query_token_info(deps, athlete_id.clone(), token_id.clone()).unwrap();
    let mut can_unlock = false;
    let curr_date = env.block.time;
    let unlock_date = token.extension.unlock_date.unwrap_or_default();
    let zero = Timestamp::from_nanos(0);

    if curr_date.minus_seconds(unlock_date.seconds()).nanos() >= zero.nanos() {
        can_unlock = true;
    }

    Ok(can_unlock)
}

fn query_use_token(
    deps: Deps,
    athlete_id: String,
    token_id: String
) -> StdResult<bool> {
    let token = query_token_info(deps, athlete_id.clone(), token_id.clone()).unwrap();
    let mut can_use = false;

    if token.extension.usage > 0 {
        can_use = true;
    }

    Ok(can_use)
}

fn _query_terrand(
    deps: DepsMut,
    env: Env,
    count: u64
) -> StdResult<String> {
    // Load terrand_addr from the state
    let terrand_addr = query_contract_info(deps.as_ref()).unwrap().terrand_addr;
    //let last_round = query_last_round(deps.as_ref()).unwrap();
    // String length to be returned by terrand should have 3 characters per athlete ID
    let string_len = count * 3;

    let msg = TerrandMsg::LatestDrand {};
    let wasm = encode_msg_query(
        to_binary(&msg).unwrap(),
        terrand_addr
    )?;
    
    let terrand_res: LatestRandomResponse = deps.querier.query(&wasm.into())?;
    let randomness_hash = hex::encode(terrand_res.randomness.as_slice());

    // if terrand_res.round <= last_round {
    //     return Err(StdError::generic_err("The current round has already been used. Please wait for the next round."))
    // }

    update_last_round(deps, env, &terrand_res.round)?;

    let n = randomness_hash
        .char_indices()
        .rev()
        .nth(string_len as usize - 1)
        .map(|(i, _)| i)
        .unwrap();
    let random_string = &randomness_hash[n..];
   
    Ok(random_string.to_string())
}

fn hex_to_athlete(
    deps: Deps,
    rand_seed: String
) -> StdResult<Vec<u64>> {

    // Splits rand_seed into a vector of 3 character strings 
    let hex_list = rand_seed.chars()
        .collect::<Vec<char>>()
        .chunks(3)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

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