# Yekaterina benchmark harness

Scope: **MCP-only. No LLM, no network, no external service.**

## Why this exists

`benchmark_results/alpha3-mcp-proof/` and `benchmark_results/mcp-realworld-alpha6/`
contain published numbers, but the harness that produced them was never committed
and the "Yekaterina Self-Regression Benchmark v0.1" referenced by
`RELEASE_CHECKLIST_V1.md` is not in the repository either. Those numbers therefore
could not be reproduced or compared against.

This harness replaces that gap with a versioned, deterministic, in-repo suite.

## Files

| File | Role |
|---|---|
| `workloads.py` | Frozen workload payloads. Same id means the same bytes forever. |
| `bench_client.py` | Thin subclass of the frozen `golden/mcp_client.py`, adding argv passthrough (`--workers`) and process metrics. The golden client is a correctness-gate artifact and is never modified. |
| `run_bench.py` | Driver: environment capture, cold start, workload timing, fingerprinting, reporting, baseline comparison. |

## Running

```powershell
python .\bench\run_bench.py --exe .\target\release\yekaterina.exe --out .\bench_results\latest
```

Compare against the frozen v1.0.0 baseline:

```powershell
python .\bench\run_bench.py --exe .\target\release\yekaterina.exe --out .\bench_results\latest --compare .\bench_results\v1.0.0-frozen\result.json
```

Worker sweep (v1.1 only; v1.0.0 binaries ignore the flag):

```powershell
python .\bench\run_bench.py --exe .\target\release\yekaterina.exe --workers 4 --out .\bench_results\w4
```

Useful flags: `--runs N` (whole-suite repeats, default 3), `--only SUBSTRING`
(filter workload ids), `--cold-reps N`.

## Methodology

* **Warmup then measure.** Each workload runs `warmup` unmeasured iterations
  before `reps` measured ones.
* **Pooled percentiles, visible variance.** p50/p95 (and p99 when n >= 100) are
  computed over samples pooled across runs. Per-run medians are also reported as
  `run_p50_ms`, and `p50_stable_ms` is the median of those. `run_spread_pct`
  exposes run-to-run noise instead of averaging it away. Comparisons use
  `p50_stable_ms`.
* **Wall-clock, client side.** Every number includes harness overhead. This
  harness deliberately does not claim a server-internal split; the
  `protocol.floor_arg_error` workload bounds the harness cost, and in-process
  attribution belongs in `benches/micro.rs`.
* **Determinism fingerprint.** Every workload hashes its concatenated response
  text. `deterministic_across_runs` must be true, and the `response_sha256` list
  is what makes "byte-identical across worker counts" a mechanical check rather
  than an assertion.
* **Isolated state.** Every run gets a fresh `YEKATERINA_HOME` temporary
  directory. The user's real UDO registry is never touched.
* **No randomness.** All payloads are deterministic functions of their size.

## Frozen MCP surface invariant

`run_bench.py` asserts the v1.0.0 surface on every run and fails the gate on drift:

* exactly 3 tools: `yk.compute`, `yk.find`, `yk.spec`
* **1725** schema bytes
* **412** schema tokens (`tiktoken:o200k_base`)

The token/byte figures depend on the exact serialization. The published v1.0.0
numbers correspond to the whole `tools/list` **result object**, compact
separators, sorted keys:

```python
json.dumps({"tools": [...]}, separators=(",", ":"), sort_keys=True)
```

Serializing the bare tools array instead yields 1715 bytes / 410 tokens, and
leaving keys unsorted yields 411 tokens. `canonical_schema_blob()` encodes the
correct form; do not "simplify" it.

## Changing workloads

Adding a new workload id is safe. **Changing an existing workload's payload
invalidates every previously frozen baseline** and requires bumping
`WORKLOAD_SET_VERSION` in `workloads.py`.
