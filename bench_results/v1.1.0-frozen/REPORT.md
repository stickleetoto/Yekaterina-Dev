# Yekaterina Benchmark - v1.1.0-frozen

Scope: **MCP-only; no LLM, no network.**

## Environment

| Field | Value |
|---|---|
| Harness / workload set | v1.0 / v1 |
| Platform | Windows-11-10.0.26200-SP0 |
| Logical / physical CPUs | 16 / 10 |
| rustc | rustc 1.98.0 (88d9e12ae 2026-08-18) |
| Workers arg | `1` |
| Binary | 4,399,616 B, sha256 `e0b23ba997e4df72...` |
| Source fingerprint | `66349dc95f4ded75...` (60 files) |
| Runs x reps | 5 |

## MCP surface (frozen invariant)

- Tools: **3** -> ['yk.compute', 'yk.find', 'yk.spec']
- Schema bytes: **1725**
- Schema tokens: **412** (tiktoken:o200k_base)
- Frozen surface (3 tools / 412 tokens / 1725 bytes): **PASS**
- Serialization: `json.dumps({'tools':...}, separators=(',',':'), sort_keys=True)`

## Cold start

- p50 **11.739 ms**, p95 13.105 ms, min 11.323 ms (n=15, process spawn through MCP initialize)

## Workloads

### batch

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `batch.scalar_10` | 0.092 ms | 0.186 ms | 0.092 ms | 92.3% | 108,225 | 9.24 | ok |
| `batch.scalar_100` | 0.203 ms | 0.286 ms | 0.186 ms | 47.5% | 492,368 | 2.03 | ok |
| `batch.scalar_1000` | 1.316 ms | 1.699 ms | 1.314 ms | 28.5% | 759,705 | 1.32 | ok |
| `batch.scalar_10000_chunked` | 12.998 ms | 17.396 ms | 12.828 ms | 27.5% | 769,349 | 1.30 | ok |

### bigarg

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `bigarg.ref_sum1000_x16` | 0.376 ms | 0.644 ms | 0.387 ms | 24.6% | 42,599 | 23.47 | ok |
| `bigarg.sum10000_x4` | 9.324 ms | 12.714 ms | 9.313 ms | 42.6% | 429 | 2331.05 | ok |
| `bigarg.sum1000_x16` | 3.687 ms | 5.025 ms | 3.635 ms | 30.0% | 4,339 | 230.45 | ok |

### cheap

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `cheap.scalar_16` | 0.103 ms | 0.173 ms | 0.090 ms | 46.7% | 155,794 | 6.42 | ok |
| `cheap.scalar_2` | 0.072 ms | 0.136 ms | 0.068 ms | 52.0% | 27,855 | 35.90 | ok |
| `cheap.scalar_32` | 0.121 ms | 0.184 ms | 0.106 ms | 36.1% | 264,135 | 3.79 | ok |
| `cheap.scalar_4` | 0.077 ms | 0.129 ms | 0.073 ms | 36.1% | 51,680 | 19.35 | ok |
| `cheap.scalar_8` | 0.090 ms | 0.155 ms | 0.078 ms | 34.7% | 89,037 | 11.23 | ok |

### dependent

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `dependent.chain_64` | 0.154 ms | 0.228 ms | 0.150 ms | 8.8% | 415,584 | 2.41 | ok |
| `dependent.half_64` | 0.135 ms | 0.185 ms | 0.134 ms | 21.8% | 475,130 | 2.10 | ok |

### heavy

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `heavy.fft512_x16` | 4.266 ms | 5.383 ms | 4.222 ms | 28.0% | 3,750 | 266.64 | ok |
| `heavy.fft512_x32` | 8.853 ms | 10.243 ms | 8.817 ms | 12.4% | 3,615 | 276.66 | ok |
| `heavy.fft512_x8` | 2.063 ms | 2.839 ms | 2.001 ms | 31.8% | 3,877 | 257.94 | ok |
| `heavy.integrate1000_x16` | 0.882 ms | 2.254 ms | 0.877 ms | 138.1% | 18,149 | 55.10 | ok |
| `heavy.integrate1000_x4` | 0.305 ms | 0.450 ms | 0.297 ms | 17.1% | 13,102 | 76.33 | ok |
| `heavy.integrate1000_x64` | 3.076 ms | 3.297 ms | 3.094 ms | 4.7% | 20,808 | 48.06 | ok |
| `heavy.mixed_skew_16` | 7.221 ms | 9.354 ms | 7.221 ms | 9.3% | 2,216 | 451.34 | ok |

