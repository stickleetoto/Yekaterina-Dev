# CURRENT_STATE.md

> Release-candidate snapshot for the public `stickleetoto/Yekaterina-Dev` tree.

## Current milestone

**v1.2.0 RC1 — surface frozen, release/showcase hardening in progress.**

RC1 branches from verified development baseline `8361beae`, which completed the
1,215 -> 1,410 operation expansion. The RC work is intentionally packaging-only:
demo tooling, Codex setup, release documentation, and CI release gates. No runtime
operation, MCP request field, or error code is added by RC1.

| | |
|---|---|
| Crate version | `1.2.0` |
| Advertised MCP version | `1.0.0` (deliberately frozen and hash-gated) |
| Registered opcodes | **1,410** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Schema footprint | **412 tokens / 1,725 bytes** |
| Error codes | **30** |
| Serialized operations | **8** — seven `udo.*` controls plus `expr.eval` |
| Golden corpus | **527/527** |
| Full Capability Audit | **1,410/1,410** |
| Rust edition / toolchain | 2024 / pinned 1.98.0 |
| Default workers | 1 |

## Verified baseline

The `8361beae` baseline recorded:

- `static_audit_v12`: **24 pass / 0 fail**;
- lexical, operation-manifest, Golden-manifest, and Full-Audit validators: PASS;
- **386** Rust test executions and clippy exit 0;
- `verify_v12_operations.py`: **164** independent checks;
- `verify_statistics.py`: **961** values against SciPy, NumPy, and mpmath;
- `verify_multiplicity.py`: **577** checks against statsmodels/SciPy plus independent definitions;
- Golden **527/527**;
- Full Capability Audit **1,410/1,410**.

These are baseline facts, not a claim that the RC packaging commit has already
passed CI. RC1 is promotable only after the RC branch itself passes the full CI
workflow.

## RC1 additions

- `SHOWCASE.md` — reviewer-facing project walkthrough and architecture.
- `DEMO_WINDOWS.bat` / `DEMO_UNIX.sh` — one-command build-and-demo wrappers.
- `tools/demo.py` — standard-library MCP demonstration that exercises
  `yk.compute`, `yk.find`, and `yk.spec`.
- `docs/CODEX_SETUP.md` — current STDIO MCP setup for Codex.
- `docs/V12_RC1.md` — release-candidate invariants and go/no-go checklist.
- `scripts/rc_gate.py` — fast release-surface gate.
- CI runs both `rc_gate.py` and the real MCP demo against the release build.

## v1.2 runtime work completed before RC1

The v1.2 runtime line added **195** operations:

- 103 exact/applied operations across `int`, `dec`, `geo`, `fin`, `vec`, `unit`, and `pct`;
- 69 statistical inference/distribution/regression operations;
- 23 multiplicity, post-hoc, effect-size, and risk-measure operations.

It also fixed the v1.1 `expr.eval` concurrency defect by classifying it
`Serialized`. Worker safety means “the worker dispatcher can execute it,” not
merely “the operation has no mutable state.”

## Release invariants

RC1 must keep all of the following unchanged:

1. exactly **1,410** registered operations;
2. exactly **3** MCP tools;
3. frozen `src/model.rs` request schema;
4. **412-token / 1,725-byte** measured tool schema;
5. MCP `initialize` version **1.0.0** and the existing instructions string;
6. **30** error codes;
7. default worker count **1**;
8. all 1,215 v1.1 operations present, unrenamed, and in order.

A change that intentionally breaks one of these is not an RC1 fix; it starts a
new compatibility decision and must update the corresponding gate and decision
record explicitly.

## Promotion checklist

Before tagging/promoting v1.2:

- GitHub CI green on the RC commit;
- `python scripts/rc_gate.py` PASS;
- `python scripts/static_audit_v12.py` PASS with no pending/fail entries;
- `cargo test --locked --all-targets` PASS;
- `cargo clippy --locked --all-targets` exit 0;
- release build PASS;
- all three Python reference verifiers PASS;
- Golden 527/527;
- Full Capability Audit 1,410/1,410;
- `python tools/demo.py <release-executable>` prints `DEMO PASS`;
- Windows `DEMO_WINDOWS.bat` checked on the packaged executable;
- release archive checksum produced and verified before promotion to
  `stickleetoto/Yekaterina`.

## Deferred / intentionally unchanged

- Pipeline parallelism remains deferred.
- `DEFAULT_WORKERS` remains 1; parallelism is opt-in.
- `OperationSource::Wasm` remains declared but unused.
- `dec.sqrt` remains excluded because the `dec.*` contract is exactness.
- Exact permutation p-values for rank tests remain out of scope.
- `test.ks_normal` accepts the normal parameters rather than estimating them.

## Repository roles

- `stickleetoto/Yekaterina-Dev` — source development, verification, RC work.
- `stickleetoto/Yekaterina` — stable binaries, public release documentation, and
  release evidence. It remains the v1.0.0 stable distribution until v1.2 is
  explicitly promoted.
