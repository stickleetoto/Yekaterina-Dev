# Yekaterina v1.1 — frozen v1.0.0 performance baseline

**This document is immutable.** It records the measurements the v1.1 work is
judged against. If it must ever be re-measured, capture a new baseline under a
new directory rather than editing this one.

## What is frozen

| Item | Value |
|---|---|
| Source fingerprint (`Cargo.toml` + `Cargo.lock` + `rust-toolchain.toml` + all `src/*.rs`) | `4c48025e92be87691e86e2ebb0f2cb9ff7defa66e9312537cdcc2c894c69453a` |
| Binary | `4,822,528` B, sha256 `753b64f8087de6285a5b1caf9e0c6e42a59d29cea0bf09ad7d08905c3e39dea4` |
| Workload set | `bench/workloads.py` `WORKLOAD_SET_VERSION = 1` (50 workloads) |
| Harness | `bench/run_bench.py` v1.0 |
| Result | `bench_results/v1.0.0-frozen/result.json`, `REPORT.md`, `micro.json` |

The binary was rebuilt from pristine v1.0.0 source with `cargo build --locked
--release` and is byte-identical to the shipped `target/release/yekaterina.exe`.
All v1.0.0 gates were green at capture time: static audit PASS, 91/91 Rust tests,
0 clippy errors, Golden 527/527, Full Capability Audit 1215/1215.

## Machine

| Field | Value |
|---|---|
| OS | Windows 11 (10.0.26200) |
| CPUs | 16 logical / 10 physical |
| RAM | 16 GiB |
| Toolchain | rustc 1.98.0, cargo 1.98.0 |
| Python | 3.13.4 |

Baselines are machine-specific. A baseline captured elsewhere is not comparable
to this one.

## Headline figures

| Metric | Value |
|---|---|
| Cold start (spawn through MCP `initialize`) | p50 **6.500 ms**, p95 8.162 ms, min 6.186 ms (n=20) |
| MCP surface | **3** tools, **412** tokens, **1725** bytes — frozen invariant, asserted on every run |
| Binary size | **4,822,528** B |
| RSS after full suite | 12.17 MiB (peak 21.30 MiB) |
| OS threads | **24** |
| CPU time for one suite run | 1.91 s user / 0.23 s system |

The 24 threads are worth noting for the parallel work: the tokio multi-thread
runtime already creates one worker per logical CPU, and `rmcp` already spawns
every `tools/call` as an independent task. Concurrency is not being introduced by
v1.1; it is being made explicit and bounded.

RSS is **not** comparable to the 5.95–6.00 MiB in `benchmark_results/alpha3-*` and
`mcp-realworld-alpha6/`: those sampled at cold start, this samples after the full
heavy suite has run.

## Full workload baseline

`p50` is pooled across 5 runs; `stable p50` is the median of the 5 per-run
medians; `spread` is the per-run median spread and is the honest noise band for
that workload.

