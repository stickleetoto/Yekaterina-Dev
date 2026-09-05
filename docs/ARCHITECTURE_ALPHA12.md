# Architecture — alpha.12 Deep Numerical Mathematics

## Invariants

1. MCP surface remains exactly `yk.compute`, `yk.find`, `yk.spec`.
2. `model.rs` request structs remain byte-identical to the verified alpha.10/alpha.11 schema source.
3. Deep mathematics is exposed only as internal opcodes.
4. Every alpha.12 deep opcode has a curated Full Capability Audit fixture.
5. Algorithms are bounded: matrix dimensions, iterations, series terms, and adaptive ODE steps all have explicit limits.
6. Deep solvers return or expose numerical diagnostics where practical.
7. No shell execution, filesystem access, network access, or unsafe host code is introduced by math operations.

## Families

- `linalg.*` uses a symmetric Jacobi eigensolver as the reusable numerical core for eigen/SVD/pseudoinverse/PCA paths.
- `special.*` uses deterministic local approximations/series; no external runtime is required.
- `optimize.*` evaluates safe Yekaterina expressions and bounds iteration counts.
- `ode.*` uses scalar safe expressions in `(x,y)` with fixed-step and adaptive RK methods.
- `series.*` covers finite/accelerated sums and common approximation bases.

## Protocol consequence

The new math catalog does not add MCP tools or request fields. Discovery remains lazy through `yk.find` and `yk.spec`; exact execution remains `yk.compute`.

## alpha.12-hotfix4 numerical/release boundaries

Hotfix4 does not expand the opcode catalog. It hardens existing deep numerical operations and release gates.

- The SVD/pseudoinverse path scales the input before forming `A^T A`, reducing overflow/underflow risk and using relative singular-value cutoffs. It is still an `A^T A` + symmetric eigensolve design, not a Golub-Reinsch/direct bidiagonal SVD. Extremely ill-conditioned matrices can therefore lose more relative accuracy than specialist numerical libraries.
- Iterative/optimization/ODE/series operations use explicit work/iteration/tolerance guards. Valid syntax is not permission for unbounded CPU work.
- Full Capability Audit 1,215/1,215 means every registered opcode was discoverable, executable through MCP, and matched its declared return-type shape. Mathematical correctness evidence is reported separately by Golden/property tests.
- Snapshot rename is the persistence commit point. Post-commit directory sync/old-snapshot cleanup is best-effort to avoid reporting a failure after the new snapshot is already authoritative. The local UDO store remains a single-writer design; multi-process locking is not provided.
- Rust is pinned to 1.98.0 and direct dependencies are exact-pinned. A fresh verification generates `Cargo.lock` once; retain that lockfile for reproducible future builds.
