# Yekaterina v1.1 — development status

Private development checkpoint. This is a short status marker, not a report.
Detail lives in `docs/V11_BASELINE.md` (frozen baseline and measurement
methodology) and `docs/ARCHITECTURE*.md`.

**This document is the v1.1 record. The tree has since moved to 1.2.0; at the time of writing it was the v1.1 development baseline, not a v1.1 release.**

## Frozen v1.0.0 compatibility baseline

| Invariant | Value |
|---|---|
| Registered operations | **1,215** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Schema footprint | **412 tokens / 1,725 bytes** |
| Golden correctness | **527/527 PASS** |
| Full Capability Audit | **1,215/1,215 PASS** |
| `yk.spec` coverage | **1,215/1,215** |
| Rust tests | **91/91** at the v1.0.0 freeze; **144/144** on this tree (53 added by Phases 2-10, none removed) |
| clippy | **exit 0** under the CI invocation `cargo clippy --locked --all-targets`. **0 warnings in any v1.1 module.** The ~120 remaining warnings are pre-existing style lints in the frozen v1.0.0 numerical modules (116 of them `suspicious_else_formatting`); they are left alone because editing frozen numerical code for style is exactly the churn this release forbids. |
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
| **4 — Operation safety classification** | **Complete** | `src/safety.rs`. Anchored on the dispatcher's control set rather than on per-operation metadata: `engine::execute` takes no `&self`, so anything routed there is pure *by type*, and statefulness requires a dispatcher arm. `server.rs` dispatches through `safety::control_op`, so classifier and dispatcher are the same code and cannot drift. Zero churn in `registry.rs`; all frozen audits keep working. 7 Serialized / 1,208 Pure. Rationale and rejected options in `docs/V11_SAFETY_MODEL.md`. |
| **5 — Thread worker pool** | **Complete** | `src/pool.rs`, the only module allowed to create OS threads (audit-enforced). Threads created once and parked on a condvar; nothing spawned per request. Jobs and results are plain owned data -- the seam a future process backend would attach to, with no trait hierarchy introduced for a second implementation that cannot exist yet. `--workers N|auto`, `YEKATERINA_WORKERS`, **default 1**. Single-worker path measured neutral (-0.46%). |
| **6 — Ordered parallel batch execution** | **Complete** | `src/scheduler.rs` plans waves of independent pure items; dependent, dynamic and control items are barriers. Results land in slots keyed by input index, so completion order cannot leak. **All 55 workload fingerprints byte-identical across workers 1/2/4/8** and to the frozen v1.0.0 baseline. Golden 527/527 and Full Audit 1215/1215 also pass at workers=4 and 8. |
| **7 — Adaptive parallelism** | **Complete** | Two-dimensional cost model (compute vs payload) with both thresholds calibrated against the worker sweep. The first single-number model was **ordered wrongly** and had to be rebuilt -- see below. Cheap and payload-bound batches stay sequential and measure neutral. |
| **9 — 1/2/4/8 worker benchmark** | **Complete** | Real scaling reported in `docs/V11_PARALLEL_MODEL.md`; no speedup asserted as a requirement. |
| 8 — Pipeline parallelism | **Deferred** | Batch parallelism was the primary goal and is done. Pipelines return only the last value by default, cap at 256 steps, and measured 1.66 us/step, so the available gain is small against real correctness risk. |
| **10 — Concurrency stress tests** | **Complete** | `bench/stress.py`: 60 rounds x 7 workloads x 4 worker counts. No thread leak (threads stay at base + N workers, stable start to end), RSS 7.8 -> 9.2 MiB with no runaway, and every response byte-stable across all rounds at every worker count. |
| **11 — Full regression and v1.1 freeze** | **Complete** | `Cargo.toml` and `Cargo.lock` at **1.1.0**; `SOURCE_INTEGRITY_V11.txt` written (the v1.0.0 record is untouched and is itself hashed inside it); `bench_results/v1.1.0-frozen/` regenerated with **BENCH GATE: PASS**, 0 fingerprint mismatches, 0 regressions, 0 non-deterministic workloads. A new gate pins the MCP server identity -- see below. |

## What the freeze can prove

