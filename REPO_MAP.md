# REPO_MAP.md

> Where things are. Locations and ownership only — no implementation detail.

## Top level

| Path | Responsibility |
|---|---|
| `src/` | The whole engine. Flat module list, no subdirectories. |
| `tests/` | Integration tests. Most use `#[path = "../src/x.rs"] mod x;` includes, not the library target. |
| `benches/micro.rs` | In-process micro suite (`harness = false`). |
| `bench/` | Python MCP benchmark harness (drives the real binary over stdio). |
| `golden/` | 527-case correctness oracle (`cases.json`, `run_golden.py`, `mcp_client.py`). |
| `full_audit/` | Per-opcode capability audit: every registered opcode must execute. |
| `scripts/` | Static audits, integrity generators, operation verifiers. |
| `docs/` | Design and release history. `V11_*` / `V12_*` are current; `*_ALPHA*` are historical. |
| `.github/workflows/ci.yml` | The authoritative gate list. |
| `target/`, `*_results/` | Build and measurement output. Not analysed. |

Outside the crate, `../../` (`포폴/Yekaterina/`) holds archived source zips and
release bundles for earlier versions plus a `_scratch/` working directory. Those
are snapshots, not part of this tree.

## Entry points

| Entry point | File | Note |
|---|---|---|
| Process `main` | `src/main.rs` | Resolves worker count, then `Yekaterina::with_workers(n).serve(stdio())`. |
| Library root | `src/lib.rs` | `pub mod` for every `src/*.rs`. Exists so benches/tests can link in-process. |
| MCP tools | `src/server.rs`, `#[tool_router] impl Yekaterina` | `yk.compute`, `yk.find`, `yk.spec`. |
| Operation dispatch | `src/engine.rs`, `execute` | Takes no `&self` — this is load-bearing, see ARCHITECTURE.md. |
| Opcode table | `src/registry.rs`, `OPERATIONS` | 1,370 `OperationSpec` entries. |

## Core modules

### `server` — `src/server.rs` (52 KB)
MCP handler and request orchestration. Owns all mutable server state.
- Symbols: `Yekaterina` (handler struct), `execute_any`, `execute_any_depth`,
  `mutate_registry`, `run_batch`, `execute_ordered`, `execute_wave`,
  `run_pipeline`, `resolve_args`, `resolve_value`, `render`, `eval_expression`.
- Constants: `MAX_BATCH` 1024, `MAX_PIPE` 256, `MAX_UDO_DEPTH` 32,
  `PARALLEL_COMPUTE_FLOOR` 50_000, `PARALLEL_PAYLOAD_RATIO` 20,
  `PARALLEL_MIN_ITEMS` 2.
- Depends on: `engine`, `formula`, `limits`, `model`, `pool`, `registry`,
  `safety`, `scheduler`, `storage`, `user_ops`.
- Tests: in-file `mod resolution_tests` plus the concurrency suite in the same
  file (torn registry, worker-count equivalence, control-op barriers).

### `registry` — `src/registry.rs` (156 KB)
The static opcode table and name resolution. Almost entirely data.
- Symbols: `OperationSpec`, `OperationSource`, `OPERATIONS`, `resolve`, `search`,
  `capability_code`, `cost_code`; lazy `EXACT_LOOKUP` / `LOOKUP` / `FAMILY_INDEX`.
- Depends on: nothing in-crate.
- Tests: `tests/registry.rs`; count and ordering gated by
  `scripts/validate_full_audit.py`.

### `engine` — `src/engine.rs`
Stateless dispatcher. Tries `precision::execute`, then `dispatch_module` by
family prefix, then an inline `match` for the base `math.*` / `stat.*` set.
Unmatched opcodes return `NYI`.
- Symbols: `execute`, `dispatch_module`, arg helpers `one` / `two` / `array` /
  `binary` / `finite`.
- Depends on: `registry`, `precision`, and every operation module.
- Tests: `tests/engine.rs`, `tests/golden_categories.rs`.

### `safety` — `src/safety.rs`
Execution-safety classification. Never exposed over MCP.
- Symbols: `Safety` (`Pure` / `Serialized`), `ControlOp` (8 variants),
  `ControlOp::opcode`, `ControlOp::ALL`, `control_op`, `classify`.
- Depends on: `registry`.
- Design: `docs/V11_SAFETY_MODEL.md`. Gated by `scripts/static_audit_v11.py`.

### `scheduler` — `src/scheduler.rs`
Batch planning only; executes nothing.
- Symbols: `Placement` (`Concurrent` / `Ordered`), `plan_batch`, `placement_of`,
  `Cost`, `estimated_cost`, `run_cost`, `is_log_linear`, `iteration_driven`;
  `MAGNITUDE_CAP` 100_000, `COMPUTE_CAP` 1_000_000_000.
- Depends on: `registry`, `safety`, `user_ops`.
- Design: `docs/V11_PARALLEL_MODEL.md`.

### `pool` — `src/pool.rs`
The only module permitted to create OS threads (audit-enforced).
- Symbols: `Job`, `Outcome`, `JobResult`, `WorkerPool::{new, workers, run}`,
  `resolve_workers`, `MAX_AUTO_WORKERS` (8).
- Depends on: `engine`.

### `limits` — `src/limits.rs`
Request and result size accounting.
- Symbols: `MAX_VALUE_NODES` 200_000, `MAX_STRING_BYTES` 1_000_000,
  `MAX_RESULT_NODES` 100_000, `MAX_RESULT_BYTES` 1_000_000, `measure_value`,
  `value_too_large`, `ResultBudget`, `values_too_large_replay` (v1.0.0 oracle).

