# Yekaterina v1.2.0

Operation expansion. **1,215 -> 1,387**: 172 operations across nine families,
plus one v1.1 correctness fix. Nothing else changed.

**Unchanged and gate-verified**
- Exactly 3 MCP tools; schema still **412 tokens / 1,725 bytes**. `tools/list`
  never enumerated operations, so this costs the model nothing until it asks.
- `src/model.rs` byte-identical to the frozen v1.0.0 schema surface.
- MCP `initialize` still advertises version `1.0.0`; the crate is 1.2.0.
- 30 error codes; no new code introduced.
- 527/527 Golden with expectations untouched.
- **All 1,215 v1.1 operations proven still registered, unrenamed and in the same
  order** against `full_audit/opcodes_v11_frozen.json`.

**Fixed: `expr.eval` returned `NYI` at more than one worker**
v1.1 classified `expr.eval` as `Pure` because it is stateless. Statelessness is
not the property that classification carries: `Pure` means a worker can run it,
and a worker runs jobs through `engine::execute`, which has no `expr.eval` arm.
A batch mixing `expr.eval` with enough compute to cross the parallel threshold
returned `{"e":"NYI"}` at two or more workers and the right answer at one,
breaking the byte-identical-across-worker-counts invariant. Now `Serialized`,
with no exception to the "has a dispatcher arm" rule. The unit test that existed
asserted the wrong property and pinned the bug; it now asserts that no control
operation is pure. Two batch shapes were added to the cross-worker equivalence
test, which had twelve shapes and no control operation inside a distributed run.
Details in `docs/V11_SAFETY_MODEL.md`.

**Added: statistical inference (69)**
The engine could compute a t statistic and not a p-value, because `prob.*` had
no t, chi-square or F distribution and `special.*` had no regularized incomplete
gamma or beta. Built bottom up in three layers:
- `special` 18 -> 21: `gamma_p`, `gamma_q`, `beta_inc`.
- `prob` 55 -> 77: t, chi-square, F, gamma, beta, lognormal and Weibull, with a
  survival function beside each CDF and quantiles for the first three.
- `test` 10 -> 37: generic p-values, complete tests returning statistic/df/p,
  ANOVA, Mann-Whitney, Wilcoxon, Levene, Bartlett, Kolmogorov-Smirnov, Fisher
  exact, exact binomial, McNemar, four confidence intervals, sample sizing,
  power. New module `src/inference.rs`.
- `reg` 14 -> 31: multiple least squares, standard errors, slope t/p/CI, overall
  F, adjusted R2, AIC, BIC, Durbin-Watson, polynomial, Kendall tau-b, three
  linearisable fits, Theil-Sen.

Tail probabilities are computed rather than subtracted: `prob.normal_sf(10,0,1)`
is `7.6e-24` where `1 - cdf` is exactly `0.0`. Quantiles solve the CDF on the
lower half and the survival function on the upper half, never forming `1 - p` on
the tail being worked. The first quantile implementation did form it and was
caught by the scipy comparison. Details in `docs/V12_STATISTICS.md`.

**Added: exact arithmetic and applied families (103)**
- `int` 8 -> 26 and `dec` 4 -> 20: exact arbitrary-precision arithmetic. `dec`
  had only add/sub/mul/div -- no rounding, no comparison, no aggregation -- so it
  could not be used for money at all. `0.1 + 0.2` is now exactly `0.3`, and
  `dec.round("2.675", 2)` is `2.68` where anything float-backed gives `2.67`.
  Two rounding modes, because ties-away and banker's rounding is a real
  accounting choice rather than a default to inherit silently.
- `geo` 8 -> 30, `fin` 8 -> 29, `vec` 9 -> 22, `unit` 13 -> 20, `pct` 3 -> 9.

`alg.mod_pow`, `alg.mod_inverse`, `alg.ext_gcd` and `alg.floor_div` already
existed and are **not** duplicated: each is capped at u64, i64 or u128 and fails
above that, while the `int.*` versions have no ceiling.

**Deliberately not added**
- Nine drafted operations were dropped as duplicates found by checking the whole
  registry rather than the target families: `pct.error`, `geo.sphere_volume`,
  `geo.sphere_area`, `geo.circle_arc`, `vec.project`, `vec.mean`/`min`/`max`.
- `dec.sqrt` was dropped on principle: a square root is irrational and this
  family's contract is exactness. `dec.pow` refuses a negative exponent likewise.

**Correctness**
- `scripts/verify_statistics.py`: **961 values** against scipy, numpy and mpmath,
  including three points where scipy itself disagrees with mpmath in the deep
  tail and mpmath is used as the reference. Runs in CI.
- `scripts/verify_v12_operations.py`: 164 assertions across 99 operations, with
  Python's arbitrary-precision `int` and `decimal.Decimal` as the oracle for the
  exact families. Exact results are compared as numbers, not strings, and a
  separate assertion rejects exponent notation in any exact result.
- 59 Rust tests built on identities a wrong formula would break: F equals t
  squared on one degree of freedom, both integer division conventions
  reconstructing the dividend, `root^2 <= n < (root+1)^2`, Minkowski at p=1 and
  p=2 collapsing onto Manhattan and Euclidean, reflection being its own inverse,
  a full-turn sector equalling the circle, multiple regression recovering an
  exactly linear relationship, and NPV at the reported IRR being zero.
- clippy's excessive-precision lint caught three unit constants that were one ULP
  wrong (`lb/ft3`, `lb/in3`, `lbf*ft`); the round-trip tests were insensitive to
  that error and would have shipped it.

