use cosmwasm_std::{
    Deps, 
    Binary, 
    Coin, 
    Empty, 
    Addr, 
    QueryRequest, 
    StdResult, 
    WasmMsg, 
    WasmQuery, 
    CosmosMsg
};
use cw20::{BalanceResponse};

pub fn encode_msg_execute(
    msg: Binary,
    address: Addr,
    coin: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    Ok(WasmMsg::Execute {
        contract_addr: address.to_string(),
        msg: msg,
        funds: coin,
    }
    .into())
}

pub fn encode_raw_query(
    deps: Deps,
    key: Binary, 
    address: Addr
) -> StdResult<Binary> {
    Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: address.to_string(),
        key: key,
    }))?)
}

pub fn encode_msg_query(msg: Binary, address: Addr) -> StdResult<QueryRequest<Empty>> {
    Ok(WasmQuery::Smart {
        contract_addr: address.to_string(),
        msg: msg,
    }
    .into())
}