# DECISIONS.md

> Only decisions whose reason is stated in the code or in `docs/`. Nothing here
> is inferred. Each entry cites where the reason is recorded.

## Operations are never enumerated in the MCP schema

**Decision:** exactly three tools; the 1,370-opcode table is reachable only
through `yk.find` and `yk.spec`.
**Reason:** `tools/list` never enumerated operations, so adding operations costs
the model nothing until it asks. The schema footprint stayed at 412 tokens /
1,725 bytes across 1,215 → 1,318 operations. — `CHANGELOG.md`, `docs/V12_OPERATIONS.md`
**Reconsider only if:** the token budget stops being the binding constraint.

## Safety is derived from the dispatcher, not from per-operation metadata

**Decision:** `safety::classify` reads `safety::control_op`, the same function
`server.rs` dispatches on. No prefix matching, no per-operation annotation.
**Reason:** `engine::execute` takes no `&self`, so anything routed there is pure
*by type*. Statefulness therefore requires a dispatcher arm, and a dispatcher arm
requires a `ControlOp` variant — a stateful operation someone forgot to classify
would simply not work rather than silently race. Classifier and dispatcher being
the same code means they cannot drift apart. Zero churn in `registry.rs`, so all
frozen audits keep working. — `src/safety.rs` module doc, `docs/V11_SAFETY_MODEL.md`
**Reconsider only if:** `engine::execute` ever needs a state parameter.

## Default is `Serialized` (fail-closed)

**Decision:** unregistered opcodes, user formulas, composites and pack operations
are all `Serialized`.
**Reason:** `Pure` must follow from a positive structural fact, never from
"nothing matched". A mutation test that deleted one `control_op` arm initially
*passed* the audit — the check searched all of `safety.rs` and the opcode still
appeared in `ControlOp::opcode()`. The check now parses the `control_op` body
specifically. — `src/safety.rs` module doc, `docs/V11_DEVELOPMENT_STATUS.md`

## The cost model scores compute and payload separately

**Decision:** `scheduler::Cost` carries two numbers; a run is distributed only
when compute is both absolutely large (`>= 50_000`) and large relative to payload
(`>= payload * 20`).
**Reason:** the first model scored `cost_class * argument_size` and was **ordered
wrongly** against the 1/2/4/8 worker sweep — 16× `signal.dft(256)` scored 131,592
and measured 0.40×, while 4× `stat.sum(10000)` scored 320,032 and measured 1.07×.
No threshold can separate those, because argument size conflates O(n²)
trigonometry on a small payload with trivial arithmetic whose time goes into
serial parsing. — `src/scheduler.rs` doc comment on `estimated_cost`,
`docs/V11_PARALLEL_MODEL.md`

## The scheduler may use prefix heuristics; `safety.rs` may not

**Decision:** `registry::cost_code` classifies by `starts_with`; `safety.rs`
deliberately contains no prefix matching.
**Reason:** a wrong cost estimate can only pick a worse schedule, never a
different result. A wrong safety classification is a race.
— `src/safety.rs` module doc, `src/scheduler.rs` doc comment on `estimated_cost`

## `DEFAULT_WORKERS = 1`

**Decision:** parallelism is opt-in via `--workers N|auto` or `YEKATERINA_WORKERS`.
**Reason:** measured benefit is real but narrow — 1.75×–3.69× on two workloads,
every other workload neutral within ±2%. Compatibility and predictability come
first. `MAX_AUTO_WORKERS` is capped at 8 because each in-flight job may hold a
large intermediate `Value`, so worker count is also a memory multiplier.
— `src/main.rs`, `src/pool.rs`, `docs/V11_DEVELOPMENT_STATUS.md`

## Unparseable `--workers` is a hard error, not a fallback

**Decision:** exit code 2 with a message on stderr.
**Reason:** a client that asked for four workers and got one should be told.
Diagnostics go to stderr because stdout carries the MCP protocol.
— `src/main.rs`

## OS threads, not tokio tasks

**Decision:** `src/pool.rs` creates real threads, parked on a condvar, created
once rather than per request; it is the only module allowed to do so.
**Reason:** compute here is CPU-bound and unyielding — `signal.dft` over 2,048
samples took 47 ms on the frozen baseline — and running that on a runtime worker
blocks the I/O driver and the response sink. Ad-hoc threading elsewhere is how
ordering bugs enter a codebase that currently has none, so the audit forbids it.
— `src/pool.rs` module doc

## `panic = "unwind"` in the release profile

**Decision:** left at the default despite `lto = "fat"` and `codegen-units = 1`.
**Reason:** the worker pool must contain a panicking job rather than aborting the
process. Jobs run under `catch_unwind`; the payload is carried back and resumed
on the request task, reproducing v1.0.0 behaviour exactly — the task dies, no
response is sent, the server survives. — `Cargo.toml`, `src/pool.rs` module doc

## `lto = "fat"` over `thin`