`SOURCE_INTEGRITY.txt` is the v1.0.0 record and was never rewritten. Checking
today's tree against it:

| | |
|---|---|
| Byte-identical to v1.0.0 | **13 of 16** |
| Intentionally changed | `Cargo.toml` (version), `src/server.rs` (the concurrency work), `CHANGELOG.md` |

The 13 include `src/model.rs` (the MCP schema surface), `src/registry.rs` (all
1,215 operation declarations), `src/engine.rs` (the dispatcher),
`golden/cases.json` (all 527 expected values), the entire full-audit corpus, and
both v1.0.0 validators. So "no operation was added, renamed or renumbered, and
no expected value was edited to make a failure disappear" is not a promise here
-- it is a hash comparison anyone can rerun.

`SOURCE_INTEGRITY_V11.txt` extends the record to 35 files and, unlike the v1.0.0
one, is **verified by a gate**: change any tracked file without regenerating it
and `static_audit_v11.py` fails. Mutation-tested with a one-byte append to
`src/safety.rs`, `docs/V11_PARALLEL_MODEL.md` and `Cargo.lock`, and by deleting
the v1.0.0 record; all four were caught.

## The advertised MCP version stays 1.0.0, and now there is a gate

`Cargo.toml` is 1.1.0. The version a client actually sees is a **separate
hard-coded literal** in `#[tool_handler(...)]`, and it is deliberately left at
`1.0.0`: the release objective is "without changing what the LLM sees", and
`initialize` output is exactly what the LLM sees. Nothing forced this choice --
no test or gate covered that literal at all, in v1.0.0 or on this tree. That is
a hole, not a design: the **instructions string** the model reads sits in the
same annotation and could have been reworded with nothing noticing.

So v1.1 adds a gate pinning the whole block -- server name, advertised version,
and instructions -- by hash. Verified by mutation: bumping the advertised
version, rewording the instructions, and renaming the server were each caught.

If you would rather `initialize` report `1.1.0`, that is a one-line change plus
the pinned hash, and it is your call, not mine -- it changes observable output.

## Parallelism helps a narrow class, and the default stays 1

Measured with paired A/B, worker count the only variable, unanimous sign tests:

| workers | `integrate1000_x64` | `mixed_skew_16` | efficiency |
|---:|---:|---:|---:|
| 2 | **1.75x** | **1.69x** | 85-87% |
| 4 | **2.98x** | **2.72x** | 68-74% |
| 8 | **3.69x** | **2.98x** | 37-46% |

Every other workload measures neutral within +/-2%. That is the intended
outcome: they are correctly kept sequential. Since the benefit is real but
narrow, `DEFAULT_WORKERS` stays 1 and parallelism remains opt-in, exactly as the
approved plan required.

**The first cost model was wrong and the sweep proved it.** It scored items by
`cost_class * argument_size`, which ranked the only workload that scaled *below*
two that gained nothing -- so no threshold could have separated them. Argument
size conflates compute with payload: `stat.sum` over 10,000 numbers is trivial
arithmetic whose time goes into serial parsing, while `signal.dft` over 256 is
quadratic work on a small payload. The model now scores those separately, and
correcting it also surfaced a workload the first model never distributed at all
(`64 x num.integrate`, now 2.8x at four workers). Detail in
`docs/V11_PARALLEL_MODEL.md`.

## Both new gates were verified by mutation

A gate that has never failed is not known to work. Each Phase 4 gate was checked
by deliberately breaking the thing it guards, and restoring afterwards:

| Mutation | Caught by |
|---|---|
| Delete one `control_op` arm (still compiles; silently classifies `udo.uninstall` as Pure) | 4 Rust tests, **and** the v1.1 audit — but only after the audit was fixed |
| Make the dispatcher match string literals again instead of `safety::control_op` | v1.1 audit, 2 failures |

The first mutation initially **passed** the audit: the check searched the whole
of `safety.rs` for the opcode, and it still appeared in `ControlOp::opcode()` and
in the tests. The check now parses the `control_op` function body specifically.
That is exactly the class of silent misclassification the fail-closed
requirement exists to prevent, so it was worth finding.

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
