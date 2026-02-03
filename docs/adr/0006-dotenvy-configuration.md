# ADR-0006: dotenvy for Configuration Management

## Status
Accepted

## Context
The node needs configurable parameters (port, difficulty, reward). Hardcoding values or requiring CLI flags every time is inconvenient for development.

## Decision
Use `dotenvy` to load `.env` files at startup, combined with `clap`'s `env` attribute for CLI argument parsing. Priority order:

1. CLI flags (highest) - `--api-port 9090`
2. Environment variables / `.env` file - `API_PORT=9090`
3. Default values (lowest) - `8080`

The `.env` file is excluded from git via `.gitignore`.

Default `.env`:
```
API_PORT=8080
P2P_PORT=0
DIFFICULTY=2
MINING_REWARD=50
RUST_LOG=info
```

## Consequences
- Developers can configure the node without remembering CLI flags
- Sensitive configuration stays out of version control
- Same binary works in different environments by swapping `.env`
- `RUST_LOG` controls tracing verbosity without code changes
