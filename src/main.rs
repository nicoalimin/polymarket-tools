use clap::{Parser, Subcommand};
use dotenv::dotenv;
use polymarket_client_sdk::{
    clob::{
        Client as ClobClient, Config as ClobConfig,
        types::{Amount, OrderType, Side, request::{OrderBookSummaryRequest, MidpointRequest, SpreadRequest}},
    },
    data::{
        Client as DataClient,
        types::{MarketFilter, request::{PositionsRequest, TradesRequest}},
    },
    gamma::{
        Client as GammaClient,
        types::request::SearchRequest,
    },
    types::{Decimal, Address},
    auth::{LocalSigner, Signer},
    POLYGON, PRIVATE_KEY_VAR,
};
use std::str::FromStr;
use std::env;
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(name = "polymarket-cli")]
#[command(about = "CLI for Polymarket", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for markets by keyword
    Search {
        /// Keywords to search for
        query: String,
    },
    /// See open positions
    Positions {
        /// Optional user address. If not provided, tries to derive from private key.
        #[arg(short, long)]
        user: Option<String>,
    },
    /// See order book for a market
    OrderBook {
        /// Token ID to fetch order book for
        #[arg(short, long)]
        token_id: String,
    },
    /// See recent trades for a market (Trade history)
    Trade {
        /// Market ID (or Token ID) to fetch trades for
        #[arg(short, long)]
        token_id: String,
    },
    /// Get the Midpoint Price for a market
    Midpoint {
        /// Token ID to fetch midpoint for
        #[arg(short, long)]
        token_id: String,
    },
    /// Place an order
    Order {
        /// Token ID of the outcome
        #[arg(short, long)]
        token_id: String,

        /// Side to trade: "buy" or "sell"
        #[arg(short, long)]
        side: String,

        /// Amount (size) of the order
        #[arg(short, long)]
        amount: String,

        /// Price for limit order. If omitted, places a Market Order (FOK).
        #[arg(short, long)]
        price: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
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
                            
                            // Parse outcomes and token_ids
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
        }
        Commands::Positions { user } => {
            let user_addr = if let Some(u) = user {
                Address::from_str(&u).context("Invalid address format")?
            } else if let Ok(u) = env::var("USER_ADDRESS") {
                Address::from_str(&u).context("Invalid address format in USER_ADDRESS")?
            } else {
                let private_key = env::var(PRIVATE_KEY_VAR).context("PRIVATE_KEY or USER_ADDRESS env var not set")?;
                let signer = LocalSigner::from_str(&private_key).context("Invalid private key")?;
                signer.address()
            };

            let client = DataClient::default();
            let request = PositionsRequest::builder().user(user_addr).limit(50)?.build();
            let positions = client.positions(&request).await.context("Failed to fetch positions")?;

            println!("Positions for {}:", user_addr);
            for pos in positions {
                println!("- Market: {}", pos.title);
                println!("  Outcome: {}", pos.outcome);
                println!("  Size: {}", pos.size);
                println!("  Avg Price: {}", pos.avg_price);
                println!("  Current Value: ${}", pos.current_value);
                println!("  PnL: ${} ({}%)", pos.cash_pnl, pos.percent_pnl);
                println!("--------------------------------------------------");
            }
        }
        Commands::OrderBook { token_id } => {
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
            
            println!("  Bids:");
            // Sort bids descending (highest first) for display
            let mut bids = book.bids;
            bids.sort_by(|a, b| b.price.cmp(&a.price));
            for bid in bids {
                println!("    Price: {}, Size: {}", bid.price, bid.size);
            }

            println!("  Asks:");
            // Sort asks ascending (lowest first) for display
            let mut asks = book.asks;
            asks.sort_by(|a, b| a.price.cmp(&b.price));
            for ask in asks {
                println!("    Price: {}, Size: {}", ask.price, ask.size);
            }
        }
        Commands::Midpoint { token_id } => {
            let client = ClobClient::new("https://clob.polymarket.com", ClobConfig::default())?;
            let request = MidpointRequest::builder().token_id(token_id).build();
            let response = client.midpoint(&request).await.context("Failed to fetch midpoint")?;
            println!("Midpoint Price: {}", response.mid);
        }
        Commands::Trade { token_id } => {
            let client = DataClient::default();
            // Using MarketFilter to filter trades by market (token_id/condition_id)
            let request = TradesRequest::builder()
                .filter(MarketFilter::markets(vec![token_id.clone()]))
                .limit(20)?
                .build();
            let trades = client.trades(&request).await.context("Failed to fetch trades")?;
            
            println!("Recent Trades for {}:", token_id);
            for trade in trades {
                println!("- Trade: {:?}", trade);
            }
        }
        Commands::Order { token_id, side, amount, price } => {
            let private_key = env::var(PRIVATE_KEY_VAR).context("Need PRIVATE_KEY environment variable")?;
            let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));
            
            let client = ClobClient::new("https://clob.polymarket.com", ClobConfig::default())?
                .authentication_builder(&signer)
                .authenticate()
                .await
                .context("Failed to authenticate")?;

            let side_enum = match side.to_lowercase().as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                _ => anyhow::bail!("Invalid side: must be 'buy' or 'sell'"),
            };

            let amount_dec = Decimal::from_str(&amount).context("Invalid amount")?;

            if let Some(p) = price {
                // Limit Order
                let price_dec = Decimal::from_str(&p).context("Invalid price")?;
                println!("Placing LIMIT {:?} order: {} tokens @ {}", side_enum, amount_dec, price_dec);

                let order = client
                    .limit_order()
                    .token_id(token_id)
                    .size(amount_dec)
                    .price(price_dec)
                    .side(side_enum)
                    .build()
                    .await
                    .context("Failed to build limit order")?;

                let signed_order = client.sign(&signer, order).await.context("Failed to sign order")?;
                let response = client.post_order(signed_order).await.context("Failed to post order")?;
                println!("Limit Order Response: {:?}", response);

            } else {
                // Market Order
                println!("Placing MARKET {:?} order: {} (USDC value approx)", side_enum, amount_dec);
                 
                 let order_amount = Amount::usdc(amount_dec)?;

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
        }
    }

    Ok(())
}