**Gates**
- `scripts/static_audit_v12.py` reproduces every v1.0.0 and v1.1 gate and
  hash-pins both earlier audits, which are untouched.
- `validate_full_audit.py` proves the v1.1 set is intact and gates the new
  fixture file as strictly as the alpha.12 one. Verified by mutation: renaming or
  reordering an operation with the manifest regenerated to match is caught only
  by this gate, and is caught.
- Full Capability Audit **1,387/1,387**; Golden 527/527 at workers 1, 4 and 8.

# Yekaterina v1.1.0

Performance and internal concurrency only. No operation was added, removed,
renamed or numerically changed.

**Unchanged and gate-verified**
- Exactly 1,215 registered opcodes and exactly 3 MCP tools.
- `src/model.rs` byte-identical to the frozen v1.0.0 schema surface.
- Tool annotation schema hash unchanged; 412 schema tokens / 1,725 bytes.
- MCP server identity unchanged: `initialize` still advertises version `1.0.0`.
  The crate version is 1.1.0; the advertised version is deliberately not tied to
  it, so no client observes a difference. Now pinned by a gate that v1.0.0
  lacked, which also covers the instructions string.
- 527/527 Golden and 1,215/1,215 Full Capability Audit, at 1, 4 and 8 workers.

**Faster**
- Result-size accounting is incremental instead of rescanning every accumulated
  result (`src/limits.rs`); the v1.0.0 algorithm is retained as a test oracle.
- Opcode normalization, step parsing and composite argument resolution borrow
  instead of cloning (`Cow`); four call sites had been cloning arguments only to
  discard them.
- Formula evaluation hoists its environment instead of cloning a variable map
  per evaluation, across `numerical`, `ode`, `series`, `advanced_numerical` and
  `optimization`.
- `lto = "fat"`, `codegen-units = 1`.
- Paired A/B over 33 workloads: median -22%, one workload slower, best
  `num.integrate` batches at 0.157x. Release binary is smaller than v1.0.0.

**Safely parallel, off by default**
- `src/pool.rs`: a fixed worker pool, the only module permitted to create OS
  threads (audit-enforced). Panics are caught in the worker and resumed on the
  request task, reproducing v1.0.0 behaviour exactly. **No new error code.**
- `src/safety.rs`: fail-closed classification, no prefix heuristics. 7 control
  operations serialized, 1,208 pure.
- `src/scheduler.rs`: waves of independent items; results land in slots keyed by
  input index, so completion order cannot reach the response.
- `--workers N|auto` and `YEKATERINA_WORKERS`. **Default is 1**; parallelism is
  opt-in. Measured 2.98x at 4 workers on integration-heavy batches, neutral
  elsewhere.
- All 55 workload response fingerprints are byte-identical across 1/2/4/8
  workers and to the frozen v1.0.0 baseline.

**Fixed**
- A real v1.0.0 defect: a composite could observe a registry mutated mid-run and
  mix two definitions of the same user operation. Reproduced on a copy of the
  v1.0.0 logic (torn read after 383 runs) and fixed by snapshotting the registry
  for the whole composite.
- Concurrent-read tail latency p95 25.7 ms -> 4.2 ms, p99 29.0 -> 4.6 ms.

**Known limitation**
- On an `OUT_LIMIT` abort a wave may execute items sequential v1.0.0 would have
  skipped, and discards them. Observable only if a batch both exceeds
  `OUT_LIMIT` and contains a panicking operation after that point.

# Yekaterina v1.0.0

- Promoted the fully verified `v0.1.0-alpha.12-hotfix9` line to the frozen V1 baseline.
- No compute opcode additions or algorithm changes in the V1 promotion.
- Package/runtime version changed to `1.0.0`.
- Preserved exactly 1,215 registered opcodes and 3 MCP tools.
- Preserved the frozen MCP request schema and 412-token benchmark footprint.
- Acceptance evidence: 527/527 Golden, 1,215/1,215 live spec, fixture, and clean replay/type coverage, Golden oracle 100%.
- Self-regression against alpha.10: +15.28% capability, unchanged schema tokens and 10k wire tokens, hard gate PASS / CURRENT WINS.

# Yekaterina v0.1.0-alpha.12-hotfix9

- Corrected `yk.spec` return metadata for three audit-exposed shape mismatches: `prob.normalize_weights` (`number` -> `number[]`) and `color.contrast_ratio` / `color.relative_luminance` (`number[]` -> `number`).
- Added exact semantic Full Audit fixtures for the final 16 uncovered operations: reshape, weight normalization, subnet count, color alpha/scalar operations, equilibrium temperature, radiation/Stefan flux, Doppler source/full, and frame axis rotations.
- Added a registry regression test so scalar color return contracts cannot drift back to array metadata.
- No opcode additions; the MCP surface remains exactly 3 tools and the fixed audit scope remains 1,215 opcodes.

# Changelog

## v0.1.0-alpha.12-hotfix8

Audit-only hardening. Rust runtime is unchanged from alpha.12-hotfix7/hotfix6.

- added semantic fixture generation for relationship-constrained operations
- added fixed-shape color fixtures (RGBA / YCbCr / YIQ / linear RGB)
- added chain-compatible frame transform/tagged fixtures
- added nested geometry predicate fixtures
- added Carnot hot/cold ordering fixtures
- added network CIDR/IP relationship fixtures
- added structured series coefficient fixtures
- added ODE method-code fixture
- expanded token-to-value inference for structured arguments
- Full Audit now prints all missing opcode names directly to the console
- preserved 1215 opcodes, 3 MCP tools, 527 Golden cases / 44 categories
