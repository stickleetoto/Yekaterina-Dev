# CURRENT_STATE.md

> Active development snapshot for `stickleetoto/Yekaterina-Dev`.

## Current milestone

**Yekaterina v1.4 development is open with a focused Core Math + Agent Usability scope.**

The stable distribution `stickleetoto/Yekaterina` is now **v1.3.0**. v1.4 development is additive over that released 1,425-operation baseline and deliberately avoids rewriting the frozen v1.2/v1.3 registry and engine layers.

| | |
|---|---|
| Stable distribution line | **1.3.0** |
| Active development line | **v1.4 Core Math + Agent Usability** |
| Development package metadata | `1.3.0` until explicit v1.4 promotion |
| Advertised MCP initialize version | `1.0.0` (compatibility-gated) |
| v1.3 released built-in/control operations | **1,425** |
| v1.4 current built-in/control operations | **1,427** |
| v1.4 new math operations | **2** |
| Native transformer operations retained | **15** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Golden corpus | **527/527** regression gate retained |
| Frozen v1.2 Full Capability Audit | **1,410/1,410** retained |
| Rust edition / toolchain | 2024 / pinned 1.98.0 |
| Default workers | 1 |

## v1.4 first slice

### Core Math

- `alg.linear_root(a, b)` — solves `a*x + b = 0`, returning `DOMAIN` when `a == 0`.
- `linalg.solve(matrix, rhs)` — solves non-singular square linear systems up to the bounded implementation limit using partial pivoting; singular systems return `DOMAIN`.

These are deliberately high-leverage operations rather than broad family expansion.

### Agent Usability

`yk.find` keeps the same MCP request/response schema but gains an additive semantic discovery layer. Exact canonical names and existing aliases remain strongest, then common natural-language intents can bridge to existing or new operations.

Initial intent examples include:

- `solve linear equation` -> `alg.linear_root`
- `system of equations` -> `linalg.solve`
- `quadratic equation` -> `alg.quadratic_roots`
- `greatest common divisor` -> `alg.gcd_many`
- `matrix inverse` -> `mat.inverse`

No fourth MCP tool is introduced.

## Layering

- `src/registry.rs` / `src/engine.rs` — frozen v1.2 baseline, 1,410 operations.
- `src/registry_v13.rs` / `src/engine_v13.rs` — frozen v1.3 layer, 1,425 operations.
- `src/registry_v14.rs` / `src/engine_v14.rs` — active v1.4 aggregate layer, currently 1,427 operations.
- `src/math_v14.rs` — v1.4-only math implementation.

`src/lib.rs` exposes v1.2 and v1.3 as explicit historical modules and routes the live `registry` / `engine` surface through v1.4.

## Verification contract

The active v1.4 branch must pass all of the following before merge:

- v1.4 static audit, including v1.2 integrity preservation and final-v1.3 layer pins;
- aggregate **1,427 unique-op** manifest check;
- `cargo test --locked --all-targets`;
- `cargo clippy --locked --all-targets`;
- locked release build;
- unchanged MCP showcase demo;
- retained v1.3 transformer runtime verifier;
- v1.4 math + natural-language discovery real-process verifier;
- v1.2 independent operation/statistics/multiplicity verifiers;
- Golden **527/527** regression;
- frozen v1.2 Full Capability Audit **1,410/1,410** under strict mode;
- benchmark invariants.

## Scope discipline

The current slice does **not** change the MCP schema, release metadata, error vocabulary, worker defaults, transformer behavior, or persistent UDO model. Larger math-family expansion and broader semantic discovery should only follow after this first slice passes end-to-end verification.
