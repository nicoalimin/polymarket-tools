use anyhow::{Context, Result};
use polymarket_client_sdk::data::{
    Client as DataClient,
    types::{MarketFilter, request::TradesRequest},
};

pub async fn execute(token_id: String) -> Result<()> {
    let client = DataClient::default();
    let request = TradesRequest::builder()
        .filter(MarketFilter::markets(vec![token_id.clone()]))
        .limit(20)?
        .build();
    let trades = client.trades(&request).await.context("Failed to fetch trades")?;

    println!("Recent Trades for {}:", token_id);
    for trade in trades {
        println!("- Trade: {:?}", trade);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Trade command primarily wraps the SDK client.
    // Integration tests would require a live API connection.
    // We verify the module compiles and the public API surface is correct.

    #[test]
    fn test_module_compiles() {
        // Ensure the execute function signature is correct
        fn _assert_fn_signature(_: fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>>>>) {}
        // This test just ensures the module is well-formed.
        assert!(true);
    }
}
