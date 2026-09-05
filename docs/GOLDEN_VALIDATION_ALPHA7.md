# Yekaterina v0.1.0-alpha.7 — Golden Correctness Validation

Alpha.7 is a correctness-hardening release. It does **not** increase the MCP tool surface and does not use any LLM/API.

## Two-layer validation

1. `cargo test --all-targets`
   - internal engine regression tests
   - category-level golden checks
2. `golden/run_golden.py`
   - launches the real release binary over stdio MCP
   - performs MCP `initialize`, `tools/list`, and `yk.compute`
   - validates serialized MCP results against fixed golden answers

## Golden corpus

`golden/cases.json` currently contains **173** cases across:

- matrix
- probability
- statistics
- numerical methods
- signal processing
- finance + unit conversion
- geometry + vectors
- bit/base/exact arithmetic
- general math

The corpus contains exact results, tolerance-based floating-point results, and expected compact error codes.

## Gate

Alpha.7 passes only if:

```text
MCP tools       exactly yk.compute / yk.find / yk.spec
Overall         173/173
Every category  100%
```

Run on Windows:

```powershell
.\VERIFY_WINDOWS.bat
```

Or only the MCP golden suite after a release build:

```powershell
.\RUN_GOLDEN_WINDOWS.bat
```

Outputs:

```text
golden_results/latest/REPORT.md
golden_results/latest/result.json
```

This suite validates correctness, not provider billing tokens. Protocol/token performance remains covered by the separate MCP Real-World Benchmark.
