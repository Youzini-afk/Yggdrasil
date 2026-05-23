#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run -p ygg-cli --bin export-schemas >/dev/null
cargo run -p ygg-cli --bin validate-schemas
