# Yekaterina-Dev

Development repository for Yekaterina.

## Status

- Stable distribution: `stickleetoto/Yekaterina` is **v1.3.0**.
- Active development line: **v1.4 — Core Math + Agent Usability**.
- Development package metadata remains **1.3.0** until an explicit v1.4 release-promotion phase.
- Live v1.4 built-in/control operation surface: **1,427 operations**.
- v1.3 baseline retained: **1,425 operations**, including **15** native `xfmr.*` operations.
- v1.4 additions in the first slice: **2** focused equation-solving operations.
- MCP tools remain exactly **3**: `yk.compute`, `yk.find`, `yk.spec`.
- Golden regression corpus: **527/527** retained.
- Frozen v1.2 full-capability audit surface: **1,410/1,410** retained.
- Advertised MCP initialize version remains `1.0.0` by compatibility policy.

## v1.4 direction

v1.4 is intentionally not an opcode-count race. The line has two goals:

1. **Core Math** — add small, high-leverage deterministic operations that remove common reasoning/calculation work from the LLM.
2. **Agent Usability** — make existing capabilities easier to discover from natural-language intent without expanding the MCP tool schema.

The first development slice adds:

- `alg.linear_root(a, b)` — solve `a*x + b = 0`;
- `linalg.solve(matrix, rhs)` — solve square linear systems with pivoted Gaussian elimination;
- semantic `yk.find` intent bridges for phrases such as `solve linear equation`, `system of equations`, `quadratic equation`, `greatest common divisor`, and `matrix inverse`.

Exact canonical/alias ownership remains stronger than semantic ranking, so existing v1.3 lookups keep their deterministic behavior.

## Additive architecture

Historical layers are preserved rather than rewritten:

- `src/registry.rs` / `src/engine.rs` — frozen v1.2 baseline, 1,410 operations;
- `src/registry_v13.rs` / `src/engine_v13.rs` — frozen v1.3 aggregate/dispatch layer, 1,425 operations;
- `src/registry_v14.rs` / `src/engine_v14.rs` — active v1.4 aggregate/dispatch layer;
- `src/math_v14.rs` — focused v1.4 math implementations.

This keeps release evidence separable while allowing the live runtime to move forward.

## Verification

The v1.4 CI path requires:

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

For the authoritative development snapshot, see [`CURRENT_STATE.md`](CURRENT_STATE.md).
