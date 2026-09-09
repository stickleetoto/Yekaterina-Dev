# Yekaterina — v1.2 Release Candidate

**Pure computation. Minimal tokens. Verified evolution.**

Yekaterina is a Rust computation offloader for LLM agents over MCP. The v1.2 release-candidate line expands the deterministic compute layer while preserving the compact three-tool interface used by clients.

> Stable distribution: [stickleetoto/Yekaterina](https://github.com/stickleetoto/Yekaterina) (`v1.0.0`).
> This repository is the active source, verification, and release-preparation tree.

## Release-candidate snapshot

| Metric | v1.2 RC |
|---|---:|
| Registered built-in/control opcodes | **1,410** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| MCP schema footprint | **412 tokens / 1,725 bytes** |
| Golden corpus | **527/527** |
| Full Capability Audit | **1,410/1,410** |
| Crate version | **1.2.0** |
| Advertised MCP version | **1.0.0** (deliberately frozen) |
| Rust toolchain | **1.98.0** |

The operation surface is frozen for RC1. The release-candidate work adds packaging, a one-command demo, Codex setup documentation, and CI gates; it does not add or rename operations.

## 60-second demo

Windows:

```powershell
.\DEMO_WINDOWS.bat
```

Linux/macOS:

```bash
./DEMO_UNIX.sh
```

If a release binary is not already present, the wrapper builds one with `cargo build --locked --release` and then runs the MCP demo.

The demo performs real MCP calls: it verifies the three-tool catalog, computes `20 + 22`, checks exact decimal `0.1 + 0.2`, executes a compact batch, discovers a Welch-test operation with `yk.find`, inspects it with `yk.spec`, and exits only on `DEMO PASS`.

See [SHOWCASE.md](SHOWCASE.md) for the walkthrough.

## Why only three tools?

Yekaterina keeps operation discovery out of `tools/list`.

```text
yk.find     -> discover an operation lazily
yk.spec     -> inspect its compact argument/result contract
yk.compute  -> execute one call, a batch, a pipeline, or supported UDO control
```

The internal registry can grow without enumerating every operation in the model-facing schema. From v1.0.0 to this RC, capability grew from 1,215 to 1,410 registered operations while the MCP tool count and measured schema footprint remained unchanged.

## What is in v1.2?

The v1.2 line adds 195 operations over the v1.1 baseline:

- exact and applied families (`int`, `dec`, `geo`, `fin`, `vec`, `unit`, `pct`);
- statistical inference, distributions, tests, confidence intervals, regression diagnostics;
- multiplicity correction, post-hoc testing, effect sizes, and risk measures.

It also fixes the v1.1 `expr.eval` worker-classification defect. `expr.eval` is serialized because worker reachability, not statelessness, is the safety property that matters.

## Verification

The verified v1.2 baseline at `8361beae` passed:

- `scripts/static_audit_v12.py` — 24 pass / 0 fail;
- lexical, manifest, Golden-manifest, and Full-Audit validators;
- **386** Rust test executions and `cargo clippy`;
- `scripts/verify_v12_operations.py` — 164 independent checks;
- `scripts/verify_statistics.py` — 961 reference values against SciPy, NumPy, and mpmath;
- `scripts/verify_multiplicity.py` — 577 checks against statsmodels/SciPy plus independent definitions;
- Golden **527/527** and Full Capability Audit **1,410/1,410**.

RC1 adds `scripts/rc_gate.py` and runs the user-facing demo in CI so the release packaging and documentation path cannot silently drift away from the executable.

See [docs/V12_RC1.md](docs/V12_RC1.md) for the go/no-go checklist.

## Codex setup

Yekaterina is a local STDIO MCP server. Current Codex clients can register it directly as an MCP server.

See [docs/CODEX_SETUP.md](docs/CODEX_SETUP.md) for CLI and `config.toml` examples.

## Protocol surface

```text
ComputeParams: op / a / ops / pipe / input / all
FindParams:    q / l
SpecParams:    op
```

`src/model.rs` remains byte-identical to the frozen v1.0.0 MCP request-schema surface.

## Security boundary

Compute operations do not expose arbitrary shell execution, arbitrary network access, or arbitrary filesystem access. Formula evaluation uses a bounded internal parser, and batch/pipeline/UDO/numerical workloads have explicit size, depth, and work guards.

Parallel batch execution is opt-in. The default worker count remains 1; `--workers N|auto` or `YEKATERINA_WORKERS` enables the v1.1 worker pool.

## Release lineage

- **v1.0.0** — frozen public stable baseline, 1,215 operations.
- **v1.1.0** — internal performance/concurrency line; no operation additions.
- **v1.2.0 RC1** — 1,410 operations plus release/showcase hardening.

The MCP `initialize` response still advertises `1.0.0` by design. That identity block is hash-gated so crate/version-line work does not silently change what existing MCP clients observe.
