#!/usr/bin/env sh
set -eu

PORT="${PORT:-8080}"
YGG_DATA_DIR="${YGG_DATA_DIR:-/data}"
YGG_PROFILE="${YGG_PROFILE:-default}"
YGG_STATIC_DIR="${YGG_STATIC_DIR:-/app/public}"

case "$YGG_PROFILE" in
  *[!A-Za-z0-9_.-]* | "" | .* | *..*)
    echo "Invalid YGG_PROFILE. Use a safe profile id with letters, numbers, dot, underscore, or dash." >&2
    exit 1
    ;;
esac

if [ "${YGG_REQUIRE_ACCESS_TOKEN:-0}" = "1" ] && [ -z "${YGG_HTTP_ACCESS_TOKEN:-}" ]; then
  echo "YGG_REQUIRE_ACCESS_TOKEN=1 but YGG_HTTP_ACCESS_TOKEN is not set." >&2
  exit 1
fi

PROFILE_DIR="$YGG_DATA_DIR/profiles"
PROFILE_PATH="$PROFILE_DIR/$YGG_PROFILE.yaml"

mkdir -p "$PROFILE_DIR" "$YGG_DATA_DIR/projects" "$YGG_DATA_DIR/store" "$YGG_DATA_DIR/cache" "$YGG_DATA_DIR/keys"

if [ ! -f "$PROFILE_PATH" ]; then
  cat >"$PROFILE_PATH" <<EOF
title: Zeabur quick validation
event_store:
  kind: sqlite
  path: events.sqlite
secret_resolver:
  store_enabled: true
autoload:
  - /app/packages/official/git-tools-lab/manifest.yaml
  - /app/packages/official/integrity-lab/manifest.yaml
  - /app/packages/official/install-lab/manifest.yaml
  - /app/packages/official/secret-store-lab/manifest.yaml
EOF
fi

set -- ygg host serve \
  --http "0.0.0.0:$PORT" \
  --data-dir "$YGG_DATA_DIR" \
  --profile "$PROFILE_PATH" \
  --static-dir "$YGG_STATIC_DIR"

if [ -n "${YGG_HTTP_ACCESS_TOKEN:-}" ]; then
  set -- "$@" --access-token "$YGG_HTTP_ACCESS_TOKEN"
else
  echo "WARNING: YGG_HTTP_ACCESS_TOKEN is not set; RPC/SSE host routes are public. Use local/dev only." >&2
fi

exec "$@"
