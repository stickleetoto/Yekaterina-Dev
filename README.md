# Yekaterina v1.0.0

**Pure computation. Minimal tokens.**

Yekaterina is a Rust compute offloader for LLM agents over MCP. v1.0.0 is the frozen production baseline promoted from the fully verified `v0.1.0-alpha.12-hotfix9` line.

> Internal compute capability may grow without growing the LLM-facing MCP tool surface.

## V1 frozen surface

- **1,215** registered built-in/control opcodes
- exactly **3 MCP tools**
  - `yk.compute`
  - `yk.find`
  - `yk.spec`
- MCP request structs frozen to the alpha.10 surface
- **527/527** MCP Golden cases
- **1,215/1,215** live `yk.spec` coverage
- **1,215/1,215** live MCP execution fixtures
- **1,215/1,215** clean replay / return-type contract coverage
- Golden oracle correctness: **100%**

The 1,215/1,215 Full Capability Audit proves live execution and return-type coverage. It is not a claim that every mathematical result is proven correct for every possible input.

## Self-regression result

v1.0.0 was accepted by comparing the release candidate against the verified alpha.10 performance baseline:

| Metric | alpha.10 baseline | v1.0.0 candidate | Result |
|---|---:|---:|---|
| Registered opcodes | 1,054 | 1,215 | **+15.28% capability** |
| MCP tools | 3 | 3 | unchanged |
| Schema tokens | 412 | 412 | unchanged |
| 10k wire tokens | 159,794 | 159,794 | unchanged |
| 10k arithmetic accuracy | 100% | 100% | unchanged |
| 10k MCP time | 24.6739 ms | 26.8091 ms | +8.65% latency |
| Hard regression gate | — | PASS | **CURRENT WINS** |

The primary V1 architectural result is that capability increased by 15.28% over alpha.10 while the LLM-facing tool count, schema-token footprint, and 10k wire-token cost stayed unchanged.

## Deep numerical families

V1 includes the alpha.12 deep numerical layer:

| Family | Ops | Focus |
|---|---:|---|
| `linalg.*` | 20 | eigen/SVD/pseudoinverse/PCA/least-squares |
| `special.*` | 18 | Gamma/Beta/erf/Bessel/zeta/Lambert W |
| `optimize.*` | 16 | Brent/golden/Newton/BFGS/Nelder-Mead |
| `ode.*` | 15 | Euler/Heun/RK4/adaptive RK45 |
| `series.*` | 16 | convergence/Taylor/Fourier/Chebyshev |

V1 also retains the verification/trust families from alpha.11 (`verify.*`, `frame.*`, `curve.*`, `predicate.*`), exact BigInt/BigDecimal operations, batch execution, pipelines, Formula UDOs, Composite UDOs, persistent snapshots, pack import/export/uninstall, indexed discovery, deterministic alias resolution, and resource guards.

## Verify on Windows

Requires Rust **1.98.0**. Direct dependencies are exact-pinned. On the first local verification, `Cargo.lock` is generated if absent; all subsequent Cargo commands in the verifier use `--locked`.

```powershell
.\VERIFY_WINDOWS.bat
.\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat
```

Required V1 acceptance target:

```text
registered opcodes          1215
MCP tools                   3
golden cases                527/527
spec coverage               1215/1215
fixture coverage            1215/1215
clean replay/type contract  1215/1215
golden oracle               100%
release build               PASS
```

## Protocol surface

```text
ComputeParams: op / a / ops / pipe / input / all
FindParams:    q / l
SpecParams:    op
```

No V1 feature expands the MCP parameter schema.

## Security boundary

Compute operations do not expose arbitrary shell execution, network access, or arbitrary filesystem access. Formula evaluation uses the bounded internal expression parser. Batch, pipeline, expression, UDO, and numerical workloads have explicit size/depth/work guards.

## Version history

`v1.0.0` is a version promotion of the verified `v0.1.0-alpha.12-hotfix9` runtime. No new compute opcode was added during the V1 promotion. Historical alpha architecture, hardening, and validation documents remain under `docs/` for traceability.

See:

- `docs/V1_RELEASE.md`
- `docs/HANDOFF.md`
- `docs/VALIDATION_ALPHA12.md`
- `full_audit/README.md`
- `CHANGELOG.md`