### iopath

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `iopath.count_1000` | 0.278 ms | 0.382 ms | 0.279 ms | 49.5% | 3,600 | - | ok |
| `iopath.count_10000` | 2.159 ms | 3.487 ms | 1.984 ms | 53.3% | 463 | - | ok |
| `iopath.count_50000` | 14.870 ms | 17.188 ms | 12.972 ms | 35.2% | 67 | - | ok |
| `iopath.cumsum_1000` | 0.324 ms | 0.502 ms | 0.319 ms | 47.8% | 3,084 | - | ok |
| `iopath.cumsum_10000` | 2.722 ms | 4.197 ms | 2.599 ms | 44.9% | 367 | - | ok |
| `iopath.cumsum_50000` | 15.412 ms | 20.481 ms | 15.767 ms | 11.4% | 65 | - | ok |

### optimize

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `heavy.optimize_gd2d_x32` | 4.956 ms | 5.298 ms | 4.993 ms | 4.3% | 6,457 | 154.88 | ok |
| `heavy.optimize_nelder_x32` | 0.481 ms | 0.616 ms | 0.476 ms | 11.3% | 66,500 | 15.04 | ok |
| `single.optimize_argmin_100k` | 8.985 ms | 10.465 ms | 8.924 ms | 9.7% | 111 | - | ok |

### pipeline

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `pipeline.chain_10` | 0.083 ms | 0.120 ms | 0.081 ms | 33.3% | 121,212 | 8.25 | ok |
| `pipeline.chain_100` | 0.192 ms | 0.328 ms | 0.190 ms | 43.2% | 522,193 | 1.92 | ok |
| `pipeline.chain_256` | 0.343 ms | 0.429 ms | 0.344 ms | 24.0% | 745,921 | 1.34 | ok |
| `pipeline.cumsum100_all_10` | 0.334 ms | 0.533 ms | 0.334 ms | 35.5% | 29,967 | 33.37 | ok |
| `pipeline.cumsum100_all_100` | 2.632 ms | 3.569 ms | 2.592 ms | 30.6% | 37,990 | 26.32 | ok |
| `pipeline.cumsum100_all_256` | 7.078 ms | 9.052 ms | 6.871 ms | 33.0% | 36,168 | 27.65 | ok |

### protocol

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `protocol.find` | 0.166 ms | 0.204 ms | 0.164 ms | 7.9% | 6,039 | - | ok |
| `protocol.floor_arg_error` | 0.067 ms | 0.104 ms | 0.066 ms | 30.0% | 14,859 | - | ok |
| `protocol.spec` | 0.068 ms | 0.101 ms | 0.067 ms | 8.4% | 14,804 | - | ok |
| `protocol.tools_list` | 0.090 ms | 0.132 ms | 0.084 ms | 25.6% | 11,050 | - | ok |

### rescan

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `rescan.cumsum100_x10` | 0.400 ms | 0.474 ms | 0.359 ms | 31.7% | 25,013 | 39.98 | ok |
| `rescan.cumsum100_x100` | 3.233 ms | 4.147 ms | 3.082 ms | 30.0% | 30,929 | 32.33 | ok |
| `rescan.cumsum100_x200` | 5.917 ms | 8.752 ms | 5.864 ms | 35.8% | 33,801 | 29.58 | ok |
| `rescan.cumsum100_x400` | 12.520 ms | 16.281 ms | 14.083 ms | 25.1% | 31,949 | 31.30 | ok |
| `rescan.cumsum100_x50` | 1.601 ms | 2.129 ms | 1.580 ms | 32.8% | 31,230 | 32.02 | ok |

### single

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `single.dec_mul_exact` | 0.066 ms | 0.116 ms | 0.070 ms | 37.8% | 15,083 | - | ok |
| `single.expr_eval` | 0.073 ms | 0.119 ms | 0.066 ms | 41.0% | 13,755 | - | ok |
| `single.int_pow_exact` | 0.102 ms | 0.140 ms | 0.100 ms | 19.6% | 9,794 | - | ok |
| `single.mat_mul_100` | 5.497 ms | 6.815 ms | 5.518 ms | 2.7% | 182 | - | ok |
| `single.mat_shape_100` | 2.216 ms | 3.024 ms | 2.200 ms | 16.2% | 451 | - | ok |
| `single.math_add` | 0.067 ms | 0.096 ms | 0.067 ms | 16.5% | 15,015 | - | ok |
| `single.num_integrate_10k` | 0.559 ms | 0.638 ms | 0.560 ms | 3.0% | 1,788 | - | ok |
| `single.ode_rk4_10k` | 1.814 ms | 2.053 ms | 1.806 ms | 9.9% | 551 | - | ok |
| `single.signal_fft_2048` | 1.271 ms | 1.438 ms | 1.271 ms | 3.7% | 787 | - | ok |
| `single.stat_mean_100` | 0.088 ms | 0.114 ms | 0.088 ms | 13.0% | 11,351 | - | ok |

