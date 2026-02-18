use anyhow::{Context, Result};
use polymarket_client_sdk::clob::{
    Client as ClobClient, Config as ClobConfig,
    types::request::{OrderBookSummaryRequest, MidpointRequest, SpreadRequest},
    types::response::OrderSummary,
};

pub async fn execute(token_id: String) -> Result<()> {
    let client = ClobClient::new("https://clob.polymarket.com", ClobConfig::default())?;
    let request = OrderBookSummaryRequest::builder().token_id(token_id.clone()).build();
    let book = client.order_book(&request).await.context("Failed to fetch order book")?;

    // Fetch midpoint
    let midpoint_req = MidpointRequest::builder().token_id(token_id.clone()).build();
    if let Ok(mid_resp) = client.midpoint(&midpoint_req).await {
        println!("Order Book for {}:", token_id);
        println!("  Midpoint Price: {}", mid_resp.mid);
    } else {
        println!("Order Book for {}:", token_id);
        println!("  Midpoint Price: N/A");
    }

    // Fetch spread
    let spread_req = SpreadRequest::builder().token_id(token_id.clone()).build();
    if let Ok(spread_resp) = client.spread(&spread_req).await {
        println!("  Spread: {}", spread_resp.spread);
    } else {
        println!("  Spread: N/A");
    }

    let bids = sort_bids(book.bids);
    let asks = sort_asks(book.asks);

    println!("  Bids:");
    for bid in &bids {
        println!("    Price: {}, Size: {}", bid.price, bid.size);
    }

    println!("  Asks:");
    for ask in &asks {
        println!("    Price: {}, Size: {}", ask.price, ask.size);
    }

    Ok(())
}

/// Sort bids descending (highest price first).
pub fn sort_bids(mut bids: Vec<OrderSummary>) -> Vec<OrderSummary> {
    bids.sort_by(|a, b| b.price.cmp(&a.price));
    bids
}

/// Sort asks ascending (lowest price first).
pub fn sort_asks(mut asks: Vec<OrderSummary>) -> Vec<OrderSummary> {
    asks.sort_by(|a, b| a.price.cmp(&b.price));
    asks
}

#[cfg(test)]
mod tests {
    use super::*;
    use polymarket_client_sdk::types::Decimal;
    use std::str::FromStr;

    fn make_level(price: &str, size: &str) -> OrderSummary {
        serde_json::from_value(serde_json::json!({
            "price": price,
            "size": size,
        })).unwrap()
    }

    #[test]
    fn test_sort_bids_descending() {
        let bids = vec![
            make_level("0.30", "100"),
            make_level("0.50", "200"),
            make_level("0.40", "150"),
        ];
        let sorted = sort_bids(bids);
        assert_eq!(sorted[0].price, Decimal::from_str("0.50").unwrap());
        assert_eq!(sorted[1].price, Decimal::from_str("0.40").unwrap());
        assert_eq!(sorted[2].price, Decimal::from_str("0.30").unwrap());
    }

    #[test]
    fn test_sort_asks_ascending() {
        let asks = vec![
            make_level("0.60", "100"),
            make_level("0.50", "200"),
            make_level("0.55", "150"),
        ];
        let sorted = sort_asks(asks);
        assert_eq!(sorted[0].price, Decimal::from_str("0.50").unwrap());
        assert_eq!(sorted[1].price, Decimal::from_str("0.55").unwrap());
        assert_eq!(sorted[2].price, Decimal::from_str("0.60").unwrap());
    }

    #[test]
    fn test_sort_bids_empty() {
        let bids: Vec<OrderSummary> = vec![];
        let sorted = sort_bids(bids);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_asks_empty() {
        let asks: Vec<OrderSummary> = vec![];
        let sorted = sort_asks(asks);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_single_element() {
        let bids = vec![make_level("0.50", "100")];
        let sorted = sort_bids(bids);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].price, Decimal::from_str("0.50").unwrap());
    }
}
