use cosmwasm_std::{
    Api, 
    Binary, 
    Coin, 
    Empty, 
    Extern, 
    Addr, 
    Querier,
    QueryRequest, 
    StdResult, 
    Storage, 
    WasmMsg, 
    WasmQuery, 
    CosmosMsg
};
use cw20::{BalanceResponse};
use crate::msg::{LatestRandomResponse};

pub fn encode_msg_execute(
    msg: Binary,
    address: Addr,
    coin: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    Ok(WasmMsg::Execute {
        contract_addr: address,
        msg: msg,
        send: coin,
    }
    .into())
}

pub fn encode_raw_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: Binary, 
    address: HumanAddr
) -> StdResult<Binary> {
    Ok(deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: address,
        key: key,
    }))?)
}

pub fn encode_msg_query(msg: Binary, address: HumanAddr) -> StdResult<QueryRequest<Empty>> {
    Ok(WasmQuery::Smart {
        contract_addr: address,
        msg: msg,
    }
    .into())
}

pub fn wrapper_msg_get_randomness<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    query: QueryRequest<Empty>,
) -> StdResult<LatestRandomResponse> {
    let res: LatestRandomResponse = deps.querier.query(&query)?;
    Ok(res)
}

pub fn wrapper_msg_anchor_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    query: QueryRequest<Empty>,
) -> StdResult<BalanceResponse> {
    let res: BalanceResponse = deps.querier.query(&query)?;
    Ok(res)
}