### stateful

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `stateful.udo_formula_define` | 1.820 ms | 2.955 ms | 1.781 ms | 12.2% | 550 | - | ok |

## Process metrics (end of each run)

| Run | RSS | Peak RSS | CPU user | CPU sys | Threads |
|---:|---:|---:|---:|---:|---:|
| 1 | 12.40 MiB | 21.36 MiB | 1.297 s | 0.344 s | 24 |
| 2 | 12.80 MiB | 21.74 MiB | 1.109 s | 0.281 s | 24 |
| 3 | 13.09 MiB | 21.15 MiB | 1.156 s | 0.438 s | 24 |
| 4 | 13.78 MiB | 21.82 MiB | 1.188 s | 0.375 s | 24 |
| 5 | 12.21 MiB | 20.25 MiB | 1.141 s | 0.422 s | 24 |

## Integrity

- Non-deterministic responses across runs: **0** (none)
- Unexpected error envelopes: **0** (none)

## Comparison against baseline

Baseline: `bench_results\v1.0.0-frozen\result.json` (v1.0.0 FROZEN BASELINE (pristine source, sequential))

Regression rule: delta > 2.0% **and** greater than the workload's own run-to-run spread, for workloads whose baseline p50 is at least 0.2 ms. Faster-but-noisy and below-floor rows are reported as `watch`, not as regressions.

