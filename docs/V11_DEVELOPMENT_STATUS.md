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
| Rust tests | **91/91** at the v1.0.0 freeze; **109/109** on this tree (18 added by Phases 2-3, none removed) |
| clippy errors | 0 |
| Toolchain | rustc / cargo 1.98.0 (pinned) |

These are frozen. v1.1 changes performance and internal concurrency only; it
adds no operations, no tools, and no schema fields.

## Phase status

| Phase | State | Summary |
|---|---|---|
| **0 — v1.1 verification + benchmark infrastructure** | **Complete** | `scripts/static_audit_v11.py` (v1.0.0 audit preserved unmodified and hash-pinned); `src/lib.rs` library target; `bench/` MCP harness, `bench/paired_ab.py` interleaved A/B, `benches/micro.rs` in-process suite; `VERIFY_V11_WINDOWS.*`, `RUN_BENCH_WINDOWS.bat`. |
| **1 — Freeze v1.0.0 performance baseline** | **Complete** | `bench_results/v1.0.0-frozen/` captured from pristine source (binary sha256 `753b64f8…`, 50 workloads, 5 runs). Documented in `docs/V11_BASELINE.md`, which is immutable. |
| **2A — Remove the O(n²) accumulated-result rescan** | **Complete** | `src/limits.rs`: `ResultBudget` folds each appended value in once instead of replaying the whole result vector. Per-item cost went from +83% growth (10→400 items) to −9%, i.e. quadratic to flat. 2.19x on a 400-item batch, 1.70x on batch-10000 and pipeline-256. All 50 workload response fingerprints byte-identical to v1.0.0. |
| 2B — Reduce avoidable argument cloning | **Complete** | `parse_step_ref` returns borrowed arguments and a `Cow` opcode; `resolve_args`/`resolve_composite_args` borrow when the tree holds no `$` string. v1.0.0 deep-copied each batch item's arguments **twice** (parse then resolve), and four validation callers copied arguments only to discard them. 0.839x on 10,000-element arguments, 0.749x on pipeline-256; substituting path neutral at 0.995x. |
| 2C — Expression evaluator (bit-identical required) | **Complete** | `parse_ident` borrows instead of allocating per identifier occurrence; new `formula::Env` hoists the per-evaluation map clone and key allocation out of the solver loops. Converted: `numerical`, `ode`, `series`, `advanced_numerical`, and (2C-3) `optimization`. `Env` uses interior mutability so `optimization.rs` call sites keep their exact original shape. 4.1x on `ode.rk4`, 3.2x on `num.integrate`, 3.4x on a 64-item integrate batch, all with unanimous sign tests and byte-identical responses. |
| 2D — Release profile tuning (measured, not assumed) | **Complete** | `lto = "fat"`, `codegen-units = 1`, `panic` left at `unwind`. Chosen over `thin` on measurement: fat is 4.8% faster (median, 2/7 cases slower) **and** 258 KB smaller. Binary 5,468,160 -> **4,286,464 B**, which is 536 KB *below* v1.0.0 despite the added library target. Build time 44 s -> 124 s. |
| **3 — Concurrency prerequisites** | **Complete** | **R1**: the registry is now copy-on-write behind an `Arc`; a composite runs entirely against one snapshot, acquired lazily at the dynamic lookup so built-in dispatch stays lock-free. **R2**: the snapshot `fsync` moved out of the registry write lock behind a dedicated store gate that orders on-disk generations. Concurrent read tail latency improved 6.4x (p99 29.0 ms -> 4.6 ms). |
| 4 — Operation safety classification (fail-closed) | **Next** | |
| 5 — Thread worker pool | Pending | Default workers = 1. |
| 6 — Ordered parallel batch execution | Pending | |
| 7 — Adaptive parallelism | Pending | |
| 8 — Pipeline parallelism | Pending | Lower priority; may be deferred. |

## R1 was a real defect, and the regression test proves it

`composite_never_observes_a_torn_registry_under_churn` runs a three-step
composite under a concurrent redefinition of its child. A coherent run returns
3 or 3000; a torn one returns a mix. Against the fixed code it passes. Against a
copy of the tree with only the v1.0.0 per-step lookup restored, it fails:

```text
torn registry observed after 383 runs: composite returned 1002,
which mixes two definitions of user.a
```

`1002` is `0 -> +1 -> +1000 -> +1`: three steps of one composite using two
different definitions. This was reachable in v1.0.0 -- `rmcp` already dispatches
each `tools/call` as its own task -- and was hidden only by the serial request
pattern of stdio clients.

## Resolved during Phase 2

**`optimization.rs` per-evaluation clone — fixed in 2C-3.** All 16 `optimize.*`
operations now share the hoisted `Env`. The conversion was blocked on call
structure: `nelder2` calls the objective twice inside a `sort_by` comparator and
`hess2_op` calls it several times within single expressions, so a `&mut Env`
would have forced those float expressions to be rewritten. `Env` was given
interior mutability instead, leaving every call site byte-for-byte as v1.0.0
wrote it. Measured 0.419x / 0.503x / 0.603x on the three new `optimize.*`
workloads, all unanimous.

**`mat.mul` layout regression — gone after 2D.** It measured 1.034x against
v1.0.0 with a significant 17/21 sign test before LTO, and 0.958x with 8/21
after. The diagnosis (incidental instruction layout, no matrix code touched) is
confirmed: changing layout wholesale removed it.

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

- `stickleetoto/Yekaterina` — public **distribution** repository. Not touched by
  this work.
- `stickleetoto/Yekaterina-Dev` — **development** repository: this tree, with the
  Rust source, the verification harnesses and the engineering history. Public as
  of 2026-09-05.

The two remain separate repositories with separate purposes; the distribution
repository is never a target of this work.
