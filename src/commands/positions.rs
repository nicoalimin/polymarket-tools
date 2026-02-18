use anyhow::{Context, Result};
use polymarket_client_sdk::{
    PRIVATE_KEY_VAR,
    auth::LocalSigner,
    data::{
        Client as DataClient,
        types::request::PositionsRequest,
    },
    types::Address,
};
use std::env;
use std::str::FromStr;

pub async fn execute(user: Option<String>) -> Result<()> {
    let user_addr = resolve_user_address(user)?;

    let client = DataClient::default();
    let request = PositionsRequest::builder().user(user_addr).limit(50)?.build();
    let positions = client.positions(&request).await.context("Failed to fetch positions")?;

    println!("Positions for {}:", user_addr);
    for pos in positions {
        println!("- Market: {}", pos.title);
        println!("  Token ID: {}", pos.asset);
        println!("  Outcome: {}", pos.outcome);
        println!("  Size: {}", pos.size);
        println!("  Avg Price: {}", pos.avg_price);
        println!("  Current Value: ${}", pos.current_value);
        println!("  PnL: ${} ({}%)", pos.cash_pnl, pos.percent_pnl);
        println!("--------------------------------------------------");
    }

    Ok(())
}

/// Resolve the user address from an explicit argument, env var, or private key derivation.
pub fn resolve_user_address(user: Option<String>) -> Result<Address> {
    if let Some(u) = user {
        return Address::from_str(&u).context("Invalid address format");
    }
    if let Ok(u) = env::var("USER_ADDRESS") {
        return Address::from_str(&u).context("Invalid address format in USER_ADDRESS");
    }
    let private_key = env::var(PRIVATE_KEY_VAR).context("PRIVATE_KEY or USER_ADDRESS env var not set")?;
    let signer = LocalSigner::from_str(&private_key).context("Invalid private key")?;
    Ok(signer.address())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_user_address_explicit_valid() {
        let addr = "0x76564A875522c78263B7c0c51B3760A1776877af".to_string();
        let result = resolve_user_address(Some(addr.clone()));
        assert!(result.is_ok());
        assert_eq!(format!("{}", result.unwrap()), addr);
    }

    #[test]
    fn test_resolve_user_address_explicit_invalid() {
        let result = resolve_user_address(Some("not_an_address".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_user_address_none_no_env() {
        // Clear both env vars to ensure we get an error
        env::remove_var("USER_ADDRESS");
        env::remove_var(PRIVATE_KEY_VAR);
        let result = resolve_user_address(None);
        assert!(result.is_err());
    }
}
