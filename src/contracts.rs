use alloy::primitives::U256;
use alloy::sol;
use anyhow::Result;
use polymarket_client_sdk::types::Address;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }
}

pub fn new_erc20<P: alloy::providers::Provider + Clone>(address: Address, provider: P) -> IERC20::IERC20Instance<P> {
    IERC20::new(address, provider)
}

pub fn new_erc1155<P: alloy::providers::Provider + Clone>(address: Address, provider: P) -> IERC1155::IERC1155Instance<P> {
    IERC1155::new(address, provider)
}

pub async fn check_allowance<P: alloy::providers::Provider>(
    token: &IERC20::IERC20Instance<P>,
    owner: Address,
    spender: Address,
) -> Result<U256> {
    let allowance = token.allowance(owner, spender).call().await?;
    Ok(allowance)
}

pub async fn check_balance<P: alloy::providers::Provider>(
    token: &IERC20::IERC20Instance<P>,
    account: Address,
) -> Result<U256> {
    let balance = token.balanceOf(account).call().await?;
    Ok(balance)
}

pub async fn check_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    account: Address,
    operator: Address,
) -> Result<bool> {
    let approved = ctf.isApprovedForAll(account, operator).call().await?;
    Ok(approved)
}

pub async fn approve_token<P: alloy::providers::Provider>(
    usdc: &IERC20::IERC20Instance<P>,
    spender: Address,
    amount: U256,
) -> Result<alloy::primitives::FixedBytes<32>> {
    let tx_hash = usdc.approve(spender, amount).send().await?.watch().await?;
    Ok(tx_hash)
}

pub async fn set_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    operator: Address,
    approved: bool,
) -> Result<alloy::primitives::FixedBytes<32>> {
    let tx_hash = ctf
        .setApprovalForAll(operator, approved)
        .send()
        .await?
        .watch()
        .await?;
    Ok(tx_hash)
}
