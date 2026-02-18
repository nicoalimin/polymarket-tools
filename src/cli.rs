use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "polymarket-cli")]
#[command(about = "CLI for Polymarket", long_about = None)]
#[command(version = include_str!("../version.txt").trim_ascii())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Check current status (available cash)
    Status,
    /// Upgrade the CLI to the latest version
    Upgrade,
}
