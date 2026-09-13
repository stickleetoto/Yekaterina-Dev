# Yekaterina-Dev — Legacy Development Repository

> **Development has moved.** Active Yekaterina development now lives in [`stickleetoto/Yekaterina`](https://github.com/stickleetoto/Yekaterina) on the [`dev/v1.4`](https://github.com/stickleetoto/Yekaterina/tree/dev/v1.4) branch.
>
> This repository is retained as a historical migration/provenance and rollback snapshot. Do not start new development here.

## Migration status

- Stable distribution: `stickleetoto/Yekaterina` `main` — **v1.3.0**.
- Active development: `stickleetoto/Yekaterina` `dev/v1.4` — **v1.4 Core Math + Agent Usability**.
- This legacy repository remains preserved to retain the original pre-consolidation commit history and source snapshot.
- Development package metadata remains **1.3.0** until an explicit v1.4 release-promotion phase.
- Live v1.4 built-in/control operation surface: **1,427 operations**.
- v1.3 stable baseline: **1,425 operations**, including **15** native `xfmr.*` operations.
- MCP tools remain exactly **3**: `yk.compute`, `yk.find`, `yk.spec`.
- Golden regression corpus: **527/527** retained.
- Frozen v1.2 full-capability audit surface: **1,410/1,410** retained.
- Advertised MCP initialize version remains `1.0.0` by compatibility policy.

## Historical v1.4 direction

v1.4 is intentionally not an opcode-count race. The line has two goals:

1. **Core Math** — add small, high-leverage deterministic operations that remove common reasoning/calculation work from the LLM.
2. **Agent Usability** — make existing capabilities easier to discover from natural-language intent without expanding the MCP tool schema.

The first development slice added:

- `alg.linear_root(a, b)` — solve `a*x + b = 0`;
- `linalg.solve(matrix, rhs)` — solve square linear systems with pivoted Gaussian elimination;
- semantic `yk.find` intent bridges for phrases such as `solve linear equation`, `system of equations`, `quadratic equation`, `greatest common divisor`, and `matrix inverse`.

## Preserved architecture snapshot

- `src/registry.rs` / `src/engine.rs` — frozen v1.2 baseline, 1,410 operations;
- `src/registry_v13.rs` / `src/engine_v13.rs` — frozen v1.3 aggregate/dispatch layer, 1,425 operations;
- `src/registry_v14.rs` / `src/engine_v14.rs` — v1.4 aggregate/dispatch layer at migration time;
- `src/math_v14.rs` — focused v1.4 math implementations.

## Verification snapshot

The migrated v1.4 source retained:

- `scripts/static_audit_v14.py`;
- aggregate operation manifest: **1,427 total**;
- locked Rust tests, clippy, and release build;
- unchanged three-tool MCP demo;
- retained v1.3 transformer runtime verification;
- v1.4 real-process math + natural-language discovery verification;
- existing v1.2 operation/statistics/multiplicity verifiers;
- Golden regression corpus;
- frozen v1.2 Full Capability Audit;
- benchmark invariants.

For current development state, use [`stickleetoto/Yekaterina/tree/dev/v1.4`](https://github.com/stickleetoto/Yekaterina/tree/dev/v1.4). The files in this repository are retained for historical reference only.
