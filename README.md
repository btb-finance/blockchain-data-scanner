# Blockchain Data Scanner

A Rust-based tool for scanning and analyzing blockchain data, with a focus on NFT holder data from the Uniswap V3 contract.

## Features

- Fetches NFT holder data from Alchemy API
- Supports pagination for large datasets
- Saves progress and can resume from last checkpoint
- Stores unique holder addresses
- Handles rate limiting and timeouts gracefully

## Prerequisites

- Rust (latest stable version)
- An Alchemy API key

## Setup

1. Clone the repository:
```bash
git clone https://github.com/btb-finance/blockchain-data-scanner.git
cd blockchain-data-scanner
```

2. Create a `.env` file with your Alchemy API key:
```bash
ALCHEMY_API_KEY=your_api_key_here
```

3. Build the project:
```bash
cargo build --release
```

## Usage

Run the scanner:
```bash
cargo run --release
```

The scanner will:
1. Fetch NFT holder data from the Uniswap V3 contract
2. Save unique holder addresses to `data/uniswap_v3_holders.txt`
3. Save progress state to `data/state.json`
4. Resume from last saved state if interrupted

## Output

- `data/uniswap_v3_holders.txt`: List of unique holder addresses
- `data/state.json`: Current scan state and progress

## License

MIT License
