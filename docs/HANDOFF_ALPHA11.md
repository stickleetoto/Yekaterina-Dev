# Yekaterina alpha.11 handoff

## Baseline

alpha.10-hotfix2 is the frozen verified baseline:

- 1054 executable opcodes
- 3 MCP tools
- 391/391 golden
- 412 schema tokens in the MCP real-world benchmark
- 80.32% 10k wire-token reduction vs the benchmark Arithma build

## alpha.11 changes

- 1130 opcodes total
- +23 `verify.*`
- +20 `frame.*`
- +16 `curve.*`
- +17 `predicate.*`
- 453 golden cases / 39 categories
- no new MCP tools
- no new MCP parameter fields

## First command on Windows

```powershell
.\VERIFY_WINDOWS.bat
```

Do not accept alpha.11 until release build and real stdio MCP golden validation are both 100%.

## After local acceptance

Rerun the MCP Real-World Benchmark and compare against alpha.10-hotfix2. Required protocol invariant: tools remain 3 and schema should stay at the 412-token baseline (or investigate any drift before continuing).

## Next depth candidates

Only after alpha.11 is accepted:

- SVD / pseudoinverse / condition diagnostics
- symmetric eigen solvers
- robust optimization diagnostics
- richer ODE integrators with step-convergence checks
- special functions with accuracy envelopes