| Workload | p50 | stable p50 | p95 | spread | us/op |
|---|---:|---:|---:|---:|---:|
| `protocol.floor_arg_error` | 0.082 | 0.083 | 0.143 | 46% | - |
| `protocol.tools_list` | 0.093 | 0.087 | 0.172 | 48% | - |
| `protocol.find` | 0.163 | 0.164 | 0.221 | 14% | - |
| `protocol.spec` | 0.064 | 0.064 | 0.106 | 12% | - |
| `single.math_add` | 0.064 | 0.065 | 0.097 | 24% | - |
| `single.stat_mean_100` | 0.084 | 0.083 | 0.150 | 55% | - |
| `single.mat_mul_100` | 5.653 | 5.663 | 7.593 | 23% | - |
| `single.mat_shape_100` | 2.518 | 2.418 | 3.664 | 29% | - |
| `single.signal_fft_2048` | 1.475 | 1.430 | 2.058 | 26% | - |
| `single.num_integrate_10k` | 1.855 | 1.855 | 2.816 | 12% | - |
| `single.ode_rk4_10k` | 6.763 | 6.758 | 7.280 | 3% | - |
| `single.int_pow_exact` | 0.123 | 0.108 | 0.276 | 90% | - |
| `single.dec_mul_exact` | 0.085 | 0.073 | 0.159 | 63% | - |
| `single.expr_eval` | 0.092 | 0.089 | 0.164 | 75% | - |
| `batch.scalar_10` | 0.101 | 0.097 | 0.178 | 39% | 10.12 |
| `batch.scalar_100` | 0.239 | 0.230 | 0.376 | 36% | 2.39 |
| `batch.scalar_1000` | 2.383 | 2.383 | 2.953 | 16% | 2.38 |
| `batch.scalar_10000_chunked` | 23.548 | 23.723 | 27.282 | 17% | 2.35 |
| `cheap.scalar_2` | 0.085 | 0.080 | 0.163 | 28% | 42.55 |
| `cheap.scalar_4` | 0.087 | 0.081 | 0.149 | 35% | 21.74 |
| `cheap.scalar_8` | 0.092 | 0.088 | 0.164 | 29% | 11.49 |
| `cheap.scalar_16` | 0.111 | 0.113 | 0.299 | 77% | 6.93 |
| `cheap.scalar_32` | 0.123 | 0.120 | 0.212 | 31% | 3.86 |
| `rescan.cumsum100_x10` | 0.399 | 0.380 | 0.673 | 53% | 39.88 |
| `rescan.cumsum100_x50` | 2.006 | 1.932 | 2.742 | 35% | 40.13 |
| `rescan.cumsum100_x100` | 4.464 | 4.464 | 5.248 | 13% | 44.64 |
| `rescan.cumsum100_x200` | 10.906 | 10.765 | 13.826 | 25% | 54.53 |
| `rescan.cumsum100_x400` | 29.144 | 29.279 | 33.378 | 13% | 72.86 |
| `heavy.integrate1000_x4` | 0.838 | 0.839 | 0.993 | 11% | 209.60 |
| `heavy.integrate1000_x16` | 2.925 | 2.933 | 3.211 | 7% | 182.79 |
| `heavy.integrate1000_x64` | 11.368 | 11.368 | 12.956 | 6% | 177.63 |
| `heavy.fft512_x8` | 2.650 | 2.578 | 3.093 | 16% | 331.26 |
| `heavy.fft512_x16` | 5.479 | 5.309 | 7.638 | 41% | 342.42 |
| `heavy.fft512_x32` | 11.872 | 11.763 | 14.128 | 22% | 370.99 |
| `heavy.mixed_skew_16` | 7.439 | 7.416 | 8.235 | 9% | 464.92 |
| `dependent.chain_64` | 0.176 | 0.171 | 0.259 | 34% | 2.74 |
| `dependent.half_64` | 0.184 | 0.192 | 0.328 | 31% | 2.88 |
| `pipeline.chain_10` | 0.100 | 0.102 | 0.148 | 41% | 10.04 |
| `pipeline.chain_100` | 0.204 | 0.203 | 0.338 | 25% | 2.04 |
| `pipeline.chain_256` | 0.457 | 0.456 | 0.605 | 6% | 1.78 |
| `pipeline.cumsum100_all_10` | 0.394 | 0.393 | 0.529 | 23% | 39.38 |
| `pipeline.cumsum100_all_100` | 4.111 | 4.248 | 4.808 | 15% | 41.11 |
| `pipeline.cumsum100_all_256` | 14.859 | 14.757 | 18.725 | 15% | 58.04 |
| `iopath.count_1000` | 0.271 | 0.266 | 0.458 | 71% | - |
| `iopath.cumsum_1000` | 0.358 | 0.341 | 0.771 | 91% | - |
| `iopath.count_10000` | 2.384 | 2.338 | 3.481 | 52% | - |
| `iopath.cumsum_10000` | 3.045 | 3.045 | 4.126 | 46% | - |
| `iopath.count_50000` | 15.206 | 17.213 | 19.024 | 27% | - |
| `iopath.cumsum_50000` | 18.698 | 20.136 | 23.686 | 31% | - |
| `stateful.udo_formula_define` | 2.259 | 2.256 | 3.562 | 30% | - |
## What the baseline confirms

Every Phase 1 audit finding reproduced under the frozen methodology.

**The accumulated-result rescan is superlinear.** `output_values_too_large` walks
the entire accumulated result vector after every item (`server.rs:124`, `:140`,
and `:87` for composites):

| Workload | stable p50 | us/op |
|---|---:|---:|
| `rescan.cumsum100_x10` | 0.380 ms | 39.88 |
| `rescan.cumsum100_x50` | 1.932 ms | 40.13 |
| `rescan.cumsum100_x100` | 4.464 ms | 44.64 |
| `rescan.cumsum100_x200` | 10.765 ms | 54.53 |
| `rescan.cumsum100_x400` | 29.279 ms | 72.86 |

40x the items costs 77x the time; per-op cost rises 83%. The real compute
(`stat.cumsum` over 100 floats) is roughly 2 us. This is Phase 2A.

**Heavy independent batches scale linearly and are the parallel opportunity.**

| Workload | stable p50 | us/op |
|---|---:|---:|
| `heavy.integrate1000_x4` | 0.839 ms | 209.60 |
| `heavy.integrate1000_x16` | 2.933 ms | 182.79 |
| `heavy.integrate1000_x64` | 11.368 ms | 177.63 |

Flat per-op cost across a 16x size range, small payloads, no `$N` references.

