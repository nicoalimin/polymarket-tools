use clap::{Parser, Subcommand};
use dotenv::dotenv;
use polymarket_client_sdk::{
    POLYGON, PRIVATE_KEY_VAR, auth::{LocalSigner, Signer}, clob::{
        Client as ClobClient, Config as ClobConfig,
        types::{Amount, OrderType, Side, SignatureType, request::{MidpointRequest, OrderBookSummaryRequest, SpreadRequest}},
    }, contract_config, data::{
        Client as DataClient,
        types::{MarketFilter, request::{PositionsRequest, TradesRequest}},
    }, gamma::{
        Client as GammaClient,
        types::request::SearchRequest,
    }, types::{Address, Decimal, address}
};
use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
// use alloy::signers::Signer as _; // Removed unused import
use alloy::sol;

const RPC_URL: &str = "https://polygon-rpc.com";

const USDC_E_ADDRESS: Address = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");
const USDC_NATIVE_ADDRESS: Address = address!("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359");

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }
}
use std::str::FromStr;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
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
    /// Approve tokens for trading
    Approve {
        /// Dry run mode (don't execute transactions)
        #[arg(long, default_value_t = false)]
        dry_run: bool,
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

            use polymarket_client_sdk::{derive_safe_wallet, derive_proxy_wallet, POLYGON};

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

            let side_enum = match side.to_lowercase().as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                _ => anyhow::bail!("Invalid side: must be 'buy' or 'sell'"),
            };

            let amount_dec = Decimal::from_str(&amount).context("Invalid amount")?;

            if let Some(p) = price {
                // Limit Order inputs converted to Market Order (FOK)
                let price_dec = Decimal::from_str(&p).context("Invalid price")?;
                
                let order_amount = match side_enum {
                    Side::Buy => {
                        let rounded_amount = amount_dec.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero);
                        let usdc_value = rounded_amount * price_dec;
                        println!("Placing MARKET Buy order (derived from limit params): {} USDC value (from {} shares)", usdc_value, rounded_amount);
                        Amount::usdc(usdc_value).context("Invalid USDC amount")?
                    }
                    Side::Sell => {
                        let rounded_amount = amount_dec.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero);
                        println!("Placing MARKET Sell order: {} Shares (from {})", rounded_amount, amount_dec);
                        Amount::shares(rounded_amount).context("Invalid Share amount")?
                    },
                    _ => unreachable!("Side derived from buy/sell string"),
                };

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
                // Market Order
                let order_amount = match side_enum {
                    Side::Buy => {
                        let rounded_amount = amount_dec.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero);
                        println!("Placing MARKET Buy order: {} USDC (from {})", rounded_amount, amount_dec);
                        Amount::usdc(rounded_amount).context("Invalid USDC amount")?
                    }
                    Side::Sell => {
                        let rounded_amount = amount_dec.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero);
                        println!("Placing MARKET Sell order: {} Shares (from {})", rounded_amount, amount_dec);
                        Amount::shares(rounded_amount).context("Invalid Share amount")?
                    },
                    _ => unreachable!("Side derived from buy/sell string"),
                };

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
        Commands::Approve { dry_run } => {
            let chain = POLYGON;
            let config = contract_config(chain, false).unwrap();
            let neg_risk_config = contract_config(chain, true).unwrap();

            // Collect all contracts that need approval
            let mut targets: Vec<(&str, Address)> = vec![
                ("CTF Exchange", config.exchange),
                ("Neg Risk CTF Exchange", neg_risk_config.exchange),
            ];

            // Add the Neg Risk Adapter if available
            if let Some(adapter) = neg_risk_config.neg_risk_adapter {
                targets.push(("Neg Risk Adapter", adapter));
            }

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

            let ctf = IERC1155::new(config.conditional_tokens, provider.clone());

            println!("phase = \"checking\", querying current allowances");

            for (name, target) in &targets {
                // Check allowances for both tokens
                let tokens = [
                    ("USDC.e", IERC20::new(USDC_E_ADDRESS, provider.clone())),
                    ("USDC (Native)", IERC20::new(USDC_NATIVE_ADDRESS, provider.clone())),
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

                // Approve both USDC versions
                let tokens = [
                    ("USDC.e", IERC20::new(USDC_E_ADDRESS, provider.clone())),
                    ("USDC (Native)", IERC20::new(USDC_NATIVE_ADDRESS, provider.clone())),
                ];

                for (token_name, token_contract) in &tokens {
                    match approve(token_contract, *target, U256::MAX).await {
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
                    ("USDC.e", IERC20::new(USDC_E_ADDRESS, provider.clone())),
                    ("USDC (Native)", IERC20::new(USDC_NATIVE_ADDRESS, provider.clone())),
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
        }
    }

    Ok(())
}

async fn check_allowance<P: alloy::providers::Provider>(
    token: &IERC20::IERC20Instance<P>,
    owner: Address,
    spender: Address,
) -> anyhow::Result<U256> {
    let allowance = token.allowance(owner, spender).call().await?;
    Ok(allowance)
}

async fn check_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    account: Address,
    operator: Address,
) -> anyhow::Result<bool> {
    let approved = ctf.isApprovedForAll(account, operator).call().await?;
    Ok(approved)
}

async fn approve<P: alloy::providers::Provider>(
    usdc: &IERC20::IERC20Instance<P>,
    spender: Address,
    amount: U256,
) -> anyhow::Result<alloy::primitives::FixedBytes<32>> {
    let tx_hash = usdc.approve(spender, amount).send().await?.watch().await?;
    Ok(tx_hash)
}

async fn set_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    operator: Address,
    approved: bool,
) -> anyhow::Result<alloy::primitives::FixedBytes<32>> {
    let tx_hash = ctf
        .setApprovalForAll(operator, approved)
        .send()
        .await?
        .watch()
        .await?;
    Ok(tx_hash)
}
