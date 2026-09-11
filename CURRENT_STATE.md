# CURRENT_STATE.md

> Active-development snapshot for `stickleetoto/Yekaterina-Dev`.

## Current milestone

**v1.2.0 remains the promoted stable distribution; the development tree now has the v1.3 transformer registry migration implemented and fully verified at 1,425 built-in/control operations.**

The stable distribution repository is still `stickleetoto/Yekaterina` at the v1.2.0 release line. This development repository preserves the v1.2 registry/engine source as historical baselines while placing the v1.3 aggregate registry and dispatch shims in front of them.

| | |
|---|---|
| Stable distribution line | `1.2.0` |
| Development package metadata | `1.2.0` until an explicit release-version promotion |
| Advertised MCP initialize version | `1.0.0` (deliberately compatibility-gated) |
| Registered built-in/control opcodes | **1,425** |
| Native transformer opcodes | **15** — registered and executable |
| Importable transformer formula-pack ops | **20** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Error vocabulary | unchanged |
| Golden corpus | **527/527** regression gate retained |
| Frozen v1.2 full-audit surface | **1,410/1,410** retained |
| Rust edition / toolchain | 2024 / pinned 1.98.0 |
| Default workers | 1 |

## v1.3 registry architecture

The migration intentionally does **not** rewrite the promoted v1.2 `src/registry.rs` or `src/engine.rs` files.

- `src/registry.rs` remains the frozen 1,410-operation v1.2 catalog.
- `src/registry_v13.rs` exposes an aggregate read-only catalog: the legacy 1,410 entries followed by the 15 transformer entries, for exactly **1,425** operations.
- `src/engine.rs` remains the historical v1.2 execution engine.
- `src/engine_v13.rs` dispatches `xfmr.*` operations to the transformer-native modules and delegates every legacy operation to the historical engine.
- `src/lib.rs` aliases the historical files as `registry_v12` / `engine_v12` and exposes the aggregate modules as the live `registry` / `engine` surface.

This keeps v1.2 evidence auditable while allowing normal v1.3 discovery and execution.

## Promoted transformer-native surface

`full_audit/opcodes_v13_transformer_candidate.json` now marks the 15-op contract as **registered**:

- 7 material/interpolation/thermal/basic-geometry operations;
- 6 winding-field/leakage/AC-loss operations;
- 2 candidate evaluation/ranking operations.

The operations are discoverable through `yk.find`, inspectable through `yk.spec`, and executable through ordinary `yk.compute`.

The candidate decision layer includes:

- `xfmr.candidate_evaluate` — explicit hard-constraint pass/fail evidence, ordered violations, and normalized 0..1 weighted desirability;
- `xfmr.candidate_rank` — deterministic feasible-first ranking, then score, then original index.

Caller-supplied limits and objective bands remain policy data. Registry promotion does not imply IEC/IEEE/DOE certification or factory-design approval.

## Verification status

The v1.3 migration head passed the complete combined verification path:

- `scripts/static_audit_v13.py`: **17 pass / 0 fail**;
- aggregate operation manifest: **1,425 total / 15 xfmr**;
- complete `cargo test --locked --all-targets`: PASS;
- complete `cargo clippy --locked --all-targets`: PASS;
- release build: PASS;
- reviewer-facing MCP demo: PASS;
- `scripts/verify_v13_transformer_runtime.py`: PASS against the real release executable, covering discovery/spec/execution for the promoted transformer surface;
- existing v1.2 operation verifier: PASS;
- statistical reference verifier: PASS;
- multiplicity verifier: PASS;
- Golden regression corpus: PASS;
- frozen v1.2 Full Capability Audit: PASS;
- benchmark invariants: PASS.

A dedicated Transformer Candidate workflow independently passed its aggregate audit, candidate tests, aggregate registry/engine tests, clippy, release build, and v1.3 MCP runtime verification.

## Compatibility invariants

The migration retains:

1. every one of the 1,410 v1.2 canonical operation names and their ordering;
2. exactly 3 MCP tools;
3. the existing model-facing request schema;
4. the compatibility-gated MCP initialize identity;
5. the existing error vocabulary;
6. default worker count 1 and deterministic scheduling rules;
7. the v1.2 Golden/reference/full-audit regression evidence;
8. non-duplication of existing generic electrical operations.

## Next development decisions

The next high-value work is no longer registry exposure. Priorities are:

- decide when to promote package/release metadata to a formal `1.3.x` release;
- extend leakage geometry to unequal-height and multi-winding arrangements;
- add CTC/strand and structural stray-loss models only with explicit assumptions and independent validation;
- add calibrated hot-spot and mechanical short-circuit models only when their model/source boundaries are documented;
- keep standard-versioned certification and dielectric-clearance claims outside the generic calculator until a dedicated validated compliance layer exists.

## Documentation status

`REPO_MAP.md` has been refreshed to describe the live v1.3 aggregate registry/engine shims and the current 1,425-operation development surface. Historical v1.2 source files remain intentionally frozen for auditability.

## Repository roles

- `stickleetoto/Yekaterina-Dev` — active source development and v1.3 verification.
- `stickleetoto/Yekaterina` — promoted stable distribution and release evidence, currently v1.2.0.
