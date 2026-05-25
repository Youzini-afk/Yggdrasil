# Zeabur Web Quick Validation

This deployment path is a minimal single-container adapter for quick web validation. It is not the desktop release path and is not intended to define new kernel concepts or package boundaries. Treat any public URL as hostile: use a throwaway volume, set an access token, and do not put real secrets into this quick-validation host.

## What it runs

The root `Dockerfile` builds:

- `clients/web` with `npm ci` and `npm run build`
- the Rust `ygg` release binary
- a runtime image that serves the built web app and host API on one port
- the minimal official lab manifests needed for web install and secret-store quick validation (`git-tools-lab`, `integrity-lab`, `install-lab`, `secret-store-lab`)

At runtime the container starts:

```sh
ygg host serve --http 0.0.0.0:$PORT --data-dir /data --profile /data/profiles/default.yaml --static-dir /app/public --access-token "$YGG_HTTP_ACCESS_TOKEN"
```

The same HTTP service exposes:

- `GET /` and web assets from `/app/public`
- `POST /rpc` (requires token when configured)
- `GET /kernel/v1/event.subscribe/:session_id` (requires token when configured)
- `GET /surface-bundles/...` (public read-only browser artifacts)
- `GET /healthz`

When `YGG_HTTP_ACCESS_TOKEN` is set, `/rpc` and `/kernel/...` routes require `Authorization: Bearer <token>`. Browser SSE uses `?access_token=<token>` because EventSource cannot send custom headers. Normally, open the web URL directly and paste the token into the login screen; the web client validates it, stores it in `localStorage`, and sends it on future RPC/SSE calls. As an optional bootstrap path, you can also visit once with `?ygg_token=<token>` or `?access_token=<token>`; the web client reads it once and scrubs it from the address bar.

Surface bundles under `/surface-bundles/...` are public frontend artifacts in this quick-validation deployment so sandboxed iframe dynamic imports, stylesheets, fonts, and images can load reliably. Do not put secrets in bundles or assets. The security boundary is the host RPC/kernel token plus the SurfaceHost bridge capability policy, not hiding frontend JavaScript or CSS.

## Zeabur settings

- Service type: Dockerfile from the repository root
- Port: set `PORT` if Zeabur does not inject it automatically; default is `8080`
- Health check: `GET /healthz`
- Volume: mount persistent storage at `/data`

Recommended environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | Public HTTP listen port; entrypoint binds `0.0.0.0:$PORT`. |
| `YGG_DATA_DIR` | `/data` | Persistent Yggdrasil data directory. |
| `YGG_PROFILE` | `default` | Safe profile id created under `/data/profiles`; entrypoint rejects unsafe values. |
| `YGG_STATIC_DIR` | `/app/public` | Built web static directory to serve. |
| `YGG_HTTP_ACCESS_TOKEN` | unset | Strongly recommended for any public URL; protects RPC/SSE/service routes. |
| `YGG_REQUIRE_ACCESS_TOKEN` | `0` | Set to `1` to make the container fail fast if `YGG_HTTP_ACCESS_TOKEN` is absent. |

If `/data/profiles/$YGG_PROFILE.yaml` does not exist, the entrypoint creates a small SQLite-backed profile. You can replace it by mounting or writing your own profile into the same path.

For Zeabur/public validation, set a random token, open the app URL, and enter that token on the login screen. Do not reuse production credentials and do not store real provider secrets in this validation instance. URL tokens (`?ygg_token=<token>` or `?access_token=<token>`) remain available as an optional one-time bootstrap path.

## Local smoke test

```sh
docker build -t ygg-zeabur-quick .
docker run --rm -p 8080:8080 -v ygg-data:/data \
  -e YGG_HTTP_ACCESS_TOKEN=dev-token \
  -e YGG_REQUIRE_ACCESS_TOKEN=1 \
  ygg-zeabur-quick
curl http://127.0.0.1:8080/healthz
```

## Limitations

- Quick validation only; this does not replace packaged desktop distribution.
- The access token is quick-validation auth only, not production-grade session auth.
- Surface bundles/assets are public read-only browser artifacts; never embed secrets in them.
- No new official package namespace or install model is introduced by this adapter.
- Use a throwaway `/data` volume for public validation and delete it after testing.
- Do not enter real API keys or sensitive secrets on a public quick-validation URL.
- The bundled web app is static; local development hot reload still uses the existing Vite workflow.
