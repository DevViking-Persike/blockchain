#!/usr/bin/env bash
set -euo pipefail

echo "=== Blockchain Node - Init ==="

# 1. Check Rust
if ! command -v rustc &>/dev/null; then
  echo "[*] Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo "[ok] Rust $(rustc --version | cut -d' ' -f2)"
fi

# 2. Check .env
if [ ! -f .env ]; then
  echo "[*] Creating default .env..."
  cat > .env <<EOF
API_PORT=8080
P2P_PORT=0
DIFFICULTY=2
MINING_REWARD=50
RUST_LOG=info
EOF
  echo "[ok] .env created"
else
  echo "[ok] .env exists"
fi

# 3. Build
echo "[*] Building workspace..."
cargo build --release --workspace

# 4. Tests
echo "[*] Running tests..."
cargo test --workspace

echo ""
echo "=== Init complete ==="
echo "Run: cargo run --release -p blockchain-node"
