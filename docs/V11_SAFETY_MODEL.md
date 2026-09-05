# Yekaterina v1.1 — operation safety model

Phase 4. Internal classification of operations into what may run concurrently on
a worker thread and what must not. **Never exposed through MCP**: no new tool, no
new `yk.spec` field, no schema change.

## Requirement

* Fail closed. A new or unknown operation must default to serialized execution.
* No prefix heuristic. `capability_code`/`cost_code` in `registry.rs` classify by
  `starts_with`, which is fine for presentation but would silently misclassify a
  future operation if used for scheduling.
* Testable against actual dispatch behaviour, not against a restatement of it.
* Do not blindly edit 1,215 declarations if a lower-churn design is equally
  explicit and auditable.

## The structural fact the design rests on

```rust
pub fn execute(opcode: &str, args: &[Value]) -> Result<Value, &'static str>
```

`engine::execute` takes no `&self` and no state parameter. It cannot touch the
user registry, the filesystem, or anything the server owns — **that is a type
guarantee, not a convention**. Auditing confirms the modules reachable from it
contain no `unsafe`, no interior mutability, no clock, no RNG, and no I/O; the
only statics in the crate are three write-once `OnceLock` lookup indexes.

The server dispatcher handles exactly eight opcodes itself and delegates
everything else to that function:

```rust
match spec.opcode {
    "udo.formula" | "udo.composite" | "udo.remove"
  | "udo.import"  | "udo.uninstall"              => // mutate registry
    "udo.list" | "udo.export"                    => // read registry snapshot
    "expr.eval"                                  => // stateless, args only
    canonical => engine::execute(canonical, args),
}
```

So: **an operation is stateful only if it has a dispatcher arm of its own**,
because that arm is the only path to `&self`. An operation without one is
physically routed to a function that cannot reach state.

## Options considered

### A. `safety` field on `OperationSpec`

Add a field to the 1,215 `op(...)` declarations in `registry.rs`.

Rejected. To be fail-closed the default must be `Serialized`, so the 1,207 pure
operations would each need an explicit marker — either a sixth argument on every
line, or a second constructor (`pop(...)`) for pure ops.

The second constructor is not merely churn, it **breaks the frozen v1.0.0
audit**: `scripts/static_audit.py`, `scripts/operation_manifest.py` and
`scripts/validate_full_audit.py` all extract opcodes with
`re.findall(r'^\s*op\("([^"]+)"', ...)` and assert exactly 1,215 matches.
Renaming the constructor for 1,207 of them would leave 8.

A sixth argument keeps the regex working but produces a 1,215-line diff whose
correctness cannot be reviewed by reading it, in exchange for information that is
already derivable.

### B. Separate exact table

A `const` list of opcode strings.

Rejected in the fail-open direction: listing the *stateful* ops and treating
everything else as pure means a future stateful operation that nobody adds to
the list is silently parallel — precisely the failure mode the requirement names.

Rejected in the fail-closed direction too: listing the 1,207 *pure* ops is a
second copy of the registry that can drift from it, and drift would be silent
until a test noticed.

### C. Mechanically derived by probing the engine

Classify by calling `engine::execute(opcode, &[])` and treating anything that is
not `OP`/`NYI`/`CONTROL` as engine-handled.

Attractive because it cannot drift — it asks the dispatcher itself. Rejected on
risk: it executes 1,215 real operation bodies with an empty argument list. Most
guard arity first, but that is not guaranteed for all of them, and a single
operation that indexes `args[0]` before checking would panic during
classification. Trading a correctness guarantee for a panic risk in the
scheduler is the wrong trade.

### D. Anchor on the dispatcher's control set — **chosen**

Name the eight control operations in one enum, have the **dispatcher itself
consult it**, and derive safety from it:

```rust
let Some(spec) = registry::resolve(opcode) else {
    return Safety::Serialized;          // dynamic user op, or unknown
};
match control_op(spec.opcode) {
    Some(ControlOp::ExprEval) => Safety::Pure,   // stateless: reads only its args
    Some(_)                   => Safety::Serialized,
    None                      => Safety::Pure,   // routed to engine::execute
}
```

Why this is the lowest-risk option:

* **Zero churn in `registry.rs`.** All frozen audits keep working unmodified.
* **Classifier and dispatcher are the same code.** `server.rs` matches on
  `control_op(...)`, so they cannot disagree — there is nothing to keep in sync.
* **Fail-closed twice over.** An unregistered opcode (a user formula, a
  composite, a typo) returns `Serialized`. And a future *stateful* operation
  cannot be misclassified, because statefulness requires a dispatcher arm, and
  adding a dispatcher arm means adding a `ControlOp` variant; a stateful
  operation without one would be routed to `engine::execute` and simply not
  work.
* **No prefix matching anywhere.**
* **Auditable.** `scripts/static_audit_v11.py` asserts that every `udo.*` opcode
  registered in `registry.rs` appears in the `ControlOp` match, so a new control
  operation cannot be added without classifying it.

The residual risk is a future operation that is stateful *without* going through
the dispatcher — for example one that reads a clock or a file from inside an
engine module. That is out of reach of any classifier and is guarded elsewhere:
the v1.1 audit's forbidden-pattern check rejects `std::process::Command`,
sockets and `unsafe` crate-wide, and thread creation is confined to `pool.rs`.

## Classification of the current 1,215 operations

| Class | Count | Which |
|---|---:|---|
| `Serialized` | 7 | `udo.formula`, `udo.composite`, `udo.remove`, `udo.import`, `udo.uninstall` (mutate), `udo.list`, `udo.export` (read server state) |
| `Pure` | 1,208 | everything else, including `expr.eval` |

Dynamic user operations (`user.*`, `pack.*`) are not in the static registry and
therefore classify as `Serialized`. A user formula is in fact pure, and a
composite cannot mutate anything (`udo.*` is rejected at composite definition
time and at any depth above zero), so both could be parallelised later. v1.1
leaves them serialized deliberately: they are rare, the cost of serializing them
is nil, and fail-closed is the stated preference.

`expr.eval` is classified `Pure` even though the dispatcher handles it, because
`eval_expression(args)` reads only its arguments. It is the one place where the
"has a dispatcher arm" heuristic and actual purity differ, so it is called out
explicitly in the match rather than left to a rule.

## Tests

`src/safety.rs` unit tests and `scripts/static_audit_v11.py` together assert:

1. every registered `udo.*` opcode is a `ControlOp` (a new one cannot slip past);
2. every `ControlOp` opcode is actually registered (no dead entries);
3. exactly 7 of the 1,215 registered operations classify `Serialized`;
4. unknown opcodes, user operations and pack operations classify `Serialized`;
5. aliases and mixed-case spellings classify the same as their canonical opcode,
   because the classifier canonicalises through `registry::resolve` exactly as
   the dispatcher does;
6. the classification of every one of the 1,215 registered opcodes agrees with
   which dispatcher arm actually handles it.
