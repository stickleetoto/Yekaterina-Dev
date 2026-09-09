# Yekaterina v1.3 Compute Program — alpha.1 contract

Status: development contract for `feature/v1.3-compute-program`.

## Goal

Yekaterina v1.3 introduces a compute-program surface that lets an MCP client describe several dependent computations and execute them through one `yk.compute` call.

The alpha.1 implementation is deliberately conservative: the program is validated as a named dependency graph, then compiled to the already-verified v1.2 sequential `pipe` executor. Parallel DAG execution is deferred to alpha.2.

The public MCP tool count remains exactly three:

- `yk.compute`
- `yk.find`
- `yk.spec`

## Request shape

```json
{
  "input": {
    "x": [1, 2, 3, 4]
  },
  "program": {
    "steps": [
      {
        "id": "sum",
        "op": "stat.sum",
        "a": ["$input.x"]
      },
      {
        "id": "scaled",
        "op": "math.mul",
        "a": ["$sum", 10]
      }
    ],
    "return": "scaled"
  }
}
```

Expected result:

```json
{"r":100.0}
```

`program` is mutually exclusive with top-level `op`, `ops`, and `pipe`.

## References

alpha.1 supports:

- `$input` — the complete top-level `input` value;
- `$input.key` — object field lookup;
- `$input.key.0` — array indexes may appear in input paths;
- `$step_id` — the complete result of another program step.

References are resolved recursively inside arrays and objects.

Result subpaths such as `$step.field` are intentionally not part of alpha.1.

Any string beginning with `$` is reference syntax. Literal strings beginning with `$` are therefore not representable in alpha.1, matching the existing v1.2 pipeline restriction.

## Step IDs

A step ID:

- is 1 to 64 bytes;
- begins with ASCII `A-Z`, `a-z`, or `_`;
- then contains only ASCII letters, digits, `_`, or `-`;
- is unique within the program;
- may not be `input`, which is reserved.

Forward references are legal. Declaration order does not define execution order.

## Graph semantics

Before execution Yekaterina:

1. parses the program;
2. validates IDs and references;
3. builds dependencies from `$step_id` references;
4. rejects cycles;
5. computes a deterministic stable topological order;
6. enforces graph limits;
7. rewrites named references to the v1.2 numeric `$N` pipeline references;
8. executes through the existing pipeline path.

When several nodes are ready at once, alpha.1 selects the earliest declared node. This makes compilation deterministic.

## Demand-driven return

When top-level `all` is false, only the dependency closure of the requested return step is compiled and executed. Pure unused branches are omitted.

`program.return` names the requested step. If it is omitted, the last declared step is the return target.

Because mutating `udo.*` operations are forbidden inside a program, pruning unused branches cannot discard required side effects.

When top-level `all` is true, all program steps are executed and the existing pipeline all-results response is returned in deterministic topological execution order.

## Limits

alpha.1 limits:

- maximum steps: 256;
- maximum dependency depth: 32;
- maximum step ID length: 64 bytes;
- existing request/result node, string, and byte budgets remain in force after compilation.

## Safety

`udo.*` operations are rejected inside Compute Programs in alpha.1. Programs are computation graphs, not mutation scripts.

Built-in pure operations, serialized read-only computation such as `expr.eval`, formulas, and existing user operations may otherwise execute through the same `execute_any` path used by v1.2 pipelines.

## Error classes

The compiler distinguishes:

- `ARG` — malformed program shape;
- `NAME` — invalid or duplicate step ID;
- `REF` — missing input path or unknown step reference;
- `CYCLE` — dependency cycle;
- `CONTROL` — forbidden `udo.*` operation;
- `LIMIT` — node/depth/ID limits exceeded.

alpha.1 performs compilation during request deserialization so malformed programs are rejected before execution. A later alpha may route these compiler errors through the compact compute error envelope without changing the graph semantics.

## Compatibility

Existing `op`, `ops`, `pipe`, `input`, and `all` behavior is unchanged when `program` is absent.

The v1.2 stable repository and `v1.2.0` tag remain frozen. All v1.3 work occurs in Yekaterina-Dev.

## alpha.1 acceptance tests

The initial gate covers at least:

- one-step program;
- dependent steps;
- top-level input reference;
- nested input/reference resolution;
- forward step reference;
- deterministic topological order;
- demand-driven unused-branch pruning;
- unknown reference rejection;
- duplicate/invalid ID rejection;
- cycle rejection;
- control-operation rejection;
- node/depth limits;
- legacy ComputeParams deserialization unchanged when `program` is absent.

## Deferred to alpha.2

- parallel execution of ready DAG nodes;
- worker-pool cost planning for graph levels;
- named multi-result envelopes;
- result subpath references;
- dry-run/cost-plan mode;
- recipes/macros.
