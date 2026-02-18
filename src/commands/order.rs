use anyhow::{Context, Result};
use polymarket_client_sdk::{
    POLYGON, PRIVATE_KEY_VAR,
    auth::{LocalSigner, Signer},
    clob::{
        Client as ClobClient, Config as ClobConfig,
        types::{Amount, OrderType, Side, SignatureType},
    },
    derive_safe_wallet, derive_proxy_wallet,
    types::Decimal,
};
use std::env;
use std::str::FromStr;

pub async fn execute(token_id: String, side: String, amount: String, price: Option<String>) -> Result<()> {
    let private_key = env::var(PRIVATE_KEY_VAR).context("Need PRIVATE_KEY environment variable")?;
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let safe_address = derive_safe_wallet(signer.address(), POLYGON);
    let proxy_address = derive_proxy_wallet(signer.address(), POLYGON);

    println!("Safe Address: {:?}", safe_address);
    println!("Proxy Address: {:?}", proxy_address);

    let client = ClobClient::new("https://clob.polymarket.com", ClobConfig::default())?
        .authentication_builder(&signer)
        .signature_type(SignatureType::Proxy)
        .authenticate()
        .await
        .context("Failed to authenticate")?;

    let ok = client.ok().await?;
    println!("Ok: {ok}");

    let api_keys = client.api_keys().await?;
    println!("API keys: {api_keys:?}");

    let side_enum = parse_side(&side)?;
    let amount_dec = Decimal::from_str(&amount).context("Invalid amount")?;

    if let Some(p) = price {
        let price_dec = Decimal::from_str(&p).context("Invalid price")?;
        let order_amount = compute_order_amount(side_enum, amount_dec, Some(price_dec))?;

        let order = client
            .market_order()
            .token_id(token_id)
            .amount(order_amount)
            .side(side_enum)
            .order_type(OrderType::FOK)
            .build()
            .await
            .context("Failed to build market order")?;

        let signed_order = client.sign(&signer, order).await.context("Failed to sign order")?;
        let response = client.post_order(signed_order).await.context("Failed to post order")?;
        println!("Limit Order Response: {:?}", response);
    } else {
        let order_amount = compute_order_amount(side_enum, amount_dec, None)?;

        let order = client
            .market_order()
            .token_id(token_id)
            .amount(order_amount)
            .side(side_enum)
            .order_type(OrderType::FOK)
            .build()
            .await
            .context("Failed to build market order")?;

        let signed_order = client.sign(&signer, order).await.context("Failed to sign order")?;
        let response = client.post_order(signed_order).await.context("Failed to post order")?;
        println!("Market Order Response: {:?}", response);
    }

    Ok(())
}

/// Parse a side string ("buy" or "sell") into the Side enum.
pub fn parse_side(side: &str) -> Result<Side> {
    match side.to_lowercase().as_str() {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => anyhow::bail!("Invalid side: must be 'buy' or 'sell'"),
    }
}

/// Compute the order amount based on side, amount, and optional price.
/// For buys with a price, computes USDC value = amount * price (the amount represents shares).
/// For sells, always uses share amount.
/// For buys without a price, uses USDC amount directly.
pub fn compute_order_amount(side: Side, amount: Decimal, price: Option<Decimal>) -> Result<Amount> {
    let rounded_amount = amount.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero);

    match (side, price) {
        (Side::Buy, Some(price_dec)) => {
            let usdc_value = rounded_amount * price_dec;
            println!("Placing MARKET Buy order (derived from limit params): {} USDC value (from {} shares)", usdc_value, rounded_amount);
            Amount::usdc(usdc_value).context("Invalid USDC amount")
        }
        (Side::Buy, None) => {
            println!("Placing MARKET Buy order: {} USDC (from {})", rounded_amount, amount);
            Amount::usdc(rounded_amount).context("Invalid USDC amount")
        }
        (Side::Sell, _) => {
            println!("Placing MARKET Sell order: {} Shares (from {})", rounded_amount, amount);
            Amount::shares(rounded_amount).context("Invalid Share amount")
        }
        _ => unreachable!("Side is always Buy or Sell"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_side_buy() {
        assert!(matches!(parse_side("buy"), Ok(Side::Buy)));
        assert!(matches!(parse_side("BUY"), Ok(Side::Buy)));
        assert!(matches!(parse_side("Buy"), Ok(Side::Buy)));
    }

    #[test]
    fn test_parse_side_sell() {
        assert!(matches!(parse_side("sell"), Ok(Side::Sell)));
        assert!(matches!(parse_side("SELL"), Ok(Side::Sell)));
        assert!(matches!(parse_side("Sell"), Ok(Side::Sell)));
    }

    #[test]
    fn test_parse_side_invalid() {
        assert!(parse_side("hold").is_err());
        assert!(parse_side("").is_err());
        assert!(parse_side("b").is_err());
    }

    #[test]
    fn test_compute_order_amount_buy_market() {
        let amount = Decimal::from_str("10.50").unwrap();
        let result = compute_order_amount(Side::Buy, amount, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_order_amount_buy_with_price() {
        let amount = Decimal::from_str("100.00").unwrap();
        let price = Decimal::from_str("0.65").unwrap();
        let result = compute_order_amount(Side::Buy, amount, Some(price));
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_order_amount_sell() {
        let amount = Decimal::from_str("50.00").unwrap();
        let result = compute_order_amount(Side::Sell, amount, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_order_amount_sell_with_price_ignored() {
        let amount = Decimal::from_str("50.00").unwrap();
        let price = Decimal::from_str("0.70").unwrap();
        // Price is ignored for sell orders — shares amount is used
        let result = compute_order_amount(Side::Sell, amount, Some(price));
        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_order_amount_rounding() {
        // 10.999 should truncate to 10.99
        let amount = Decimal::from_str("10.999").unwrap();
        let result = compute_order_amount(Side::Buy, amount, None);
        assert!(result.is_ok());
    }
}
