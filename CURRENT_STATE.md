# CURRENT_STATE.md

> Snapshot of this working tree. Verified against the source on 2026-09-06.

## Current milestone

`Cargo.toml` = **1.2.0**. The tree is the **v1.2 development line, documented
and gated**. No batch is outstanding.

| | |
|---|---|
| Crate version | 1.2.0 (`Cargo.toml`, `Cargo.lock`) |
| Advertised MCP version | `1.0.0` (deliberate; separate literal in `#[tool_handler]`) |
| Registered opcodes | **1,387** in `src/registry.rs` |
| MCP tools | 3 — `yk.compute`, `yk.find`, `yk.spec` |
| Schema footprint | 412 tokens / 1,725 bytes |
| Error codes | 30 |
| Serialized operations | 8 — the seven `udo.*` plus `expr.eval` |
| Golden corpus | 527 cases (`golden/cases.json`, version tag `0.1.0-alpha.12-hotfix6`) |
| Rust edition / toolchain | 2024 / pinned 1.98.0 |
| Default workers | 1 (`DEFAULT_WORKERS` in `src/main.rs`) |

## Completed

**v1.0.0** — 1,215 operations, 3 tools, 527/527 golden, full capability audit
1,215/1,215. `SOURCE_INTEGRITY.txt` is that record and has never been rewritten.

**v1.1.0** — performance and concurrency, no new operations. All eleven planned
phases except Phase 8 are complete (`docs/V11_DEVELOPMENT_STATUS.md`):

- Phase 0/1 — verification and benchmark infrastructure; frozen v1.0.0 baseline.
- Phase 2A — `ResultBudget` removed the O(n²) accumulated-result rescan.
- Phase 2B — borrowed arguments (`Cow`) in `parse_step_ref` / `resolve_args`.
- Phase 2C — `formula::Env` hoists the per-evaluation map clone out of the
  `num` / `ode` / `series` / `optimize` solver loops. Acceptance was bit-identical
  f64 output, pinned by `tests/formula_bit_identity.rs`.
- Phase 2D — release profile: `lto = "fat"`, `codegen-units = 1`, `panic` left at
  `unwind` so a panicking job is contained rather than aborting the process.
- Phase 3 — copy-on-write registry behind `Arc`; fsync moved out of the registry
  write lock behind a dedicated store gate.
- Phase 4 — `src/safety.rs`, anchored on the dispatcher's control set.
- Phase 5 — `src/pool.rs`, the only thread-creating module.
- Phase 6 — ordered parallel batch execution via `src/scheduler.rs`.
- Phase 7 — two-dimensional (compute vs payload) adaptive parallelism.
- Phase 9/10 — 1/2/4/8 worker sweep and concurrency stress runs.
- Phase 11 — v1.1 freeze, `SOURCE_INTEGRITY_V11.txt`, MCP-identity hash gate.

**v1.2** — 172 operations, 1,215 → 1,387, in two batches and one fix.

- Applied families, 103 ops: `int` 8→26, `dec` 4→20, `geo` 8→30, `fin` 8→29,
  `vec` 9→22, `unit` 13→20, `pct` 3→9. `docs/V12_OPERATIONS.md`, verified by
  `scripts/verify_v12_operations.py` (164 assertions).
- Statistical inference, 69 ops: `special` +3 (regularized incomplete gamma and
  beta), `prob` +22 (t, chi-square, F, gamma, beta, lognormal, Weibull, each with
  a survival function beside its CDF), `test` +27, `reg` +17. New module
  `src/inference.rs`. `docs/V12_STATISTICS.md`, verified by
  `scripts/verify_statistics.py` — 961 values against scipy, numpy and mpmath,
  which runs in CI.
- **Fixed a v1.1 defect**: `expr.eval` was classified `Pure` and could be sent to
  a worker, where `engine::execute` has no arm for it, so a distributed mixed
  batch returned `NYI` at more than one worker. Now `Serialized`.
  `docs/V11_SAFETY_MODEL.md` records why the reasoning was wrong and why no test
  caught it.

