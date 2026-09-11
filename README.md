# Yekaterina-Dev

Active development repository for Yekaterina.

## Status

- Stable distribution: `stickleetoto/Yekaterina` on the v1.2.0 release line.
- Development tree: v1.3 transformer registry migration implemented and verified.
- Live built-in/control operation surface: **1,425 operations**.
- Transformer-native operations: **15**.
- MCP tools remain exactly **3**: `yk.compute`, `yk.find`, `yk.spec`.
- Golden regression corpus: **527/527** retained.
- Frozen v1.2 full-capability audit surface: **1,410/1,410** retained.

For the authoritative active snapshot, see [`CURRENT_STATE.md`](CURRENT_STATE.md).
For repository ownership and file locations, see [`REPO_MAP.md`](REPO_MAP.md).

## Development architecture

The v1.3 migration keeps the v1.2 implementation auditable rather than rewriting it in place:

- `src/registry.rs` — frozen v1.2 registry, 1,410 operations.
- `src/engine.rs` — frozen v1.2 execution engine.
- `src/registry_v13.rs` — live aggregate registry, 1,425 operations total.
- `src/engine_v13.rs` — live dispatch shim for transformer-native operations plus legacy delegation.

The normal model-facing surface remains `yk.compute`, `yk.find`, and `yk.spec`.

## Verification

The current v1.3 migration state has passed the combined verification path recorded in `CURRENT_STATE.md`, including Rust tests, clippy, release build, runtime transformer verification, Golden regression, frozen v1.2 full audit, and benchmark invariants.

## Repository roles

`Yekaterina-Dev` is the active development tree. The separate `Yekaterina` repository is the promoted stable distribution.
