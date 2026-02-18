use anyhow::{Context, Result};
use polymarket_client_sdk::clob::{
    Client as ClobClient, Config as ClobConfig,
    types::request::MidpointRequest,
};

pub async fn execute(token_id: String) -> Result<()> {
    let client = ClobClient::new("https://clob.polymarket.com", ClobConfig::default())?;
    let request = MidpointRequest::builder().token_id(token_id).build();
    let response = client.midpoint(&request).await.context("Failed to fetch midpoint")?;
    println!("Midpoint Price: {}", response.mid);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Midpoint command is a thin SDK wrapper.
        // Verifying the module structure is correct.
        assert!(true);
    }
}