## In progress

The tree, its documentation and its gates agree. Nothing is half-finished.

**Not yet published.** The last commit on `stickleetoto/Yekaterina-Dev` is
`f3cbc065` (the v1.1 freeze). Everything in "Completed" under v1.2 is local
only. Publishing is a GitMake plan plus a terminal `gitmake approve`; a new
session should run `gitmake_prepare` again rather than reuse an old plan id,
because a plan is bound to the tree it was prepared from.

Post-publication checks that have been used every time: commit SHA matches,
remote visibility still `public`, `target/` absent from the remote tree, secret
scan clean, and `stickleetoto/Yekaterina` — the separate public distribution
repository — still at `09f62455` and untouched.

## Deferred

- **Phase 8 — pipeline parallelism.** Explicitly deferred in
  `docs/V11_DEVELOPMENT_STATUS.md`: pipelines return only the last value by
  default, cap at 256 steps, and measured 1.66 µs/step, so the available gain is
  small against real correctness risk.
- **`DEFAULT_WORKERS` stays 1.** Parallelism helps a narrow class (1.75×–3.69×
  on `integrate1000_x64` and `mixed_skew_16`; every other workload neutral within
  ±2%), so it remains opt-in via `--workers N|auto` or `YEKATERINA_WORKERS`.
- **`OperationSource::Wasm`** is declared and `#[allow(dead_code)]`. No Wasm
  operation exists.
- **`dec.sqrt`** dropped on principle; `dec.pow` refuses negative exponents.
- **Exact permutation p-values** for the rank tests. Tractable only for small
  samples; an operation whose cost explodes with input size does not belong in a
  batch engine. The tie-corrected normal approximation is used instead.
- **`test.ks_normal` takes the normal's parameters** rather than estimating them.
  Estimating turns it into Lilliefors, whose null distribution differs.

## Blockers

None.

## Tooling for the next change

| script | what it is for |
|---|---|
| `scripts/register_ops.py` | add operations: writes `registry.rs`, the manifest, the fixture file and every count that names the total, and **refuses to write if a new opcode or alias collides** with anything registered. That check found two of the nine duplicates in the v1.2 batch. |
| `scripts/mutate_gates.py` | break what each gate guards and confirm it notices. All six mutations are caught. Run it after touching a gate; a gate that has never failed is a guess. |
| `scripts/gen_source_integrity_v12.py` | regenerate `SOURCE_INTEGRITY_V12.txt` after any tracked file changes, or `static_audit_v12.py` fails. |
| `scripts/verify_statistics.py` | 961 values against scipy, numpy and mpmath. Needs those three; skips cleanly without them. |
| `scripts/verify_v12_operations.py` | 164 assertions for the applied and exact families. |

Two ordering facts worth knowing before editing the dispatchers:

- `engine::dispatch_module` must ask `inference::execute` **before**
  `advanced_stats::execute`, which claims the whole `test.` and `reg.` prefixes
  with `starts_with` and answers `OP` for anything it does not implement.
- `scripts/validate_full_audit.py` checks `opcodes_v11_frozen.json` for
  corruption **before** the deep-family gate, which scopes itself to that list.

## Constraints in force

- Externally observable behaviour must stay byte-identical across worker counts
  and against the frozen baseline. Response SHA-256 fingerprints are a hard gate.
- `bench/paired_ab.py` is the deciding instrument for accepting an optimization;
  stored-baseline comparison is not trusted (the baseline machine drifted 1.73×
  within a single run).
- A regression counts only above 2% **and** the workload's own spread, above a
  0.20 ms noise floor.
- No operation may be added, renamed or reordered without updating
  `full_audit/opcodes_alpha12.json` and a fixture in `full_audit/fixtures_v12.json`.
- Two separate repositories: `stickleetoto/Yekaterina` (public distribution, not
  touched by this work) and `stickleetoto/Yekaterina-Dev` (this tree).
