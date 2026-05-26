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
DEFAULT_PROFILE_SENTINEL="$PROFILE_DIR/.zeabur-default-profile-v2"

mkdir -p "$PROFILE_DIR" "$YGG_DATA_DIR/projects" "$YGG_DATA_DIR/store" "$YGG_DATA_DIR/cache" "$YGG_DATA_DIR/keys"

REQUIRED_MANIFESTS="/app/packages/official/git-tools-lab/manifest.yaml
/app/packages/official/integrity-lab/manifest.yaml
/app/packages/official/install-lab/manifest.yaml
/app/packages/official/secret-store-lab/manifest.yaml"

if [ "${YGG_PROFILE_RESET:-0}" = "1" ]; then
  echo "YGG_PROFILE_RESET=1; replacing $PROFILE_PATH" >&2
  rm -f "$PROFILE_PATH" "$DEFAULT_PROFILE_SENTINEL"
fi

if [ "$YGG_PROFILE" = "default" ] && [ -f "$PROFILE_PATH" ] && [ ! -f "$DEFAULT_PROFILE_SENTINEL" ]; then
  if awk '/^  - / { print $2 }' "$PROFILE_PATH" | while IFS= read -r manifest; do [ -f "$manifest" ] || exit 1; done; then
    :
  else
    echo "Existing default profile references missing autoload manifests; backing it up and writing Zeabur quick-validation defaults." >&2
    cp "$PROFILE_PATH" "$PROFILE_PATH.bak.$(date +%Y%m%d%H%M%S)" 2>/dev/null || true
    rm -f "$PROFILE_PATH"
  fi
fi

if [ ! -f "$PROFILE_PATH" ]; then
  cat >"$PROFILE_PATH" <<EOF
title: Zeabur quick validation
event_store:
  kind: sqlite
  path: $YGG_DATA_DIR/events.sqlite
secret_resolver:
  store_enabled: true
autoload:
  - /app/packages/official/git-tools-lab/manifest.yaml
  - /app/packages/official/integrity-lab/manifest.yaml
  - /app/packages/official/install-lab/manifest.yaml
  - /app/packages/official/secret-store-lab/manifest.yaml
EOF
  if [ "$YGG_PROFILE" = "default" ]; then
    touch "$DEFAULT_PROFILE_SENTINEL"
  fi
fi

echo "Yggdrasil Zeabur startup:" >&2
echo "  data_dir=$YGG_DATA_DIR" >&2
echo "  profile=$PROFILE_PATH" >&2
echo "  static_dir=$YGG_STATIC_DIR" >&2
echo "  autoload manifest count=$(grep -c '^  - ' "$PROFILE_PATH" || true)" >&2

printf '%s\n' "$REQUIRED_MANIFESTS" | while IFS= read -r manifest; do
  if [ ! -f "$manifest" ]; then
    echo "Required packaged manifest is missing: $manifest" >&2
    exit 1
  fi
done

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
