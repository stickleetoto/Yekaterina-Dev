# ARCHITECTURE.md

> System-level view. Longer design rationale lives in `docs/V11_SAFETY_MODEL.md`,
> `docs/V11_PARALLEL_MODEL.md` and `docs/V12_OPERATIONS.md`.

## System overview

A single-process MCP server over stdio. It exposes **three** tools and a table of
**1,370** opcodes. The design premise is that the operation table must not cost
the model anything: `tools/list` never enumerates operations, so the schema
footprint stays at 412 tokens / 1,725 bytes no matter how large the table grows.
Operations are discovered only on demand, through `yk.find` and `yk.spec`.

## Major components

### 1. MCP boundary — `src/server.rs` + `src/model.rs`
- **Responsibility:** parse `ComputeParams` / `FindParams` / `SpecParams`,
  enforce request limits, orchestrate execution, render one JSON string back.
- **Inputs:** an MCP `tools/call`.
- **Outputs:** a JSON string, always either `{"r": ...}` or `{"e": "<CODE>"}`.
- **Owns:** the user-operation registry `Arc`, the store gate, the store
  directory, the worker pool.

`yk.compute` accepts four mutually exclusive shapes, checked in this order:
`pipe` (sequential, `$0`/`$input` references, last value only unless `all`),
`ops` (batch), `op` + `a` (single), otherwise `ARG`.

### 2. Stateless execution — `src/engine.rs` and the operation modules
- **Responsibility:** compute one operation from its arguments.
- **Interface:** `execute(opcode: &str, args: &[Value]) -> Result<Value, &str>`.
  No `&self`, no state parameter, no I/O.
- **Boundary:** this signature *is* the safety boundary. Anything routed here is
  structurally unable to reach server state.

### 3. Safety classification — `src/safety.rs`
- **Responsibility:** decide what may leave the request task.
- **Interface:** `classify(opcode) -> Safety`, `control_op(canonical) -> Option<ControlOp>`.
- **Boundary:** `server.rs` dispatches on `control_op` directly, so the
  classifier and the dispatcher are the same code and cannot drift apart.
- **Fail-closed:** the default is `Serialized`. Unregistered opcodes, user
  formulas, composites and pack operations are all serialized. `Pure` is only
  ever concluded from a positive structural fact, never from "nothing matched".

### 4. Planning — `src/scheduler.rs`
- **Responsibility:** turn a batch into a `Vec<Placement>`; estimate cost.
- **Interface:** `plan_batch(&[Value]) -> Vec<Placement>`, `run_cost(&[Value]) -> Cost`.
- **Boundary:** produces a plan and nothing else. A wrong cost estimate can only
  pick a worse schedule, never a different result — which is why this module is
  allowed the prefix heuristics that `safety.rs` forbids.

### 5. Execution pool — `src/pool.rs`
- **Responsibility:** run pure jobs off the tokio runtime.
- **Interface:** `Job { index, opcode, args }` in, `Result<Value, &str>` out —
  plain owned data, no borrow of server state. That is the seam a future process
  or remote backend would attach to.
- **Boundary:** the only module allowed to create OS threads.

### 6. Persistence — `src/user_ops.rs` + `src/storage.rs`
- **Responsibility:** define, look up and persist user operations.
- **Interface:** `UserRegistry` methods; `storage::{load, save}` over
  generation-numbered snapshots, newest-first with fallback to older generations
  on parse failure.

## Data flow

```text
stdio (MCP)
    |
    v
rmcp tool_handler  ->  Yekaterina::{compute,find,spec}     [src/server.rs]
    |
    +-- request_too_large  (limits::measure_value)          [src/limits.rs]
    |
    +-- pipe -> run_pipeline ---+
    +-- ops  -> run_batch ------+--> execute_any_depth
    +-- op   -------------------+          |
                                           |
              scheduler::plan_batch <------+  (batch only)
                     |
        +------------+------------+
        |                         |
   Placement::Ordered      Placement::Concurrent
        |                         |
        |                   execute_wave
        |                     |         |
        |            (below threshold)  (parallel)
        |                     |         |
        |                     |    pool.run --> worker threads
        |                     |                     |
        v                     v                     v
   registry::resolve  ---> safety::control_op
        |                         |
        +-- Some(ControlOp) -> mutate_registry / snapshot read / eval_expression
        |                         |
        |                    storage::save  (behind store_gate, fsync before publish)
        |
        +-- None -> engine::execute -> precision / dispatch_module / inline match
        |
        +-- not a built-in -> UserRegistry::lookup -> formula | composite (recurse)
                                           |
                                           v
                                 ResultBudget::admit  [src/limits.rs]
                                           |
                                           v
                                   render -> JSON string
```

## Important interfaces

| Interface | Contract |
|---|---|
| `engine::execute(&str, &[Value])` | Stateless. Adding a `&self` parameter would collapse the entire safety argument. |
| `safety::control_op(canonical)` | Single source of truth for "the server handles this itself". |
| `scheduler::Placement` | `Concurrent` means the item is independent of every other item in its run. |
| `pool::Job` / `JobResult` | Owned data only, keyed by input index. |
| `UserRegistry` | Cloned, mutated, persisted, then published as a new `Arc`. Never mutated in place while visible. |
| `#[tool_handler(name, version, instructions)]` | The literal text the model sees at `initialize`. Hash-pinned. |

## Invariants that must hold

1. **Three tools, frozen schema.** Exactly `yk.compute`, `yk.find`, `yk.spec`;
   412 tokens / 1,725 bytes. `src/model.rs` byte-identical to the v1.0.0 record.
2. **Advertised MCP version stays `1.0.0`**, independently of the crate version.
   It is a separate hard-coded literal, deliberately, and it is gated.
3. **`engine::execute` takes no `&self`.** Every claim about parallel safety
   rests on this.
4. **Statefulness requires a `ControlOp` variant.** An operation that mutates or
   reads server state and lacks one is routed to `engine::execute` and simply
   fails, rather than silently racing.
5. **Result order is structural.** Results are written into slots keyed by input
   index. Ordering is never recovered by sorting, so completion order cannot leak.
6. **Byte-identical output across worker counts.** Response fingerprints at
   workers 1/2/4/8 must match each other and the frozen v1.0.0 baseline. This is
   a hard, non-statistical gate.
7. **Registry reads are snapshot-coherent.** One composite executes entirely
   against one `Arc<UserRegistry>`; a concurrent mutation cannot be observed
   mid-composite.
8. **Persist before publish.** `storage::save` completes before the in-memory
   `Arc` is swapped, so disk never lags memory.
9. **Mutations are serialized by the store gate**, not by the registry lock, so
   readers are never blocked behind an fsync.
10. **Errors are values, not aborts.** A failed batch item becomes
    `{"e":"CODE"}` in its slot; later items still run and may reference it.
11. **Batch references point strictly backwards.** A forward `$N` resolves
    against a not-yet-populated vector and yields `REF`.
12. **Threads are created only in `src/pool.rs`.**
13. **Error codes are a closed set of 30.** Adding one is a release decision.
14. **Exact families return exact values.** `int.*` and `dec.*` never return an
    approximation and never use exponent notation.
15. **`udo.*` cannot originate inside a composite** — rejected at definition time
    and at any depth above zero. Recursion is capped at `MAX_UDO_DEPTH` 32.
