use anyhow::{Context, Result};
use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
use polymarket_client_sdk::{
    POLYGON, PRIVATE_KEY_VAR,
    auth::{LocalSigner, Signer},
    contract_config,
    types::Address,
};
use std::env;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

use crate::constants::{RPC_URL, USDC_E_ADDRESS, USDC_NATIVE_ADDRESS};
use crate::contracts::{
    new_erc20, new_erc1155,
    check_allowance, check_approval_for_all,
    approve_token, set_approval_for_all,
};

pub async fn execute(dry_run: bool) -> Result<()> {
    let chain = POLYGON;
    let targets = build_approval_targets(chain)?;

    if dry_run {
        println!("mode = \"dry_run\", showing approvals without executing");
        for (name, target) in &targets {
            println!("contract = {}, address = {}, would receive approval", name, target);
        }
        println!("total = {}, contracts would be approved", targets.len());
        return Ok(());
    }

    let private_key = env::var(PRIVATE_KEY_VAR).context("Need PRIVATE_KEY environment variable")?;
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(chain));

    let provider = ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(RPC_URL)
        .await?;

    let owner = signer.address();
    println!("wallet loaded: {}", owner);

    let config = contract_config(chain, false).unwrap();
    let ctf = new_erc1155(config.conditional_tokens, provider.clone());

    println!("phase = \"checking\", querying current allowances");

    for (name, target) in &targets {
        let tokens = [
            ("USDC.e", new_erc20(USDC_E_ADDRESS, provider.clone())),
            ("USDC (Native)", new_erc20(USDC_NATIVE_ADDRESS, provider.clone())),
        ];

        for (token_name, token_contract) in &tokens {
            match check_allowance(token_contract, owner, *target).await {
                Ok(allowance) => println!("contract = {}, token = {}, allowance = {}", name, token_name, allowance),
                Err(e) => eprintln!("contract = {}, token = {}, error = {:?}, failed to check allowance", name, token_name, e),
            }
        }

        match check_approval_for_all(&ctf, owner, *target).await {
            Ok(approved) => println!("contract = {}, ctf_approved = {}", name, approved),
            Err(e) => eprintln!("contract = {}, error = {:?}, failed to check CTF approval", name, e),
        }
    }

    println!("phase = \"approving\", setting approvals");

    for (name, target) in &targets {
        println!("contract = {}, address = {}, approving", name, target);

        println!("Waiting 10s...");
        sleep(Duration::from_secs(10)).await;

        let tokens = [
            ("USDC.e", new_erc20(USDC_E_ADDRESS, provider.clone())),
            ("USDC (Native)", new_erc20(USDC_NATIVE_ADDRESS, provider.clone())),
        ];

        for (token_name, token_contract) in &tokens {
            match approve_token(token_contract, *target, U256::MAX).await {
                Ok(tx_hash) => println!("contract = {}, token = {}, tx = {}, approved", name, token_name, tx_hash),
                Err(e) => eprintln!("contract = {}, token = {}, error = {:?}, approve failed", name, token_name, e),
            }
            println!("Waiting 10s...");
            sleep(Duration::from_secs(10)).await;
        }

        println!("Waiting 10s...");
        sleep(Duration::from_secs(10)).await;

        match set_approval_for_all(&ctf, *target, true).await {
            Ok(tx_hash) => println!("contract = {}, tx = {}, CTF approved", name, tx_hash),
            Err(e) => eprintln!("contract = {}, error = {:?}, CTF setApprovalForAll failed", name, e),
        }
    }

    println!("phase = \"verifying\", confirming approvals");

    for (name, target) in &targets {
        let tokens = [
            ("USDC.e", new_erc20(USDC_E_ADDRESS, provider.clone())),
            ("USDC (Native)", new_erc20(USDC_NATIVE_ADDRESS, provider.clone())),
        ];

        for (token_name, token_contract) in &tokens {
            match check_allowance(token_contract, owner, *target).await {
                Ok(allowance) => println!("contract = {}, token = {}, allowance = {}, verified", name, token_name, allowance),
                Err(e) => eprintln!("contract = {}, token = {}, error = {:?}, verification failed", name, token_name, e),
            }
        }

        match check_approval_for_all(&ctf, owner, *target).await {
            Ok(approved) => println!("contract = {}, ctf_approved = {}, verified", name, approved),
            Err(e) => eprintln!("contract = {}, error = {:?}, verification failed", name, e),
        }
    }

    println!("all approvals complete");

    Ok(())
}

/// Build the list of contracts that need token approvals.
pub fn build_approval_targets(chain: u64) -> Result<Vec<(&'static str, Address)>> {
    let config = contract_config(chain, false).context("Failed to get contract config")?;
    let neg_risk_config = contract_config(chain, true).context("Failed to get neg risk contract config")?;

    let mut targets: Vec<(&str, Address)> = vec![
        ("CTF Exchange", config.exchange),
        ("Neg Risk CTF Exchange", neg_risk_config.exchange),
    ];

    if let Some(adapter) = neg_risk_config.neg_risk_adapter {
        targets.push(("Neg Risk Adapter", adapter));
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polymarket_client_sdk::POLYGON;

    #[test]
    fn test_build_approval_targets_polygon() {
        let targets = build_approval_targets(POLYGON);
        assert!(targets.is_ok());
        let targets = targets.unwrap();
        // Should have at least CTF Exchange and Neg Risk CTF Exchange
        assert!(targets.len() >= 2);
        assert_eq!(targets[0].0, "CTF Exchange");
        assert_eq!(targets[1].0, "Neg Risk CTF Exchange");
    }

    #[test]
    fn test_build_approval_targets_has_neg_risk_adapter() {
        let targets = build_approval_targets(POLYGON).unwrap();
        let has_adapter = targets.iter().any(|(name, _)| *name == "Neg Risk Adapter");
        assert!(has_adapter, "Expected Neg Risk Adapter in approval targets for Polygon");
    }

    #[test]
    fn test_build_approval_targets_addresses_non_zero() {
        let targets = build_approval_targets(POLYGON).unwrap();
        for (name, addr) in &targets {
            assert_ne!(*addr, Address::ZERO, "{} should have a non-zero address", name);
        }
    }
}
