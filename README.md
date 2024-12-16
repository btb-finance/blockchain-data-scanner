# Uniswap V3 NFT Holders Scanner

This Rust application scans the Ethereum blockchain to identify and analyze holders of Uniswap V3 NFT positions.

## Features

- Scans all Uniswap V3 NFT position tokens
- Tracks holder addresses and their token counts
- Saves results in both JSON and text formats
- Provides total supply and unique holder statistics

## Prerequisites

- Rust and Cargo installed
- Access to an Ethereum node (via Infura, Alchemy, or other providers)

## Setup

1. Clone the repository
2. Copy `env.example` to `.env` and add your Ethereum node URL:
   ```
   ETHEREUM_RPC_URL=your_ethereum_node_url
   ```
3. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the scanner:
```
cargo run --release
```

The results will be saved in:
- `data/state.json`: Complete scan results in JSON format
- `data/uniswap_v3_holders.txt`: Simple text file with addresses and token counts

## License

MIT
