# KNOWN_ISSUES.md

> Only issues verifiable from the code, tests, comments or `docs/`. Each entry
> says how it was established. No speculation.

## Confirmed

None outstanding.

## Resolved since the last snapshot

### `expr.eval` in a parallel wave returned `NYI` — fixed
The static finding was confirmed by execution before being acted on:
`[["signal.dft", <256 samples>], ["expr.eval", {"e":"1+1"}]]` returned `2.0` at
`--workers 1` and `{"e":"NYI"}` at `--workers 2` and `4`.

`safety::classify` now returns `Serialized` for every operation with a dispatcher
arm, with no exception. The unit test that asserted the opposite -- that
`expr.eval` was the one pure control operation -- pinned the bug rather than
catching it and was replaced by one asserting that no control operation is pure,
plus a second checking the real invariant directly. Two batch shapes containing a
control operation inside a distributed run were added to
`server::tests::batch_results_are_identical_at_every_worker_count`, which had
twelve shapes and none of that kind. Reintroducing the old classification makes
them fail with `shape "control op inside a distributed run" differs at
workers=2`, so the regression is pinned rather than assumed.
See `docs/V11_SAFETY_MODEL.md`.

### `advanced_stats` would have swallowed new `test.*` and `reg.*` operations — fixed
`advanced_stats::execute` claims both prefixes with `op.starts_with` and returns
`Some(Err("OP"))` for anything it does not implement, so `.or_else` never ran and
the first 27 operations added to `test.*` all returned `OP`.
`engine::dispatch_module` now asks `inference::execute` first; it matches an
explicit list, so it cannot shadow an operation that already worked.

### `SOURCE_INTEGRITY_V12.txt` was stale — regenerated
`scripts/gen_source_integrity_v12.py` was run once the batch settled;
`scripts/static_audit_v12.py` passes on this tree.

### Documentation lagged the tree — closed
`CHANGELOG.md` and `docs/` describe all 1,387 operations, and
`docs/V12_STATISTICS.md` covers the inference batch.

## Known limitations

- **Pipelines are sequential.** No parallelism, capped at `MAX_PIPE` 256 steps,
  and only the last value is returned unless `all` is set.
- **Parallelism helps a narrow class.** 1.75×/2.98×/3.69× at 2/4/8 workers on
  `integrate1000_x64`, 1.69×/2.72×/2.98× on `mixed_skew_16`; efficiency falls to
  37–46% at 8 workers. Every other measured workload is neutral within ±2%.
  `docs/V11_PARALLEL_MODEL.md`.
- **A panicking operation still kills the request.** The pool contains the panic
  so the worker thread survives, but the payload is resumed on the request task,
  which dies with no response sent — deliberately identical to v1.0.0.
  `src/pool.rs` module doc.
- **`MAX_AUTO_WORKERS` is 8**, deliberately conservative, because each in-flight
  job may hold a large intermediate `Value`.
- **Hard request/result ceilings**, returned as `LIMIT` / `OUT_LIMIT`:
  `MAX_BATCH` 1024, `MAX_PIPE` 256, `MAX_UDO_DEPTH` 32, `MAX_VALUE_NODES` 200,000,
  `MAX_STRING_BYTES` 1,000,000, `MAX_RESULT_NODES` 100,000, `MAX_RESULT_BYTES`
  1,000,000, `MAX_USER_OPS` 4096, `MAX_COMPOSITE_STEPS` 256, `MAX_PACK_OPS` 1024,
  `MAX_EXPR_LEN` 4096, `MAX_PARAMS` 32, `MAX_DEPTH` 64.
- **`alg.*` modular and combinatorial operations are capped** at `u64` / `i64` /
  `u128` and return `LIMIT` or `TYPE` above that. The uncapped path is `int.*`.
- **`storage::load` silently falls back.** A corrupt or unparseable newest
  snapshot is skipped in favour of an older generation, and an empty registry is
  returned if none loads. No error surfaces to the client.
- **Only 3 snapshot generations are kept** (`KEEP_SNAPSHOTS`).
- **~120 pre-existing clippy style warnings** in the frozen v1.0.0 numerical
  modules, 116 of them `suspicious_else_formatting`. `cargo clippy --locked
  --all-targets` still exits 0. New modules carry zero warnings.
- **The advertised MCP version is `1.0.0` while the crate is 1.2.0.** Intentional
  and gated, but it does mean `initialize` does not identify the build.
- **Tests are split across two linking styles.** The v1.0.0 corpus uses
  `#[path = "../src/x.rs"]` includes; only `benches/micro.rs` and newer code link
  the library target. A module included by `#[path]` is compiled twice.

## Intentionally deferred

- **Phase 8, pipeline parallelism** — deferred in
  `docs/V11_DEVELOPMENT_STATUS.md`: measured 1.66 µs/step, so the available gain
  is small against real correctness risk.
- **`DEFAULT_WORKERS = 1`** — raising it is a deliberate release decision, not a
  default to drift into (`src/main.rs`).
- **`OperationSource::Wasm`** — declared, `#[allow(dead_code)]`, no Wasm
  operation exists.
- **`udo.reset`** — named in `src/safety.rs:242` as a plausible future control
  operation, not added.
- **A second `pool` backend** — `Job` / `JobResult` are plain owned data
  specifically as the seam a process or remote backend would attach to, but no
  trait hierarchy is introduced because the audit forbids spawning a subprocess,
  so a second implementation cannot exist yet (`src/pool.rs` module doc).
- **`dec.sqrt`** — dropped on principle; `dec.pow` refuses negative exponents.
- **Style cleanup of the frozen numerical modules** — refused as churn.
