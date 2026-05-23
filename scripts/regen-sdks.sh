#!/bin/bash
set -euo pipefail

cargo run -p ygg-cli --bin export-schemas
cargo run -p ygg-cli --bin generate-sdks
echo "SDKs regenerated. Review and commit changes under sdk/."
