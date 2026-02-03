# ADR-0003: Ed25519 for Transaction Signatures

## Status
Accepted

## Context
Transactions need digital signatures to prevent forgery. Options considered: ECDSA (secp256k1), Ed25519, RSA.

## Decision
Use Ed25519 via the `ed25519-dalek` crate. Wallet addresses are derived from the SHA-256 hash of the public key, truncated to 40 hex characters with a `0x` prefix.

Signing flow:
1. Wallet generates Ed25519 keypair
2. Transaction is hashed (SHA-256 of id + sender + recipient + amount + timestamp)
3. Hash is signed with the private key
4. Signature and public key are attached to the transaction
5. Validators verify the signature before accepting the transaction

System transactions (mining rewards) from sender "system" bypass signature verification.

## Consequences
- Ed25519 is fast (both signing and verification)
- Fixed 64-byte signatures, 32-byte public keys
- No signature malleability issues (unlike ECDSA)
- Widely audited implementation in `ed25519-dalek`
- Address format is not compatible with Ethereum (different curve)
