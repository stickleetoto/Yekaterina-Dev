# Full Capability Audit — Yekaterina v1.0.0

This audit is deliberately separate from the mathematical Golden/property suites.

It uses the frozen `opcodes_alpha12.json` list of exactly **1,387** opcodes. Static validation requires that list to match `src/registry.rs` exactly in count, uniqueness, order, and content. At runtime it then:

1. calls `yk.spec` for all 1,387 opcodes,
2. obtains/reuses a valid canonical fixture,
3. executes each opcode through real `yk.compute`,
4. verifies the returned JSON shape against the `yk.spec` return-type contract,
5. replays all learned fixtures in a fresh process,
6. reruns all **527** Golden cases as a separate correctness oracle.

Fixture selection order:

1. existing non-error Golden case,
2. curated alpha.12 override,
3. compact argument-signature synthesis,
4. bounded family fallback candidates.

`RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat` is strict. It exits non-zero unless:

- registry/manifest scope is exactly 1,387,
- MCP tool surface is exactly 3,
- `yk.spec` is valid 1,387/1,387,
- fixture discovery is 1,387/1,387,
- clean replay + return-type contract is 1,387/1,387,
- Golden oracle is 527/527.

A **1,387/1,387 replay is execution/type coverage, not proof that every mathematical result is correct for every possible input.** Golden and direct property/regression tests are reported separately for correctness evidence.

## hotfix7 fixture discovery

For legacy operation families whose `yk.spec` still reports `args...`, the audit now synthesizes inputs from four bounded sources: direct canonical fixtures, family-specific candidate banks, successful same-family Golden inputs, and a numeric arity sweep. These candidates only establish executable/type coverage; the separate Golden oracle remains the correctness check. The default per-operation search cap is 192 candidates.
