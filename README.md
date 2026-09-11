# Yekaterina-Dev

Release-finalization repository for Yekaterina v1.3.0.

## Status

- Stable distribution: `stickleetoto/Yekaterina` remains on the v1.2.0 release line until promotion.
- Development package metadata: **1.3.0**.
- Live built-in/control operation surface: **1,425 operations**.
- Transformer-native operations: **15**.
- MCP tools remain exactly **3**: `yk.compute`, `yk.find`, `yk.spec`.
- Golden regression corpus: **527/527** retained.
- Frozen v1.2 full-capability audit surface: **1,410/1,410** retained.
- Advertised MCP initialize version remains `1.0.0` by compatibility policy.

The v1.3 compute scope is closed for release finalization. New calculator families or transformer-model extensions belong to a later development line rather than the v1.3 release candidate.

For the authoritative snapshot, see [`CURRENT_STATE.md`](CURRENT_STATE.md).
For repository ownership and file locations, see [`REPO_MAP.md`](REPO_MAP.md).
For the release gate and promotion procedure, see [`RELEASE_CHECKLIST_V13.md`](RELEASE_CHECKLIST_V13.md).

## v1.3 architecture

v1.3 keeps the promoted v1.2 implementation auditable rather than rewriting it in place:

- `src/registry.rs` — frozen v1.2 registry, 1,410 operations.
- `src/engine.rs` — frozen v1.2 execution engine.
- `src/registry_v13.rs` — live aggregate registry, 1,425 operations total.
- `src/engine_v13.rs` — live dispatch shim for transformer-native operations plus legacy delegation.

The model-facing surface remains `yk.compute`, `yk.find`, and `yk.spec`.

## Verification

The authoritative CI path runs the v1.3 static audit, locked Rust tests and clippy, release build, MCP demo, transformer runtime verification, v1.2 independent reference verifiers, Golden regression, frozen full-capability audit, and benchmark invariants.

`scripts/static_audit_v13.py` additionally gates the v1.3 package version in both `Cargo.toml` and `Cargo.lock`, preventing release metadata drift.

## Repository roles

`Yekaterina-Dev` is the release-finalization and future development tree. The separate `Yekaterina` repository is the promoted stable distribution and should only receive v1.3 after the finalization CI gate passes.
