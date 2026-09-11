# CURRENT_STATE.md

> Release-finalization snapshot for `stickleetoto/Yekaterina-Dev`.

## Current milestone

**Yekaterina v1.3.0 is in release finalization with 1,425 built-in/control operations. The v1.3 compute scope is closed; the remaining gate is verification and promotion, not feature expansion.**

The stable distribution repository `stickleetoto/Yekaterina` remains on v1.2.0 until the final v1.3 CI gate passes and the release is deliberately promoted. This repository preserves the v1.2 registry/engine source as historical baselines while exposing the v1.3 aggregate registry and dispatch shims.

| | |
|---|---|
| Stable distribution line | `1.2.0` until v1.3 promotion |
| Development package metadata | **`1.3.0`** |
| Cargo.lock root package | **`1.3.0`** |
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

The release intentionally does **not** rewrite the promoted v1.2 `src/registry.rs` or `src/engine.rs` files.

- `src/registry.rs` remains the frozen 1,410-operation v1.2 catalog.
- `src/registry_v13.rs` exposes the live aggregate catalog: 1,410 legacy entries followed by 15 transformer entries, exactly **1,425** operations.
- `src/engine.rs` remains the historical v1.2 execution engine.
- `src/engine_v13.rs` dispatches `xfmr.*` operations to transformer-native modules and delegates every legacy operation to the historical engine.
- `src/lib.rs` aliases the historical files as `registry_v12` / `engine_v12` and exposes the aggregate modules as the live `registry` / `engine` surface.

This preserves v1.2 audit evidence while allowing normal v1.3 discovery and execution.

## Promoted transformer-native surface

`full_audit/opcodes_v13_transformer_candidate.json` marks the 15-op contract as **registered**:

- 7 material/interpolation/thermal/basic-geometry operations;
- 6 winding-field/leakage/AC-loss operations;
- 2 candidate evaluation/ranking operations.

The operations are discoverable through `yk.find`, inspectable through `yk.spec`, and executable through `yk.compute`.

The candidate decision layer includes:

- `xfmr.candidate_evaluate` — explicit hard-constraint pass/fail evidence, ordered violations, and normalized 0..1 weighted desirability;
- `xfmr.candidate_rank` — deterministic feasible-first ranking, then score, then original index.

Caller-supplied limits and objective bands remain policy data. Registry promotion does not imply IEC/IEEE/DOE certification or factory-design approval.

## Release-finalization verification

The authoritative CI path requires all of the following to pass on the finalization commit:

- `scripts/static_audit_v13.py`, including `Cargo.toml` and `Cargo.lock` version agreement at `1.3.0`;
- aggregate operation manifest: **1,425 total / 15 xfmr**;
- `cargo test --locked --all-targets`;
- `cargo clippy --locked --all-targets`;
- locked release build;
- reviewer-facing MCP demo;
- `scripts/verify_v13_transformer_runtime.py` against the real release executable;
- existing v1.2 operation, statistics, and multiplicity reference verifiers;
- Golden regression corpus;
- frozen v1.2 Full Capability Audit;
- benchmark invariants.

The pre-finalization v1.3 migration head had already passed the combined verification path. Final release status must be taken from CI on the version-promoted finalization commit rather than inferred from that earlier run.

## Compatibility invariants

v1.3 retains:

1. every one of the 1,410 v1.2 canonical operation names and their ordering;
2. exactly 3 MCP tools;
3. the existing model-facing request schema;
4. the compatibility-gated MCP initialize identity;
5. the existing error vocabulary;
6. default worker count 1 and deterministic scheduling rules;
7. the v1.2 Golden/reference/full-audit regression evidence;
8. non-duplication of existing generic electrical operations.

## Scope freeze

The following are explicitly **post-v1.3** work and are not release blockers:

- unequal-height and multi-winding leakage geometry;
- CTC/strand and structural stray-loss models;
- calibrated hot-spot and mechanical short-circuit models;
- standard-versioned certification or dielectric-clearance compliance layers;
- unrelated generic calculator expansion.

These require separate assumptions, references, validation, or product decisions and should not destabilize the v1.3 closeout.

## Repository roles

- `stickleetoto/Yekaterina-Dev` — v1.3 release finalization and subsequent development.
- `stickleetoto/Yekaterina` — promoted stable distribution, currently v1.2.0 until final v1.3 promotion.

See `RELEASE_CHECKLIST_V13.md` for the promotion gate.
