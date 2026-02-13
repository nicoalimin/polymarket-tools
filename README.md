# Polymarket CLI

A command-line interface for interacting with Polymarket, built with Rust.

## User Guide: VM & OpenClaw Setup

Follow these steps to set up the Polymarket CLI on a Virtual Machine with OpenClaw.

### 1. Get a VM

Provision a Linux Virtual Machine (e.g., Ubuntu) from your preferred provider (AWS, GCP, DigitalOcean, etc.).

### 2. Install OpenClaw

SSH into your VM and install OpenClaw using the onboard command:

```bash
openclaw onboard
```

Follow the prompts to complete the setup.

### 3. Download the CLI Binary

Download the latest release of the Polymarket CLI for Linux.
Run the following command (ensure you use the correct version, e.g., v0.3.0):

```bash
curl -L -o polymarket-cli https://github.com/nicoalimin/polymarket-tools/releases/download/v0.3.0/polymarket-cli-linux-amd64
chmod +x polymarket-cli
```

### 4. Configure OpenClaw Tools

Add the Polymarket CLI to your OpenClaw tools. Use the contents of the **AI Instructions** below as the tool description/documentation. This tells the AI how to use the CLI.

<details>
<summary><b>Tool Documentation (AI_README)</b></summary>

# Polymarket CLI for AI Agents

This document outlines the usage of the Polymarket CLI tool, designed for AI agents to interact with Polymarket data and execute trades.

## Overview

The tool is a Rust-based CLI that provides access to:

- Market Search
- User Portfolios/Positions
- Order Books
- Recent Trades
- Market Order Execution

## Usage

All commands are invoked via the binary. If running from source with valid environment variables setup:

```bash
cargo run -- <COMMAND> [ARGS]
```

Or if using a compiled binary:

```bash
./polymarket-cli <COMMAND> [ARGS]
```

## Commands

### 1. `search`

Search for markets by keyword.

- **Syntax**: `search <QUERY>`
- **Arguments**:
  - `QUERY`: String keyword to search for (e.g., "Trump", "Bitcoin").
- **Output**: List of events, markets, and outcomes with their Token IDs.
- **Example**:
  ```bash
  cargo run -- search "Bitcoin"
  ```
- **Sample Output**:
  ```text
  Found 2 events:
  Event: Bitcoin Price 2024 (ID: 12345)
    - Market: Will Bitcoin hit $100k in 2024? (ID: 67890)
      Outcomes:
        - Yes: 213... (Token ID)
        - No: 456... (Token ID)
  ```

### 2. `positions` (or `portfolios`)

View a user's open positions (portfolio).

- **Syntax**: `positions [--user <ADDRESS>]`
- **Arguments**:
  - `--user <ADDRESS>` (Optional): Ethereum address of the user. If omitted, defaults to `USER_ADDRESS` env var or derives from `PRIVATE_KEY` env var.
- **Output**: List of active positions including market title, outcome, size, average price, current value, and PnL.
- **Example**:
  ```bash
  cargo run -- positions --user 0x123...
  ```
- **Sample Output**:
  ```text
  Positions for 0x123...:
  - Market: Will Bitcoin hit $100k?
    Outcome: Yes
    Size: 10.5
    Avg Price: 0.45
    Current Value: $5.25
    PnL: $0.52 (11.1%)
  --------------------------------------------------
  ```

### 3. `order-book`

Fetch the order book for a specific outcome (Token ID).

- **Syntax**: `order-book --token-id <TOKEN_ID>`
- **Arguments**:
  - `--token-id <TOKEN_ID>`: The specific Token ID for the outcome (get this from `search`).
- **Output**: Midpoint price, spread, and a list of bids/asks.
- **Example**:
  ```bash
  cargo run -- order-book --token-id 213...
  ```
- **Sample Output**:
  ```text
  Order Book for 213...:
    Midpoint Price: 0.55
    Spread: 0.02
    Bids:
      Price: 0.54, Size: 100
      Price: 0.53, Size: 50
    Asks:
      Price: 0.56, Size: 200
  ```

### 4. `midpoint`

Quickly fetch just the midpoint price for a token.

- **Syntax**: `midpoint --token-id <TOKEN_ID>`
- **Arguments**:
  - `--token-id <TOKEN_ID>`: The Token ID.
- **Output**: The midpoint price.
- **Example**:
  ```bash
  cargo run -- midpoint --token-id 213...
  ```
- **Sample Output**:
  ```text
  Midpoint Price: 0.55
  ```

### 5. `trade` (Trade History)

View recent trades for a specific market/token.

- **Syntax**: `trade --token-id <TOKEN_ID>`
- **Arguments**:
  - `--token-id <TOKEN_ID>`: The Token ID.
- **Output**: List of recent trades.
- **Example**:
  ```bash
  cargo run -- trade --token-id 213...
  ```
- **Sample Output**:
  ```text
  Recent Trades for 213...:
  - Trade: Trade { price: 0.55, size: 100, side: Buy, ... }
  ```

### 6. `order`

Place a trade order. **REQUIRES `POLYMARKET_PRIVATE_KEY` ENV VAR.**

- **Syntax**: `order --token-id <ID> --side <SIDE> --amount <AMT> [--price <PRICE>]`
- **Arguments**:
  - `--token-id <ID>`: The Token ID of the outcome to trade.
  - `--side <SIDE>`: `buy` or `sell`.
  - `--amount <AMT>`: Amount of shares/contracts.
  - `--price <PRICE>` (Optional): Limit price. If omitted, places a Market Order (FOK).
- **Example (Limit Order)**:
  ```bash
  cargo run -- order --token-id 213... --side buy --amount 10 --price 0.55
  ```
- **Example (Market Order)**:
  ```bash
  cargo run -- order --token-id 213... --side buy --amount 10
  ```
- **Sample Output**:
  ```text
  Placing LIMIT Buy order: 10 tokens @ 0.55
  Limit Order Response: OrderResponse { ... }
  ```

## Error Handling

- **Invalid Token ID**: Returns "No orderbook exists..." or similar. Check `search` output for correct IDs.
- **Authentication**: `order` and default `positions` require `POLYMARKET_PRIVATE_KEY` in `.env`.
- **404**: Common for stale IDs or markets with no activity.

## Workflow Example for Agents

1.  **Discovery**: Agent runs `search "Topic"` to find relevant markets and Token IDs.
2.  **Analysis**: Agent runs `order-book` or `midpoint` on interesting Token IDs to check pricing.
3.  **Context**: Agent runs `positions` to check current inventory/exposure.
4.  **Action**: Agent runs `order` to execute a decision based on analysis and context.
</details>

### 5. Create .env File

Create a `.env` file in the same directory as the binary with your credentials.

```bash
nano .env
```

Add the following content (fill in your values):

```env
POLYMARKET_PRIVATE_KEY=your_private_key_here
USER_ADDRESS=your_wallet_address_here
# Optional: POLYGON_RPC_URL=...
```

### 6. Start Trading!

You can now start using the tool or let OpenClaw manage it.

```bash
./polymarket-cli search "Trump"
```

---

## Building from Source (Alternative)

1. Clone the repository.
2. Run `cargo build --release`.
3. The binary will be at `target/release/polymarket-cli`.

### Release Workflow

This project uses GitHub Actions to automatically build binaries for Linux (x86_64) and macOS (x86_64, ARM64) on tag push.

```bash
git tag v0.3.0
git push origin v0.3.0
```
