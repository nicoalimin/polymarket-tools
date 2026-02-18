mod cli;
mod commands;
mod constants;
mod contracts;

use clap::Parser;
use cli::{Cli, Commands};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => commands::search::execute(query).await,
        Commands::Positions { user } => commands::positions::execute(user).await,
        Commands::OrderBook { token_id } => commands::orderbook::execute(token_id).await,
        Commands::Trade { token_id } => commands::trade::execute(token_id).await,
        Commands::Midpoint { token_id } => commands::midpoint::execute(token_id).await,
        Commands::Order { token_id, side, amount, price } => {
            commands::order::execute(token_id, side, amount, price).await
        }
        Commands::Status => commands::status::execute().await,
        Commands::Approve { dry_run } => commands::approve::execute(dry_run).await,
    }
}
