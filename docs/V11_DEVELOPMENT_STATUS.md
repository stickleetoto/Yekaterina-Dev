# Yekaterina v1.1 — development status

Private development checkpoint. This is a short status marker, not a report.
Detail lives in `docs/V11_BASELINE.md` (frozen baseline and measurement
methodology) and `docs/ARCHITECTURE*.md`.

**This tree is the v1.1 development baseline, not a v1.1 release.**

## Frozen v1.0.0 compatibility baseline

| Invariant | Value |
|---|---|
| Registered operations | **1,215** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Schema footprint | **412 tokens / 1,725 bytes** |
| Golden correctness | **527/527 PASS** |
| Full Capability Audit | **1,215/1,215 PASS** |
| `yk.spec` coverage | **1,215/1,215** |
| Rust tests | **91/91 pass**, 0 clippy errors |
| Toolchain | rustc / cargo 1.98.0 (pinned) |

These are frozen. v1.1 changes performance and internal concurrency only; it
adds no operations, no tools, and no schema fields.

## Phase status

| Phase | State | Summary |
|---|---|---|
| **0 — v1.1 verification + benchmark infrastructure** | **Complete** | `scripts/static_audit_v11.py` (v1.0.0 audit preserved unmodified and hash-pinned); `src/lib.rs` library target; `bench/` MCP harness, `bench/paired_ab.py` interleaved A/B, `benches/micro.rs` in-process suite; `VERIFY_V11_WINDOWS.*`, `RUN_BENCH_WINDOWS.bat`. |
| **1 — Freeze v1.0.0 performance baseline** | **Complete** | `bench_results/v1.0.0-frozen/` captured from pristine source (binary sha256 `753b64f8…`, 50 workloads, 5 runs). Documented in `docs/V11_BASELINE.md`, which is immutable. |
| **2A — Remove the O(n²) accumulated-result rescan** | **Next** | Not started. |
| 2B — Reduce avoidable argument cloning | Pending | |
| 2C — Expression AST reuse (bit-identical required) | Pending | |
| 2D — Release profile tuning (measured, not assumed) | Pending | |
| 3 — Concurrency prerequisites (R1 torn snapshot, fsync under lock) | Pending | Blocking for Phase 6. |
| 4 — Operation safety classification (fail-closed) | Pending | |
| 5 — Thread worker pool | Pending | Default workers = 1. |
| 6 — Ordered parallel batch execution | Pending | |
| 7 — Adaptive parallelism | Pending | |
| 8 — Pipeline parallelism | Pending | Lower priority; may be deferred. |

## Current goal

**Performance optimization and parallel runtime foundation**, with no change to
what the LLM sees. Externally observable behaviour must remain byte-identical.

## Measurement rules in force

Established during Phase 0/1 and binding on every later phase:

- `bench/paired_ab.py` is the deciding instrument for accepting an optimization.
  The baseline machine drifted **1.73× within a single benchmark run**, and a
  stored-baseline comparison once reported +14…+26% for a change a paired run
  measured at −0.35%.
- A regression counts only if it exceeds 2% **and** the workload's own measured
  spread, above a 0.20 ms noise floor. Everything else is reported as `watch`.
- Response SHA-256 fingerprints are a **hard, non-statistical gate**. Any
  mismatch means externally observable behaviour changed.
- No optimization merges without before/after measurements and an explanation of
  the delta.

## Repository separation

- `stickleetoto/Yekaterina` — **public** distribution repository. Not touched by
  this work.
- `stickleetoto/Yekaterina-Dev` — **private** development repository. This tree.
