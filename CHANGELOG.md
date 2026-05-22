# Changelog

All notable changes to Yggdrasil will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Vite bundling and iframe-based SurfaceHost in clients/web (S1)
- Tauri 2.x desktop wrapper at clients/desktop (S2)
- GitHub Actions CI and release workflows (S3)
- `scripts/release-version.sh` for version stamping
- `BUILDING.md` with cross-platform build instructions
- This changelog

### Outbound (prior to S-track)
- `kernel.outbound.execute` for unary HTTPS
- `kernel.outbound.stream` for SSE/NDJSON streaming
- `kernel.outbound.websocket.*` for bidirectional WebSocket
- Outbound completion audit events
- Manifest `permissions.secret_refs` declarations
- Subprocess SDK reverse `kernel.*` dispatch + `kernelClient.openWebSocket`

## [0.1.0] — TBD (initial release)

Initial public release.