**Cheap batches are entirely protocol floor and must never be parallelised.**
`cheap.scalar_2` through `cheap.scalar_32` all land between 0.080 and 0.120 ms,
against a `protocol.floor_arg_error` of 0.083 ms. There is no compute to
distribute; any scheduling would be pure loss. This is the Phase 7 guard.

**Stateful writes cost ~27x a compute request.**
`stateful.udo_formula_define` = 2.256 ms, against 0.083 ms of protocol floor —
a full registry clone, snapshot serialization, `fsync` and rename, all under the
registry write lock. Phase 3.

## Measurement instruments, and what each may claim

Phase 0 established something that changes how every later phase must be judged:
**this machine drifts more than most optimizations will move the needle.**

`benches/micro.rs` reported `ode.rk4 n=10000` at 6.4 ms, 13.3 ms and 14.6 ms
across three runs of *identical code* in which the calibrator chose the identical
iteration count. The spread came entirely from how much CPU work preceding cases
had done. A start/end canary now runs with every micro report; on the baseline
capture it measured **1.73x drift within a single run**.

Separately, comparing a fresh `run_bench.py` run against the stored baseline
reported a consistent +14% to +26% on every sub-millisecond workload after a
change that a paired run showed to be -0.35%. That was pure session-to-session
drift.

| Instrument | Claims it can support | Claims it cannot support |
|---|---|---|
| `bench/paired_ab.py` | Accept/reject for any single change, including effects under 10%. Interleaves both binaries and flips order per round, so drift cancels. | Absolute cost of anything. |
| `bench/run_bench.py --compare` | Suite-level reporting; response-fingerprint and determinism gating; large effects. | Small deltas across sessions. Regression flags are noise-banded for this reason. |
| `benches/micro.rs` | Where time goes, at order-of-magnitude granularity. Which of two builds is faster when run back to back in the same order. | Absolute attribution. Cross-harness arithmetic against `run_bench.py`. Any change smaller than the run's canary drift. |

### Consequences for the Phase 2 merge rule

1. **`paired_ab.py` is the deciding instrument** for accepting an optimization.
   A `run_bench.py --compare` delta alone is not sufficient evidence.
2. A regression is significant only when it exceeds **both** the 2% threshold and
   the workload's own measured spread, and the workload sits above the 0.20 ms
   noise floor. `run_bench.py` implements this; rows that fail it are reported as
   `watch`, not as regressions.
3. **Response fingerprints are a hard gate and are not statistical.** Any
   `response_sha256` mismatch against the baseline fails the run outright,
   because it means externally observable behaviour changed.

## In-process attribution (order of magnitude only)

From `bench_results/v1.0.0-frozen/micro.json`. Read these as ratios within one
run, never as absolute costs — the canary on this capture was 1.73x.

| Area | Observation |
|---|---|
| Registry lookup | 14–15 ns canonical, ~13 ns alias, ~115 ns mixed-case/miss. Against ~2.3 us/op batch dispatch this is under 1%. **The audit's "low priority" call is confirmed; do not optimize it.** |
| `parse_step` | ~150–200 ns per item, both forms. The array form clones its argument `Value`s via `tail.to_vec()`. |
| `engine::execute("math.add")` | ~70–80 ns. Execution is a few percent of a batch item's cost. |
| Expression evaluator | `formula::eval("x*x")` ~230–290 ns, and the per-evaluation `HashMap` clone plus `"x".to_string()` that `num.`/`ode.`/`optimize.`/`series.` perform costs ~215–220 ns on its own — comparable to the entire evaluation it precedes. This is Phase 2C. |
| Input path | `serde_json` parse ~20–59 ns/element; `Value` to `Vec<f64>` ~1.3–1.6 ns/element. Parsing dominates conversion by more than an order of magnitude. |
| Output path | `Value::to_string` ~23–54 ns/element. |
| 1000-item batch parse | ~249 us, ~249 ns/item. |

One audit correction: the Phase 1 report measured ~287 ns per element end-to-end
on the input path and stated the split between harness transport and server-side
deserialization was unmeasured. It is now measured, and **server-side parsing is
the smaller share** (tens of ns/element). The remainder is harness and pipe. The
audit's ranking of the input path as the largest single cost centre stands for
end-to-end latency, but the portion Yekaterina can actually optimize is smaller
than that number implied.

## Reproducing

```powershell
cargo build --locked --release
python .\bench\run_bench.py --exe .\target\release\yekaterina.exe --out .\bench_results\latest --compare .\bench_results\v1.0.0-frozen\result.json
cargo bench --locked --bench micro -- --json .\bench_results\latest\micro.json
```

For an accept/reject decision on a specific change:

```powershell
python .\bench\paired_ab.py --a .\baseline.exe --b .\target\release\yekaterina.exe --only batch,rescan,heavy
```
