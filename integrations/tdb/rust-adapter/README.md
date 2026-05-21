# Yggdrasil TDB Rust Adapter

This is an opt-in TriviumDB/TDB adapter proof.

- Default build: JSON-RPC stdio shell, no TriviumDB dependency, no backend open.
- Real local build: `../rust-adapter-real-local/Cargo.toml` uses the sibling TriviumDB checkout and the `real-tdb` feature to call real Rust APIs while reusing this adapter source.
- Package role: ordinary subprocess package adapter, not kernel storage and not `EventStore`.

Default check:

```bash
cargo check --manifest-path integrations/tdb/rust-adapter/Cargo.toml
```

Real local smoke test:

```bash
cargo test --manifest-path integrations/tdb/rust-adapter-real-local/Cargo.toml --features real-tdb
```

This real-local manifest is intentionally separate from the default manifest so ordinary Yggdrasil clones are not bound to a sibling checkout.