### `user_ops` — `src/user_ops.rs`
User-defined operations: formulas, composites, packs.
- Symbols: `UserRegistry` (`define_formula`, `define_composite`, `remove`,
  `import_pack`, `uninstall_pack`, `lookup`, `find`, `list`, `snapshot`,
  `from_snapshot`, `export_pack`), `UserOp`, `FormulaOp`, `CompositeOp`,
  `UserSnapshot`, `OperationPack`, `PackOp`, `parse_step_ref`, `parse_step`,
  `resolve_composite_args`, `contains_reference`, `execute_formula_spec`.
- Constants: `MAX_USER_OPS` 4096, `MAX_COMPOSITE_STEPS` 256, `MAX_PACK_OPS` 1024,
  `SNAPSHOT_VERSION` 1, `PACK_VERSION` 1.
- Tests: `tests/user_ops.rs`. Pack format: `docs/UDO_PACKS.md`.

### `storage` — `src/storage.rs`
Generation-numbered JSON snapshots of the user registry.
- Symbols: `default_store_dir`, `load`, `save`; `KEEP_SNAPSHOTS` 3, file name
  `snapshot-<20 digits>.json`.
- Store dir resolution: `YEKATERINA_HOME` → `%LOCALAPPDATA%\Yekaterina\udo`
  (Windows) → `$XDG_DATA_HOME/yekaterina/udo` → `~/.yekaterina/udo` → `.yekaterina/udo`.

### `formula` — `src/formula.rs`
Safe arithmetic expression parser/evaluator behind `expr.eval` and user formulas;
also the inner loop of the `num.` / `ode.` / `optimize.` / `series.` solvers.
- Symbols: `eval`, `Env`; `MAX_EXPR_LEN` 4096, `MAX_PARAMS` 32, `MAX_DEPTH` 64.
- Tests: `tests/formula.rs`; `tests/formula_bit_identity.rs` against
  `tests/fixtures/formula_bits.json` (bit-exact f64 oracle).

### `model` — `src/model.rs`
The entire MCP schema surface: `ComputeParams`, `FindParams`, `SpecParams`.
Frozen and byte-identical to v1.0.0.

### `precision` — `src/precision.rs`
Exact arithmetic (`int.*`, `dec.*`, `base.*`) over `num-bigint` / `bigdecimal`.
Consulted by `engine::execute` **before** family dispatch.

## Operation modules

`engine::dispatch_module` routes on the family prefix of the opcode. To find an
implementation: take the prefix, read this table, then grep the opcode string.

| Prefix | Module(s), in fallback order |
|---|---|
| `math` | `extra_math`, then the inline `match` in `engine` |
| `stat`, `reg`, `test` | `inference` → `stats` → `advanced_stats` |
| `prob` | `probability` → `advanced_probability` |
| `num` | `numerical` → `advanced_numerical` |
| `mat` | `matrix` → `advanced_matrix` |
| `signal` | `signal` → `advanced_signal` |
| `int`, `dec`, `base` | `precision` (resolved before family dispatch) |
| `bit`, `base` | `radix` |
| `vec` | `vector` |
| `linalg` | `deep_linalg` |
| `geo` | `geometry` |
| `pct`, `fin`, `unit` | `practical` |
| `alg` | `algebra` |
| `cplx` | `complex_math` |
| `special` | `special_functions` |
| `optimize`, `ode`, `series`, `curve`, `frame`, `predicate` | same-named module |
| `verify` | `verification` |
| `data` | `data_ops` |
| `disc` | `discrete` |
| `info` | `information` |
| `phys`, `eng`, `mech`, `fluid`, `elec`, `optics`, `wave` | same-named module (`physics`, `engineering`, `mechanics`, `fluids`, `electrical`, `optics`, `waves`) |
| `thermo`, `chem`, `net`, `color`, `astro`, `time`, `geod` | `thermodynamics`, `chemistry`, `networking`, `color`, `astronomy`, `time_ops`, `geodesy` |

The order inside `stat` / `reg` / `test` is deliberate: `inference` is asked
first because `advanced_stats` claims those whole prefixes and would answer `OP`
for anything it does not implement. See the comment at `src/engine.rs:119`.

## Tests and gates

| Where | What it covers |
|---|---|
| `tests/engine.rs` | Numerical operations across families. |
| `tests/golden_categories.rs` | Category-level golden expectations. |
| `tests/registry.rs` | Alias resolution and search ranking. |
| `tests/user_ops.rs` | Composite/formula definition and validation. |
| `tests/formula.rs`, `tests/formula_bit_identity.rs` | Expression parser; bit-exact regression. |
| in-file `mod tests` in `server`, `safety`, `scheduler`, `pool` | Concurrency, classification, planning. |
| `scripts/static_audit_v12.py` | Current release gate: 1,370 opcodes, crate version 1.2.0, tool surface, integrity manifest; hash-pins the v1.0.0 and v1.1 audits. |
| `scripts/validate_full_audit.py` | 1,370-opcode manifest; proves all 1,215 v1.1 opcodes are still registered, unrenamed and in order. |
| `scripts/verify_v12_operations.py` | Drives the real binary over MCP; independent recomputation of the v1.2 operations. |
| `scripts/lexical_rust_audit.py`, `scripts/operation_manifest.py`, `scripts/validate_golden_manifest.py` | Lexical and manifest gates. |
| `golden/run_golden.py` + `scripts/check_golden_result.py` | 527 MCP-level cases. |
| `full_audit/run_full_audit.py` | Every registered opcode executes against a fixture. |
| `bench/run_bench.py`, `bench/paired_ab.py`, `bench/stress.py` | Performance, A/B decisions, concurrency stress. |
