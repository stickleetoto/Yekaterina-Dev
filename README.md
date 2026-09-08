# Yekaterina — Development

**Development repository for Yekaterina Core.**

This is where Yekaterina is actively built, optimized, benchmarked, and verified before changes are promoted to the stable distribution.

> Looking for the stable release? Go to **[stickleetoto/Yekaterina](https://github.com/stickleetoto/Yekaterina)**.

**Pure computation. Minimal tokens. Verified evolution.**

Yekaterina is a high-density Rust compute offloader for LLM agents over MCP. Its core design goal is simple:

> Grow internal compute capability without growing the LLM-facing tool surface.

### Development snapshot

- Active development: **v1.2.0**
- Language: **Rust**
- MCP surface: **3 tools**
- Registered operations: **1,387**
- MCP schema footprint: **412 tokens / 1,725 bytes**
- Stable distribution: **[stickleetoto/Yekaterina](https://github.com/stickleetoto/Yekaterina)**

## What Yekaterina does

Instead of exposing hundreds or thousands of separate tools to an LLM, Yekaterina keeps the protocol surface compact:

```text
yk.compute
yk.find
yk.spec
```

Behind those three tools is a large registry of deterministic computation primitives covering arithmetic, exact numbers, statistics, probability, linear algebra, optimization, ODEs, signal processing, geometry, finance, units, verification utilities, user-defined operations, pipelines, and more.

The result is a compute layer designed to give AI agents more capability without forcing them to carry an ever-growing tool schema in context.

## Current development status — v1.2.0

The current development line expands the frozen v1.1 surface from **1,215 → 1,387 operations** while keeping the MCP interface unchanged.

### v1.2 highlights

- **172 new operations**
- statistical inference stack
  - regularized special functions
  - t / chi-square / F distributions
  - hypothesis tests
  - regression helpers
- expanded exact arithmetic and applied families
  - `int.*`
  - `dec.*`
  - `geo.*`
  - `fin.*`
  - `vec.*`
  - `unit.*`
  - `pct.*`
- exact decimal behavior for money-style arithmetic
- deep-tail probability handling that avoids catastrophic `1 - cdf` cancellation
- duplicate-operation review before registry expansion
- source-integrity gates for the new release line

A concurrency defect introduced during v1.1 development was also found and fixed: `expr.eval` could return `NYI` when included in a distributed batch with more than one worker. The classifier, scheduler assumptions, and cross-worker equivalence coverage were corrected together.

## Parallel execution — v1.1 foundation

v1.1 introduced a worker pool and ordered parallel batch scheduler without expanding the protocol surface.

Measured compute-bound batch speedups:

| Workers | Speedup |
|---:|---:|
| 2 | **1.75×** |
| 4 | **2.98×** |
| 8 | **3.69×** |

The default remains **1 worker**. Parallel execution is enabled only where the scheduler can preserve the behavioral contract.

Outputs are placed by input position rather than completion order so worker timing cannot leak into result ordering.

## Verification

Yekaterina is developed around regression gates rather than feature count alone.

Current v1.2 verification includes:

- **1,387 / 1,387** Full Capability Audit at workers 1 and 8
- **527 / 527** Golden MCP cases retained across workers 1, 4 and 8
- **323 Rust test executions**
- **961 statistical reference values** checked against SciPy, NumPy and mpmath
- **164 exact-operation assertions** using Python arbitrary-precision integers and `decimal.Decimal` as external oracles
- `static_audit_v12`: **24 pass / 0 fail**
- the original **3 MCP tools** preserved
- schema footprint preserved at **412 tokens / 1,725 bytes**
- all **1,215 v1.1 operations** proven still registered, unrenamed and in order

The capability audit proves live execution and return-type coverage. It is not a claim that every mathematical result is formally proven for every possible input.

## Stable v1.0 baseline

The stable v1.0.0 line established the compatibility surface that later development preserves:

- **1,215** registered built-in/control opcodes
- exactly **3 MCP tools**
- **527 / 527** MCP Golden cases
- **1,215 / 1,215** live `yk.spec` coverage
- **1,215 / 1,215** live MCP execution fixtures
- **1,215 / 1,215** clean replay / return-type contract coverage
- Golden oracle correctness: **100%**

The v1.0 release increased capability by **15.28%** over the earlier alpha.10 baseline while keeping tool count, schema-token footprint and 10k wire-token cost unchanged.

## Deep numerical families

The stable base already includes substantial numerical coverage:

| Family | Focus |
|---|---|
| `linalg.*` | eigen / SVD / pseudoinverse / PCA / least-squares |
| `special.*` | Gamma / Beta / erf / Bessel / zeta / Lambert W |
| `optimize.*` | Brent / golden / Newton / BFGS / Nelder-Mead |
| `ode.*` | Euler / Heun / RK4 / adaptive RK45 |
| `series.*` | convergence / Taylor / Fourier / Chebyshev |

Yekaterina also includes exact BigInt/BigDecimal operations, batch execution, pipelines, Formula UDOs, Composite UDOs, persistent snapshots, pack import/export/uninstall, indexed discovery, deterministic alias resolution, verification families, and resource guards.

## Protocol surface

```text
ComputeParams: op / a / ops / pipe / input / all
FindParams:    q / l
SpecParams:    op
```

The development rule is that internal capability can expand without casually expanding this LLM-facing schema.

## Security boundary

Compute operations do not expose arbitrary shell execution, network access, or arbitrary filesystem access.

Formula evaluation uses a bounded internal expression parser. Batch, pipeline, expression, UDO, and numerical workloads have explicit size, depth, and work guards.

## Repository model

```text
stickleetoto/Yekaterina
    └─ stable public distribution

stickleetoto/Yekaterina-Dev
    └─ active development, optimization, verification and release preparation
```

The development repository intentionally keeps historical validation documents and integrity records so changes can be audited against earlier frozen baselines.

## Verification on Windows

Requires Rust **1.98.0** for the current project toolchain.

```powershell
.\VERIFY_WINDOWS.bat
.\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat
```

Additional v1.2 verification scripts live under `scripts/`.

## Development philosophy

Yekaterina follows a recurring pattern:

```text
Build
  → Measure
  → Verify against known behavior
  → Freeze the contract
  → Extend without silently regressing it
```

Performance work is accepted only when correctness and compatibility gates continue to hold.

## License

Apache License 2.0.
