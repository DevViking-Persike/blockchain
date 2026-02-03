# ADR-0002: Proof of Work Consensus

## Status
Accepted

## Context
The blockchain needs a consensus mechanism to validate new blocks. Options considered: Proof of Work (PoW), Proof of Stake (PoS), Proof of Authority (PoA).

## Decision
Use SHA-256 based Proof of Work with configurable difficulty. Difficulty is expressed as the number of leading zeros required in the block hash. The nonce is incremented until a valid hash is found.

Configuration via CLI flag or `.env`:
```
--difficulty 2  # requires hash starting with "00"
```

## Consequences
- Simple to implement and reason about
- Mining time increases exponentially with difficulty
- CPU-intensive; not suitable for energy-efficient production use
- Easy to adjust difficulty for development (difficulty=1) vs testing (difficulty=2+)
- Future option to migrate to PoS without changing core block/transaction structures
