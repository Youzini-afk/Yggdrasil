# Yggdrasil TDB Rust Adapter

This is an opt-in TriviumDB/TDB adapter proof.

- Default build: JSON-RPC stdio shell, no TriviumDB dependency, no backend open.
- Real build: added in the next phase through a local opt-in manifest / feature path.
- Package role: ordinary subprocess package adapter, not kernel storage and not `EventStore`.

Default check:

```bash
cargo check --manifest-path integrations/tdb/rust-adapter/Cargo.toml
```