| Workload | baseline p50 | current p50 | delta | noise band | speedup | fp |
|---|---:|---:|---:|---:|---:|:--:|
| `single.stat_mean_100` | 0.083 ms | 0.088 ms | +5.9% (watch) _below floor_ | +/-55% | 0.94x | ok |
| `iopath.count_1000` | 0.266 ms | 0.279 ms | +4.9% (watch) | +/-71% | 0.95x | ok |
| `single.math_add` | 0.065 ms | 0.067 ms | +4.3% (watch) _below floor_ | +/-24% | 0.96x | ok |
| `protocol.spec` | 0.064 ms | 0.067 ms | +4.1% (watch) _below floor_ | +/-12% | 0.96x | ok |
| `protocol.find` | 0.164 ms | 0.164 ms | +0.1%  _below floor_ | +/-14% | 1.00x | ok |
| `single.mat_mul_100` | 5.663 ms | 5.518 ms | -2.6%  | +/-23% | 1.03x | ok |
| `heavy.mixed_skew_16` | 7.416 ms | 7.221 ms | -2.6%  | +/-9% | 1.03x | ok |
| `protocol.tools_list` | 0.087 ms | 0.084 ms | -3.2%  _below floor_ | +/-48% | 1.03x | ok |
| `single.dec_mul_exact` | 0.073 ms | 0.070 ms | -4.4%  _below floor_ | +/-63% | 1.05x | ok |
| `batch.scalar_10` | 0.097 ms | 0.092 ms | -4.9%  _below floor_ | +/-92% | 1.05x | ok |
| `rescan.cumsum100_x10` | 0.380 ms | 0.359 ms | -5.5%  | +/-53% | 1.06x | ok |
| `iopath.cumsum_1000` | 0.341 ms | 0.319 ms | -6.4%  | +/-91% | 1.07x | ok |
| `pipeline.chain_100` | 0.203 ms | 0.190 ms | -6.4%  | +/-43% | 1.07x | ok |
| `single.int_pow_exact` | 0.108 ms | 0.100 ms | -7.7%  _below floor_ | +/-90% | 1.08x | ok |
| `single.mat_shape_100` | 2.418 ms | 2.200 ms | -9.0%  | +/-29% | 1.10x | ok |
| `cheap.scalar_4` | 0.081 ms | 0.073 ms | -9.9%  _below floor_ | +/-36% | 1.11x | ok |
| `cheap.scalar_8` | 0.088 ms | 0.078 ms | -10.7%  _below floor_ | +/-35% | 1.12x | ok |
| `single.signal_fft_2048` | 1.430 ms | 1.271 ms | -11.1%  | +/-26% | 1.12x | ok |
| `cheap.scalar_32` | 0.120 ms | 0.106 ms | -12.3%  _below floor_ | +/-36% | 1.14x | ok |
| `dependent.chain_64` | 0.171 ms | 0.150 ms | -12.3%  _below floor_ | +/-34% | 1.14x | ok |
| `cheap.scalar_2` | 0.080 ms | 0.068 ms | -14.3%  _below floor_ | +/-52% | 1.17x | ok |
| `iopath.cumsum_10000` | 3.045 ms | 2.599 ms | -14.6%  | +/-46% | 1.17x | ok |
| `iopath.count_10000` | 2.338 ms | 1.984 ms | -15.1%  | +/-53% | 1.18x | ok |
| `pipeline.cumsum100_all_10` | 0.393 ms | 0.334 ms | -15.2%  | +/-35% | 1.18x | ok |
| `rescan.cumsum100_x50` | 1.932 ms | 1.580 ms | -18.2%  | +/-35% | 1.22x | ok |
| `batch.scalar_100` | 0.230 ms | 0.186 ms | -19.1%  | +/-47% | 1.24x | ok |
| `pipeline.chain_10` | 0.102 ms | 0.081 ms | -20.1%  _below floor_ | +/-41% | 1.25x | ok |
| `protocol.floor_arg_error` | 0.083 ms | 0.066 ms | -20.3%  _below floor_ | +/-46% | 1.25x | ok |
| `heavy.fft512_x16` | 5.309 ms | 4.222 ms | -20.5%  | +/-41% | 1.26x | ok |
| `cheap.scalar_16` | 0.113 ms | 0.090 ms | -20.9%  _below floor_ | +/-77% | 1.26x | ok |
| `stateful.udo_formula_define` | 2.256 ms | 1.781 ms | -21.1%  | +/-30% | 1.27x | ok |
| `iopath.cumsum_50000` | 20.136 ms | 15.767 ms | -21.7%  | +/-31% | 1.28x | ok |
| `heavy.fft512_x8` | 2.578 ms | 2.001 ms | -22.4%  | +/-32% | 1.29x | ok |
| `pipeline.chain_256` | 0.456 ms | 0.344 ms | -24.6%  | +/-24% | 1.33x | ok |
| `iopath.count_50000` | 17.213 ms | 12.972 ms | -24.6%  | +/-35% | 1.33x | ok |
| `heavy.fft512_x32` | 11.763 ms | 8.817 ms | -25.0%  | +/-22% | 1.33x | ok |
| `single.expr_eval` | 0.089 ms | 0.066 ms | -25.6%  _below floor_ | +/-75% | 1.34x | ok |
| `dependent.half_64` | 0.192 ms | 0.134 ms | -30.2%  _below floor_ | +/-31% | 1.43x | ok |
| `rescan.cumsum100_x100` | 4.464 ms | 3.082 ms | -31.0%  | +/-30% | 1.45x | ok |
| `pipeline.cumsum100_all_100` | 4.248 ms | 2.592 ms | -39.0%  | +/-31% | 1.64x | ok |
| `batch.scalar_1000` | 2.383 ms | 1.314 ms | -44.9%  | +/-29% | 1.81x | ok |
| `rescan.cumsum100_x200` | 10.765 ms | 5.864 ms | -45.5%  | +/-36% | 1.84x | ok |
| `batch.scalar_10000_chunked` | 23.723 ms | 12.828 ms | -45.9%  | +/-27% | 1.85x | ok |
| `rescan.cumsum100_x400` | 29.279 ms | 14.083 ms | -51.9%  | +/-25% | 2.08x | ok |
| `pipeline.cumsum100_all_256` | 14.757 ms | 6.871 ms | -53.4%  | +/-33% | 2.15x | ok |
| `heavy.integrate1000_x4` | 0.839 ms | 0.297 ms | -64.6%  | +/-17% | 2.83x | ok |
| `single.num_integrate_10k` | 1.855 ms | 0.560 ms | -69.8%  | +/-12% | 3.31x | ok |
| `heavy.integrate1000_x16` | 2.933 ms | 0.877 ms | -70.1%  | +/-138% | 3.34x | ok |
| `heavy.integrate1000_x64` | 11.368 ms | 3.094 ms | -72.8%  | +/-6% | 3.67x | ok |
| `single.ode_rk4_10k` | 6.758 ms | 1.806 ms | -73.3%  | +/-10% | 3.74x | ok |

- Significant regressions: **0**
- Watch (over threshold but within noise): **4** -> ['single.stat_mean_100', 'iopath.count_1000', 'single.math_add', 'protocol.spec']
- Fingerprint mismatches: **0**

