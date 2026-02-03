# ADR-0005: libp2p for P2P Networking

## Status
Accepted

## Context
Nodes need to discover each other and exchange blocks/transactions. Options considered: custom TCP protocol, libp2p, ZeroMQ.

## Decision
Use libp2p with:
- **gossipsub** - Pub/sub message propagation for blocks and transactions
- **mDNS** - Local network peer discovery (zero configuration)
- **noise** - Encrypted transport
- **yamux** - Stream multiplexing
- **TCP** - Transport layer

Communication between the network layer and the application uses `tokio::mpsc` channels:
- `NetworkCommand` (API -> network): broadcast transactions/blocks, request chain
- `NetworkEvent` (network -> node): received transactions/blocks, peer connect/disconnect

## Consequences
- Automatic peer discovery on local networks via mDNS
- Gossipsub ensures efficient message propagation without flooding
- Encrypted connections by default (noise protocol)
- Channel-based architecture decouples network from business logic
- Adding WAN peer discovery would require adding Kademlia DHT or bootstrap nodes
