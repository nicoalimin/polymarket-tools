use anyhow::{Context, Result};
use alloy::providers::ProviderBuilder;
use polymarket_client_sdk::{
    POLYGON, PRIVATE_KEY_VAR,
    auth::{LocalSigner, Signer},
    derive_proxy_wallet,
    types::{Address, Decimal},
};
use std::env;
use std::str::FromStr;

use crate::constants::{RPC_URL, USDC_E_ADDRESS, USDC_NATIVE_ADDRESS};
use crate::contracts::{new_erc20, check_balance};

pub async fn execute() -> Result<()> {
    let private_key = env::var(PRIVATE_KEY_VAR).context("Need PRIVATE_KEY environment variable")?;
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));
    let owner = signer.address();
    println!("User Address: {}", owner);

    let proxy_address = derive_proxy_wallet(owner, POLYGON).context("Failed to derive proxy wallet")?;
    println!("Proxy Address: {}", proxy_address);

    let provider = ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(RPC_URL)
        .await?;

    let tokens = [
        ("USDC.e", new_erc20(USDC_E_ADDRESS, provider.clone())),
        ("USDC (Native)", new_erc20(USDC_NATIVE_ADDRESS, provider.clone())),
    ];

    for (name, token) in &tokens {
        let balance = check_balance(token, proxy_address).await?;
        let balance_fmt = format_balance(balance);
        println!("{}: ${}", name, balance_fmt);
    }

    Ok(())
}

/// Format a raw token balance (with 6 decimals) into a human-readable decimal string.
pub fn format_balance(raw_balance: alloy::primitives::U256) -> Decimal {
    let balance_dec = Decimal::from_str(&raw_balance.to_string()).unwrap_or_default();
    balance_dec / Decimal::from(1_000_000)
}

/// Get the list of token names and addresses to query.
#[allow(dead_code)]
pub fn token_list() -> Vec<(&'static str, Address)> {
    vec![
        ("USDC.e", USDC_E_ADDRESS),
        ("USDC (Native)", USDC_NATIVE_ADDRESS),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_format_balance_zero() {
        let balance = U256::from(0u64);
        let formatted = format_balance(balance);
        assert_eq!(formatted, Decimal::from(0));
    }

    #[test]
    fn test_format_balance_one_usdc() {
        // 1 USDC = 1_000_000 raw units
        let balance = U256::from(1_000_000u64);
        let formatted = format_balance(balance);
        assert_eq!(formatted, Decimal::from(1));
    }

    #[test]
    fn test_format_balance_fractional() {
        // 4.281842 USDC = 4_281_842 raw units
        let balance = U256::from(4_281_842u64);
        let formatted = format_balance(balance);
        assert_eq!(formatted, Decimal::from_str("4.281842").unwrap());
    }

    #[test]
    fn test_format_balance_large_amount() {
        // 1000 USDC
        let balance = U256::from(1_000_000_000u64);
        let formatted = format_balance(balance);
        assert_eq!(formatted, Decimal::from(1000));
    }

    #[test]
    fn test_format_balance_small_dust() {
        // 0.000001 USDC = 1 raw unit
        let balance = U256::from(1u64);
        let formatted = format_balance(balance);
        assert_eq!(formatted, Decimal::from_str("0.000001").unwrap());
    }

    #[test]
    fn test_token_list_has_two_entries() {
        let tokens = token_list();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].0, "USDC.e");
        assert_eq!(tokens[1].0, "USDC (Native)");
    }

    #[test]
    fn test_token_list_addresses_are_correct() {
        let tokens = token_list();
        assert_eq!(tokens[0].1, USDC_E_ADDRESS);
        assert_eq!(tokens[1].1, USDC_NATIVE_ADDRESS);
    }
}
