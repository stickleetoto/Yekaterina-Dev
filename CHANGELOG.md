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
