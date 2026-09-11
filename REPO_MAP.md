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
| `docs/` | Design and release history. `V11_*` / `V12_*` are retained release-history docs; v1.3 finalization state is summarized in `CURRENT_STATE.md`. |
| `.github/workflows/ci.yml` | The authoritative release-finalization gate list. |
| `target/`, `*_results/` | Build and measurement output. Not analysed. |

Outside the crate, `../../` (`포폴/Yekaterina/`) holds archived source zips and
release bundles for earlier versions plus a `_scratch/` working directory. Those
are snapshots, not part of this tree.

## Entry points

| Entry point | File | Note |
|---|---|---|
| Process `main` | `src/main.rs` | Resolves worker count, then `Yekaterina::with_workers(n).serve(stdio())`. |
| Library root | `src/lib.rs` | Aliases frozen v1.2 registry/engine modules and exposes the aggregate v1.3 live registry/engine surface. |
| MCP tools | `src/server.rs`, `#[tool_router] impl Yekaterina` | `yk.compute`, `yk.find`, `yk.spec`. |
| Historical operation dispatch | `src/engine.rs`, `execute` | Frozen v1.2 execution engine for the legacy surface. |
| Live operation dispatch | `src/engine_v13.rs` | Dispatches `xfmr.*` to transformer-native modules and delegates legacy operations to the historical engine. |
| Historical opcode table | `src/registry.rs`, `OPERATIONS` | Frozen v1.2 catalog: 1,410 operations. |
| Live aggregate opcode table | `src/registry_v13.rs` | v1.2 catalog plus 15 transformer-native operations: 1,425 total. |

## Core modules

### `server` — `src/server.rs`
MCP handler and request orchestration. Owns mutable server state.
- Symbols include `Yekaterina`, `execute_any`, registry mutation helpers, batch execution, pipeline execution, argument resolution, rendering, and expression evaluation.
- MCP surface remains exactly three tools: `yk.compute`, `yk.find`, `yk.spec`.

### `registry_v13` — `src/registry_v13.rs`
The live aggregate read-only registry for v1.3.
- Preserves all 1,410 v1.2 operation names and ordering.
- Appends 15 registered transformer-native operations.
- Total live built-in/control operation count: **1,425**.

### `registry` historical source — `src/registry.rs`
Frozen v1.2 static opcode table and name-resolution implementation.
- Retained as an auditable baseline rather than rewritten during the v1.3 migration.
- Historical operation count: **1,410**.

### `engine_v13` — `src/engine_v13.rs`
Live aggregate dispatcher.
- Routes `xfmr.*` operations to the transformer-native implementation.
- Delegates legacy operations to the frozen v1.2 engine.

### `engine` historical source — `src/engine.rs`
Frozen v1.2 stateless dispatcher for the legacy operation surface.
- Tries precision execution, then family dispatch, then the inline base math/stat set.
- Retained unchanged as migration evidence.

### `safety` — `src/safety.rs`
Execution-safety classification. Never exposed over MCP.
- Classifies pure and serialized/control execution behavior.

### `scheduler` — `src/scheduler.rs`
Batch planning only; executes nothing.
- Handles concurrent versus ordered placement and cost estimation.

### `pool` — `src/pool.rs`
The worker-pool implementation and the only audit-approved location for OS-thread creation.

### `limits` — `src/limits.rs`
Request/result size accounting and result-budget enforcement.

### `user_ops` — `src/user_ops.rs`
User-defined operations: formulas, composites, packs, snapshots, import/export, and validation.

### `storage` — `src/storage.rs`
Generation-numbered JSON snapshots of the user registry.

### `formula` — `src/formula.rs`
Safe arithmetic expression parser/evaluator used by expression execution and several numerical/solver families.

### `model` — `src/model.rs`
MCP schema surface (`ComputeParams`, `FindParams`, `SpecParams`). The model-facing request schema remains compatibility-frozen.

### `precision` — `src/precision.rs`
Exact arithmetic (`int.*`, `dec.*`, `base.*`) over the precision dependencies and consulted before ordinary family dispatch.

## Operation modules

The legacy engine dispatches by opcode family prefix. v1.3 additionally exposes the transformer-native `xfmr.*` surface through the aggregate shim.

| Prefix | Module(s), in fallback order |
|---|---|
| `math` | `extra_math`, then the inline `match` in the historical engine |
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
| `phys`, `eng`, `mech`, `fluid`, `elec`, `optics`, `wave` | same-named domain modules |
| `thermo`, `chem`, `net`, `color`, `astro`, `time`, `geod` | matching domain modules |
| `xfmr` | v1.3 transformer-native modules through `engine_v13` |

## Tests and gates

| Where | What it covers |
|---|---|
| `tests/` | Integration and operation-family regression tests. |
| `golden/` | **527/527** MCP-level correctness regression corpus. |
| `full_audit/` | Frozen v1.2 **1,410/1,410** full-capability evidence plus v1.3 transformer manifests/fixtures. |
| `scripts/static_audit_v13.py` | v1.3 aggregate static audit, including synchronized `Cargo.toml` / `Cargo.lock` package version at **1.3.0**. |
| aggregate operation manifest | **1,425 total / 15 `xfmr.*`** operations. |
| `scripts/verify_v13_transformer_runtime.py` | Real release-binary discovery/spec/execution verification for the promoted transformer surface. |
| `cargo test --locked --all-targets` | Complete Rust test gate. |
| `cargo clippy --locked --all-targets` | Complete lint gate. |
| release build | Locked release build gate. |
| benchmark invariants | Timing-independent schema/determinism/error-envelope checks. |
| `RELEASE_CHECKLIST_V13.md` | Final v1.3 acceptance and promotion procedure. |

## Repository roles

- `stickleetoto/Yekaterina-Dev` — v1.3 release finalization and subsequent development.
- `stickleetoto/Yekaterina` — promoted stable distribution and release evidence, currently on v1.2.0 until v1.3 promotion.

For the authoritative release-finalization snapshot, use `CURRENT_STATE.md`.
