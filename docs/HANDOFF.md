# CURRENT HANDOFF — Yekaterina v1.0.0

V1 is the frozen baseline promoted from `v0.1.0-alpha.12-hotfix9`. No new compute capability was added during promotion.

Frozen targets:

- 1,215 built-in/control opcodes
- exactly 3 MCP tools
- 527/527 MCP Golden cases
- 1,215/1,215 `yk.spec` coverage
- 1,215/1,215 Full Capability Audit fixture coverage
- 1,215/1,215 clean replay / return-type contract coverage
- Golden oracle 100%
- schema tokens 412
- 10k fixed arithmetic wire tokens 159,794
- self-regression hard gate PASS / CURRENT WINS

Windows acceptance:

```powershell
.\VERIFY_WINDOWS.bat
.\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat
```

See `V1_RELEASE.md` for the release evidence and claim boundaries.

---

# CURRENT HANDOFF — Yekaterina v0.1.0-alpha.11

Alpha.11 is the Verification & Numerical Trust candidate. It starts from the verified alpha.10-hotfix2 baseline and adds 76 internal trust opcodes without changing the three MCP tools or their parameter schemas.

Current targets:

- 1130 built-in/control opcodes
- 453 MCP golden cases / 39 categories
- `verify.*`, `frame.*`, `curve.*`, `predicate.*`
- alpha.10 model schema source hash frozen
- alpha.10 normalized tool annotations frozen

First command on Windows:

```powershell
.\VERIFY_WINDOWS.bat
```

Do not claim alpha.11 runtime acceptance until that local Rust + real MCP validation passes.

See `HANDOFF_ALPHA11.md` and `VERIFICATION_ALPHA11.md`.

---

# CURRENT HANDOFF — Yekaterina v0.1.0-alpha.10

## Identity

Yekaterina is a Rust MCP compute offloader designed to expose large deterministic compute capability through a tiny protocol surface.

Invariant:

> Internal operation count may grow; exposed MCP tool count remains exactly 3.

Public MCP tools:

```text
yk.compute
yk.find
yk.spec
```

## Historical measured baseline

Verified on Windows in the alpha.6/alpha.7 lineage:

```text
MCP tools                     3
schema tokens                 412
schema reduction vs Arithma   97.56%
10k wire-token reduction      80.32%
10k arithmetic accuracy       100%
recovery                      PASS
UDO persistence               PASS
```

These are MCP-only protocol measurements.

## Alpha.10 candidate

Alpha.10 raises the catalog from **709 to 1054 executable built-in/control opcodes**.

New packs:

```text
time.*     40
geod.*     40
thermo.*   40
mech.*     40
fluid.*    35
elec.*     40
optics.*   35
wave.*     35
data.*     40
```

Scaling changes:

- exact opcode/alias resolution remains indexed with `OnceLock<HashMap>`
- family-scoped discovery has its own lazy family index
- engine module selection uses a prefix dispatcher instead of a linear handler probe chain
- `yk.find` remains bounded and `yk.spec` remains compact

## Correctness gate

Golden corpus:

```text
391 cases
35 categories
```

Run:

```powershell
.\VERIFY_WINDOWS.bat
```

Required:

```text
static audit PASS
cargo test PASS
clippy PASS
release build PASS
MCP tools exactly 3
golden 391/391
35 categories at 100%
```

Then rerun the MCP Real-World Benchmark with the alpha.10 release binary.

## Critical invariants

1. Never expose one MCP tool per operation.
2. Do not grow `tools/list` when adding operation families.
3. Large exact integer/decimal results remain strings at the JSON boundary.
4. Compute operations do not execute shell commands or perform network access.
5. Formula/Composite UDOs remain bounded and host-sandboxed.
6. Persistent UDO graphs remain acyclic and referentially valid.
7. Errors stay compact and machine-readable.
8. Batch/pipeline intermediates stay inside the process whenever possible.
9. New operations need registry entry + implementation + representative golden coverage.
10. Keep exact dispatch indexed and domain routing bounded as operation count grows.

## Next target after alpha.10 acceptance

- rerun MCP protocol benchmark and record fresh alpha.10 result
- deepen advanced matrix eigen/SVD paths
- add more probability distributions and special functions
- consider generated/split registry catalog before 2,000+ operations
- benchmark `yk.find` generic-vs-family discovery at 1K+ operations
