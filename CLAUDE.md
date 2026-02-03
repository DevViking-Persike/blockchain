# Blockchain Node - Rust

## Project Overview
A full blockchain implementation in Rust with smart contracts, P2P networking, and REST API.

## Architecture
Cargo workspace with 5 crates (clean architecture, dependencies flow inward):

```
blockchain-node (binary)
  ├── blockchain-core      # Domain: blocks, transactions, merkle, wallet, state, chain
  ├── blockchain-vm        # Smart contracts: opcodes, VM, compiler, executor
  ├── blockchain-api       # REST API: Axum with 12 endpoints
  └── blockchain-network   # P2P: libp2p (gossipsub + mDNS)
```

## Quick Start
```bash
./init.sh                              # Setup: install rust, build, test
cargo run --release -p blockchain-node # Run with .env defaults
```

## Common Commands
```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all tests (39 tests)
cargo run --release -p blockchain-node -- --api-port 8080 --difficulty 2
```

## Configuration
Priority: CLI flags > .env > defaults

| Variable       | Default | Description                    |
|---------------|---------|--------------------------------|
| API_PORT      | 8080    | REST API port                  |
| P2P_PORT      | 0       | P2P listen port (0 = random)   |
| DIFFICULTY    | 2       | Mining difficulty (leading zeros)|
| MINING_REWARD | 50      | Block mining reward             |
| RUST_LOG      | info    | Log level                       |

## API Endpoints
| Method | Route                      | Description           |
|--------|----------------------------|-----------------------|
| GET    | /api/node/info             | Node info             |
| GET    | /api/chain                 | Full chain            |
| GET    | /api/chain/valid           | Validate chain        |
| POST   | /api/blocks/mine           | Mine a block          |
| GET    | /api/blocks/:index         | Get block by index    |
| POST   | /api/transactions          | Create transaction    |
| GET    | /api/transactions/pending  | Pending transactions  |
| POST   | /api/wallet/new            | Create wallet         |
| GET    | /api/balance/:address      | Check balance         |
| POST   | /api/contracts/deploy      | Deploy contract       |
| POST   | /api/contracts/call        | Call contract         |
| GET    | /api/peers                 | List peers            |

## Code Style
- Follow Rust idioms (clippy-clean)
- Use `thiserror` for error enums
- Use `tracing` for logging (not println)
- Tests in each module (`#[cfg(test)] mod tests`)

## ADRs
Architecture decisions are documented in `docs/adr/`.