**Decision:** fat LTO.
**Reason:** measured — 4.8% faster (median, 2/7 cases slower) *and* 258 KB
smaller. Binary went 5,468,160 → 4,286,464 B, which is 536 KB below v1.0.0
despite the added library target. Build time 44 s → 124 s.
— `docs/V11_DEVELOPMENT_STATUS.md`

## The registry is copy-on-write behind an `Arc`

**Decision:** readers clone an `Arc` once; a composite runs entirely against that
snapshot. The snapshot is taken lazily at the dynamic lookup.
**Reason:** v1.0.0 took a fresh read lock per lookup, including once per composite
step, so a concurrent `udo.remove` or `udo.import` between two steps let a
composite observe two registry versions. Reachable in v1.0.0 — `rmcp` dispatches
every `tools/call` as an independent task — and hidden only by the serial request
pattern of stdio clients. Holding one read guard across the composite is not a
fix: tokio's `RwLock` is write-preferring, so a second read acquisition from the
same task while a writer waits would deadlock. Taking the snapshot eagerly cost a
lock acquisition per batch item and measured 5% slower on a 1,000-item batch.
— `src/server.rs` doc comment on `Yekaterina::user_ops`

## Persist before publishing, behind a store gate

**Decision:** `mutate_registry` clones, applies, calls `storage::save`, then swaps
the `Arc`. Ordering is enforced by a dedicated mutex, not the registry lock.
**Reason:** v1.0.0 mutated the live registry, wrote, and rolled back on failure —
holding the registry write lock across the fsync and leaving a window where
memory and disk disagreed. The new order reaches the same observable outcome with
the write lock held only for the swap. Concurrent read tail latency improved 6.4×
(p99 29.0 ms → 4.6 ms). — `src/server.rs`, `docs/V11_DEVELOPMENT_STATUS.md`

## Result order is structural, never recovered by sorting

**Decision:** wave results are written into slots keyed by input index.
**Reason:** so completion order is structurally unable to leak out.
— `src/scheduler.rs` module doc, `src/pool.rs` doc comment on `Job::index`

## Argument resolution borrows when it can

**Decision:** `resolve_args` returns `Cow::Borrowed` when no `$`-prefixed string
appears in the argument tree.
**Reason:** v1.0.0 rebuilt the tree unconditionally, so a single 10,000-element
array argument was deep-copied on every item even with no references present —
and `parse_step` had already copied it once before that. Measured 0.839× on
10,000-element arguments and 0.749× on pipeline-256; the substituting path is
neutral at 0.995×. — `src/server.rs` doc comment on `resolve_args`

## `ResultBudget` folds incrementally

**Decision:** each appended value is charged once instead of replaying the whole
result vector.
**Reason:** the v1.0.0 form was a fold recomputed from scratch per step, so
validation cost grew quadratically: 40× the items cost 77× the time against
roughly 2 µs of actual compute per item. The traversal order and saturation
behaviour of `measure_value` are frozen because they determine *which item index*
first crosses a limit. — `src/limits.rs` module doc

## The advertised MCP version stays `1.0.0` and is now gated

**Decision:** the crate is 1.2.0; `#[tool_handler(version = "1.0.0")]` is
unchanged, and the whole block — name, version, instructions — is hash-pinned.
**Reason:** the release objective was "without changing what the LLM sees", and
`initialize` output is exactly what the LLM sees. The gate exists because nothing
covered that literal before: the instructions string sits in the same annotation
and could have been reworded with nothing noticing. Verified by mutation —
bumping the version, rewording the instructions, and renaming the server were
each caught. — `docs/V11_DEVELOPMENT_STATUS.md`
**Reconsider only if:** the owner decides `initialize` should report the crate
version; it is a one-line change plus the pinned hash, and it changes observable
output.

## Exact families reject inexact results

**Decision:** `dec.sqrt` was dropped; `dec.pow` refuses a negative exponent.
**Reason:** a square root is irrational and the family's contract is that every
result is exact. `math.sqrt` already covers the approximate case.
— `docs/V12_OPERATIONS.md`

## `int.*` extends `alg.*` rather than duplicating it

**Decision:** `int.mod_pow`, `int.mod_inverse` and friends were added even though
`alg.*` has same-named operations.
**Reason:** reading the implementations showed every `alg.*` version is capped at
`u64`, `i64` or `u128` and returns `LIMIT` or `TYPE` above that; the `int.*`
versions have no ceiling. A test does modular exponentiation with a 128-bit
modulus that `alg.mod_pow` cannot accept. — `docs/V12_OPERATIONS.md`

## Duplicate names are removed, not shipped

**Decision:** eight of 112 drafted operations were dropped as duplicates
(`pct.error`, `geo.sphere_volume`, `geo.sphere_area`, `geo.circle_arc`,
`vec.project`, `vec.mean` / `min` / `max`).
**Reason:** a second name for an existing operation makes `yk.find` worse, not
better — it splits results for one concept across two entries and gives the model
a choice that does not matter. Reuse across families is allowed only when the
*type* differs (`vec.lerp` vs `math.lerp`). — `docs/V12_OPERATIONS.md`

