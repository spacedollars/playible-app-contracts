// use cosmwasm_bignumber::{Decimal256};
use cosmwasm_std::{
    Decimal, Deps, Coin, StdResult, Uint128,
};
use terra_cosmwasm::TerraQuerier;


pub fn compute_tax(
    deps: Deps,
    coin: &Coin,
) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint128::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);

    //TODO: Declare this as a static variable, which is incredibly difficult to do for some reason
    let decimal_fraction: Uint128 = Uint128::from(1_000_000_000_000_000_000 as u128);

    Ok(std::cmp::min(
        (coin.amount.checked_sub(coin.amount.multiply_ratio(
            decimal_fraction,
            decimal_fraction * tax_rate + decimal_fraction,
        )))?,
        tax_cap,
    ))

    /*Ok(std::cmp::min(
        amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax_rate)).into(),
        tax_cap,
    ))*/
}

pub fn compute_price_with_tax(
    deps: Deps,
    coin: &Coin,
) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint128::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);

    //TODO: Declare this as a static variable, which is incredibly difficult to do for some reason
    let decimal_fraction: Uint128 = Uint128::from(1_000_000_000_000_000_000 as u128);
    
    //let amount = Uint128::from(coin.amount);
    Ok(std::cmp::min(
        (coin.amount.checked_sub(coin.amount.multiply_ratio(
            decimal_fraction,
            decimal_fraction * tax_rate + decimal_fraction,
        )))?,
        tax_cap,
    ))
}

pub fn deduct_tax(
    deps: Deps,
    coin: Coin,
) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin).unwrap_or(Uint128::zero());
    Ok(Coin {
        denom: coin.denom,
        amount: (coin.amount.checked_sub(tax_amount))?,
    })
}

