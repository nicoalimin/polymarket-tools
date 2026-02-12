# Polymarket CLI

A command-line interface for interacting with Polymarket, built with Rust.

## Features

- **Search Markets**: Find markets by keyword.
- **View Positions**: See your open positions and their values.
- **Order Book**: View the order book for a specific market outcome.
- **Trades**: View recent trades for a specific market outcome.
- **Trade**: Place Limit or Market orders.

## Installation

### From Source
1. Clone the repository.
2. Run `cargo build --release`.
3. The binary will be at `target/release/polymarket-cli`.

### Binaries
Download the latest binary for your platform from the [Releases](https://github.com/yourusername/luminescent-protostar/releases) page.

## Configuration

Create a `.env` file in the same directory as the binary or set environment variables:

```env
PRIVATE_KEY=your_private_key_here
# Optional: POLYGON_RPC_URL=...
```

## Usage

```bash
# Search for markets
./polymarket-cli search "Trump"

# View your positions
./polymarket-cli positions
# View another user's positions
./polymarket-cli positions --user 0x...

# View Order Book
./polymarket-cli order-book --token-id <TOKEN_ID>

# View Midpoint Price
./polymarket-cli midpoint --token-id <TOKEN_ID>

# View Recent Trades
./polymarket-cli trade --token-id <TOKEN_ID>

# Place a Limit Order (Buy 100 shares of TokenID at $0.50)
./polymarket-cli order --side buy --token-id <TOKEN_ID> --amount 100 --price 0.50

# Place a Market Order (Buy $50 worth of TokenID)
./polymarket-cli order --side buy --token-id <TOKEN_ID> --amount 50
```

## Building for Release

This project uses GitHub Actions to automatically build binaries for Linux (x86_64) and macOS (x86_64, ARM64) on tag push.

To trigger a release:
```bash
git tag v0.1.0
git push origin v0.1.0
```