## `fin.irr` uses bisection, not Newton

**Decision:** fixed bracket, fixed iteration count.
**Reason:** it is then a pure function of its input on every platform. Newton
would converge faster but would make the result depend on the starting guess and
the floating-point path — not a trade worth making in an engine whose contract is
byte-identical output. — `docs/V12_OPERATIONS.md`

## Tail probabilities come from survival functions

**Decision:** `src/inference.rs` uses `*_sf` throughout rather than `1 - cdf`.
**Reason:** a p-value is a tail probability, and that is exactly where the
subtraction throws away the digits that matter. — `src/inference.rs` module doc

## `inference` is asked before `stats` and `advanced_stats`

**Decision:** dispatch order for the `stat` / `reg` / `test` prefixes.
**Reason:** `advanced_stats` claims those whole prefixes and would answer `OP` for
anything it does not itself implement, swallowing every operation added later.
`inference` matches an explicit list, so going first cannot shadow an existing
operation. — `src/engine.rs:119`

## The crate gained a library target for v1.1

**Decision:** `src/lib.rs` exists; `benches/micro.rs` links against it. The
v1.0.0 test corpus deliberately keeps its `#[path]` includes.
**Reason:** v1.0.0 was binary-only, so tests had to textually include sources and
the engine could not be benchmarked in-process at all. Leaving the old tests on
`#[path]` keeps them proving what they proved before. — `src/lib.rs` module doc

## Clippy warnings in frozen numerical modules are left alone

**Decision:** ~120 pre-existing style lints (116 of them
`suspicious_else_formatting`) are not fixed.
**Reason:** editing frozen numerical code for style is exactly the churn the
release forbids. Zero warnings are allowed in any new module.
— `docs/V11_DEVELOPMENT_STATUS.md`

## Fixtures for new operations are explicit, not inferred

**Decision:** every new operation carries a fixture in
`full_audit/fixtures_v12.json` rather than relying on the runner's 192-candidate
inference sweep.
**Reason:** discovery becomes a lookup instead of a search, and the fixture
doubles as the documented example. Kept out of `overrides_alpha12.json` because
that file's own gate requires it to hold exactly the 85 deep-family operations.
— `docs/V12_OPERATIONS.md`

## `Cargo.lock` is validated, not regenerated, in CI

**Decision:** no `cargo generate-lockfile` step before the `--locked` builds.
**Reason:** regenerating first made every later `--locked` step check the freshly
generated file instead, defeating the reproducibility guarantee the exact version
pins exist to provide. — `.github/workflows/ci.yml`

## Serialized means "a worker cannot run it", not "it has state"

**Decision:** `safety::classify` returns `Serialized` for every operation with a
dispatcher arm, including the stateless `expr.eval`. No exception.
**Reason:** v1.1 classified `expr.eval` as `Pure` because it reads only its
arguments. That is true and irrelevant: a worker runs a job through
`engine::execute`, which has no `expr.eval` arm, so a distributed batch returned
`NYI` at more than one worker. The classification carries reachability, not
purity. — `src/safety.rs`, `docs/V11_SAFETY_MODEL.md`

## Tail probabilities are computed, never subtracted

**Decision:** every distribution ships a survival function beside its CDF, and
quantiles solve the CDF on the lower half and the survival function on the upper
half.
**Reason:** `1 - prob.normal_cdf(10,0,1)` is exactly `0.0` where the true tail is
`7.6e-24`, and solving `cdf(x) = 1 - 1e-8` discards eight digits before the
search starts. The first quantile implementation did form `1 - p` and was caught
by the scipy comparison. — `docs/V12_STATISTICS.md`

## Iterative numerics use fixed iteration counts

**Decision:** the incomplete gamma and beta, every quantile, and `fin.irr` run a
fixed number of steps rather than iterating to a convergence test.
**Reason:** the engine's contract is byte-identical output. A loop whose trip
count depends on the input can differ between platforms. The caps are far above
what convergence needs, so they bound the loop rather than truncate it.
— `src/special_functions.rs`, `src/advanced_probability.rs`

## Statistical references are external and independent

**Decision:** `scripts/verify_statistics.py` checks against scipy, numpy and
mpmath, and CI installs them.
**Reason:** a reimplementation of the same algorithm is not a check on it. Where
scipy and mpmath disagree — three points in the deep tail — mpmath at 50 digits
is the reference and the disagreement is reported rather than absorbed.
— `docs/V12_STATISTICS.md`

## `inference::execute` is asked before `advanced_stats::execute`

**Decision:** the `stat`/`reg`/`test` dispatcher arm tries `inference` first.
**Reason:** `advanced_stats::execute` claims the whole `test.` and `reg.`
prefixes with `op.starts_with` and answers `OP` for anything it does not
implement, which swallowed all 27 new `test.*` operations. `inference` matches an
explicit list, so going first cannot shadow an operation that already worked.
— `src/engine.rs`
