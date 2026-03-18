#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
cargo run -p vigil-cli -- seed-demo
cargo run -p vigil-cli -- detect
cargo run -p vigil-cli -- daemon --port 8080
