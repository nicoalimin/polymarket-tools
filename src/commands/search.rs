use anyhow::{Context, Result};
use polymarket_client_sdk::gamma::{
    Client as GammaClient,
    types::request::SearchRequest,
};

pub async fn execute(query: String) -> Result<()> {
    let client = GammaClient::default();
    let search = SearchRequest::builder().q(query).build();
    let results = client.search(&search).await.context("Failed to search markets")?;

    if let Some(events) = results.events {
        println!("Found {} events:", events.len());
        for event in events {
            println!("Event: {} (ID: {})", event.title.unwrap_or_default(), event.id);
            if let Some(markets) = event.markets {
                for market in markets {
                    println!("  - Market: {} (ID: {})", market.question.unwrap_or_default(), market.id);

                    let outcomes_str = market.outcomes.unwrap_or_else(|| "[]".to_string());
                    let token_ids_str = market.clob_token_ids.unwrap_or_else(|| "[]".to_string());

                    let outcomes_list: Vec<String> = serde_json::from_str(&outcomes_str).unwrap_or_default();
                    let token_ids_list: Vec<String> = serde_json::from_str(&token_ids_str).unwrap_or_default();

                    if !outcomes_list.is_empty() && outcomes_list.len() == token_ids_list.len() {
                        println!("    Outcomes:");
                        for (outcome, token_id) in outcomes_list.iter().zip(token_ids_list.iter()) {
                            println!("      - {}: {}", outcome, token_id);
                        }
                    } else {
                        println!("    Outcomes (raw): {}", outcomes_str);
                        println!("    Token IDs (raw): {}", token_ids_str);
                    }
                }
            }
        }
    } else {
        println!("No events found.");
    }

    Ok(())
}

/// Parse outcomes and token IDs from their JSON string representations.
/// Returns paired (outcome, token_id) tuples if both lists parse and have equal length.
#[allow(dead_code)]
pub fn parse_outcomes(outcomes_str: &str, token_ids_str: &str) -> Option<Vec<(String, String)>> {
    let outcomes: Vec<String> = serde_json::from_str(outcomes_str).ok()?;
    let token_ids: Vec<String> = serde_json::from_str(token_ids_str).ok()?;

    if outcomes.is_empty() || outcomes.len() != token_ids.len() {
        return None;
    }

    Some(outcomes.into_iter().zip(token_ids.into_iter()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_outcomes_valid() {
        let outcomes = r#"["Yes","No"]"#;
        let token_ids = r#"["token_a","token_b"]"#;
        let result = parse_outcomes(outcomes, token_ids);
        assert!(result.is_some());
        let pairs = result.unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("Yes".to_string(), "token_a".to_string()));
        assert_eq!(pairs[1], ("No".to_string(), "token_b".to_string()));
    }

    #[test]
    fn test_parse_outcomes_empty_arrays() {
        let outcomes = "[]";
        let token_ids = "[]";
        let result = parse_outcomes(outcomes, token_ids);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_outcomes_mismatched_lengths() {
        let outcomes = r#"["Yes","No"]"#;
        let token_ids = r#"["token_a"]"#;
        let result = parse_outcomes(outcomes, token_ids);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_outcomes_invalid_json() {
        let outcomes = "not json";
        let token_ids = r#"["token_a"]"#;
        let result = parse_outcomes(outcomes, token_ids);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_outcomes_single_outcome() {
        let outcomes = r#"["Yes"]"#;
        let token_ids = r#"["token_x"]"#;
        let result = parse_outcomes(outcomes, token_ids);
        assert!(result.is_some());
        let pairs = result.unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("Yes".to_string(), "token_x".to_string()));
    }
}
