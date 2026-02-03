# ADR-0001: Cargo Workspace Architecture

## Status
Accepted

## Context
The blockchain project requires multiple distinct concerns: core domain logic, a virtual machine for smart contracts, a REST API, P2P networking, and a binary that ties everything together. Putting all code in a single crate would create tight coupling and slow compilation.

## Decision
Use a Cargo workspace with 5 crates following clean architecture principles:

- **blockchain-core** - Domain layer: blocks, transactions, merkle tree, wallet, state, chain
- **blockchain-vm** - Application layer: opcodes, VM, compiler, contract executor
- **blockchain-api** - Interface layer: REST endpoints with Axum
- **blockchain-network** - Interface layer: P2P with libp2p (gossipsub + mDNS)
- **blockchain-node** - Infrastructure layer: binary integrating all crates

Dependencies flow inward: `node -> api/network -> vm -> core`.

## Consequences
- Each crate compiles independently, enabling parallel compilation
- Core domain has zero dependency on infrastructure (API, network)
- Crates can be tested in isolation (`cargo test -p blockchain-core`)
- Adding new interfaces (CLI, WebSocket) doesn't require changing core logic
- Slightly more boilerplate for inter-crate type sharing
