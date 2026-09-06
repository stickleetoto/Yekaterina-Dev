# Yekaterina Benchmark - v1.2.0-frozen

Scope: **MCP-only; no LLM, no network.**

## Environment

| Field | Value |
|---|---|
| Harness / workload set | v1.0 / v1 |
| Platform | Windows-11-10.0.26200-SP0 |
| Logical / physical CPUs | 16 / 10 |
| rustc | rustc 1.98.0 (88d9e12ae 2026-08-18) |
| Workers arg | `1` |
| Binary | 4,538,368 B, sha256 `c8b2e0ffe52fffb0...` |
| Source fingerprint | `1b07a9466da38412...` (60 files) |
| Runs x reps | 5 |

## MCP surface (frozen invariant)

- Tools: **3** -> ['yk.compute', 'yk.find', 'yk.spec']
- Schema bytes: **1725**
- Schema tokens: **412** (tiktoken:o200k_base)
- Frozen surface (3 tools / 412 tokens / 1725 bytes): **PASS**
- Serialization: `json.dumps({'tools':...}, separators=(',',':'), sort_keys=True)`

## Cold start

- p50 **5.810 ms**, p95 6.069 ms, min 5.542 ms (n=15, process spawn through MCP initialize)

## Workloads

### batch

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `batch.scalar_10` | 0.111 ms | 0.159 ms | 0.111 ms | 38.1% | 90,253 | 11.08 | ok |
| `batch.scalar_100` | 0.224 ms | 0.404 ms | 0.227 ms | 29.7% | 446,030 | 2.24 | ok |
| `batch.scalar_1000` | 1.467 ms | 1.552 ms | 1.486 ms | 15.6% | 681,849 | 1.47 | ok |
| `batch.scalar_10000_chunked` | 14.700 ms | 15.900 ms | 14.808 ms | 22.3% | 680,291 | 1.47 | ok |

### bigarg

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `bigarg.ref_sum1000_x16` | 0.425 ms | 2.168 ms | 0.430 ms | 28.4% | 37,629 | 26.58 | ok |
| `bigarg.sum10000_x4` | 11.522 ms | 12.258 ms | 11.689 ms | 26.1% | 347 | 2880.53 | ok |
| `bigarg.sum1000_x16` | 4.555 ms | 6.705 ms | 4.558 ms | 27.3% | 3,513 | 284.67 | ok |

### cheap

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `cheap.scalar_16` | 0.115 ms | 0.144 ms | 0.116 ms | 33.3% | 139,191 | 7.18 | ok |
| `cheap.scalar_2` | 0.088 ms | 0.118 ms | 0.085 ms | 38.5% | 22,663 | 44.12 | ok |
| `cheap.scalar_32` | 0.131 ms | 0.177 ms | 0.134 ms | 33.0% | 244,088 | 4.10 | ok |
| `cheap.scalar_4` | 0.086 ms | 0.109 ms | 0.086 ms | 42.9% | 46,323 | 21.59 | ok |
| `cheap.scalar_8` | 0.105 ms | 0.129 ms | 0.106 ms | 36.0% | 76,409 | 13.09 | ok |

### dependent

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `dependent.chain_64` | 0.181 ms | 0.359 ms | 0.179 ms | 39.6% | 354,178 | 2.82 | ok |
| `dependent.half_64` | 0.178 ms | 0.226 ms | 0.178 ms | 26.0% | 359,349 | 2.78 | ok |

### heavy

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `heavy.fft512_x16` | 4.844 ms | 5.210 ms | 4.875 ms | 17.7% | 3,303 | 302.73 | ok |
| `heavy.fft512_x32` | 10.281 ms | 12.737 ms | 10.375 ms | 25.8% | 3,113 | 321.27 | ok |
| `heavy.fft512_x8` | 2.300 ms | 2.510 ms | 2.351 ms | 19.1% | 3,478 | 287.49 | ok |
| `heavy.integrate1000_x16` | 0.853 ms | 1.392 ms | 0.852 ms | 8.6% | 18,760 | 53.31 | ok |
| `heavy.integrate1000_x4` | 0.286 ms | 0.340 ms | 0.291 ms | 15.0% | 13,976 | 71.55 | ok |
| `heavy.integrate1000_x64` | 2.981 ms | 3.133 ms | 2.982 ms | 3.0% | 21,470 | 46.58 | ok |
| `heavy.mixed_skew_16` | 6.809 ms | 9.157 ms | 6.825 ms | 6.4% | 2,350 | 425.56 | ok |

### iopath

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `iopath.count_1000` | 0.328 ms | 0.422 ms | 0.329 ms | 42.2% | 3,045 | - | ok |
| `iopath.count_10000` | 2.598 ms | 3.033 ms | 2.619 ms | 32.5% | 385 | - | ok |
| `iopath.count_50000` | 14.655 ms | 15.415 ms | 14.728 ms | 23.9% | 68 | - | ok |
| `iopath.cumsum_1000` | 0.393 ms | 1.973 ms | 0.395 ms | 28.7% | 2,546 | - | ok |
| `iopath.cumsum_10000` | 3.407 ms | 3.978 ms | 3.454 ms | 35.4% | 294 | - | ok |
| `iopath.cumsum_50000` | 18.915 ms | 20.036 ms | 18.948 ms | 28.3% | 53 | - | ok |

### optimize

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `heavy.optimize_gd2d_x32` | 4.806 ms | 6.088 ms | 4.790 ms | 2.3% | 6,658 | 150.19 | ok |
| `heavy.optimize_nelder_x32` | 0.482 ms | 0.554 ms | 0.489 ms | 10.5% | 66,349 | 15.07 | ok |
| `single.optimize_argmin_100k` | 8.936 ms | 9.416 ms | 8.936 ms | 4.1% | 112 | - | ok |

### pipeline

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `pipeline.chain_10` | 0.109 ms | 0.211 ms | 0.109 ms | 32.7% | 91,659 | 10.91 | ok |
| `pipeline.chain_100` | 0.213 ms | 0.332 ms | 0.214 ms | 35.2% | 469,484 | 2.13 | ok |
| `pipeline.chain_256` | 0.379 ms | 0.788 ms | 0.380 ms | 20.2% | 675,818 | 1.48 | ok |
| `pipeline.cumsum100_all_10` | 0.397 ms | 1.774 ms | 0.398 ms | 27.2% | 25,221 | 39.65 | ok |
| `pipeline.cumsum100_all_100` | 3.226 ms | 3.578 ms | 3.238 ms | 33.8% | 30,994 | 32.26 | ok |
| `pipeline.cumsum100_all_256` | 8.314 ms | 9.770 ms | 8.384 ms | 34.5% | 30,790 | 32.48 | ok |

### protocol

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `protocol.find` | 0.181 ms | 0.212 ms | 0.182 ms | 22.3% | 5,513 | - | ok |
| `protocol.floor_arg_error` | 0.093 ms | 0.112 ms | 0.094 ms | 37.3% | 10,753 | - | ok |
| `protocol.spec` | 0.096 ms | 0.107 ms | 0.096 ms | 38.1% | 10,438 | - | ok |
| `protocol.tools_list` | 0.128 ms | 0.154 ms | 0.130 ms | 41.5% | 7,794 | - | ok |

### rescan

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `rescan.cumsum100_x10` | 0.393 ms | 0.479 ms | 0.399 ms | 31.2% | 25,439 | 39.31 | ok |
| `rescan.cumsum100_x100` | 3.498 ms | 3.932 ms | 3.574 ms | 26.0% | 28,584 | 34.98 | ok |
| `rescan.cumsum100_x200` | 6.833 ms | 8.305 ms | 6.951 ms | 24.8% | 29,268 | 34.17 | ok |
| `rescan.cumsum100_x400` | 13.822 ms | 14.369 ms | 13.496 ms | 22.3% | 28,940 | 34.55 | ok |
| `rescan.cumsum100_x50` | 1.727 ms | 1.881 ms | 1.759 ms | 26.5% | 28,957 | 34.53 | ok |

### single

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `single.dec_mul_exact` | 0.098 ms | 0.152 ms | 0.098 ms | 39.4% | 10,225 | - | ok |
| `single.expr_eval` | 0.099 ms | 0.111 ms | 0.099 ms | 40.0% | 10,142 | - | ok |
| `single.int_pow_exact` | 0.124 ms | 0.172 ms | 0.124 ms | 32.7% | 8,071 | - | ok |
| `single.mat_mul_100` | 6.200 ms | 7.697 ms | 6.290 ms | 19.0% | 161 | - | ok |
| `single.mat_shape_100` | 2.628 ms | 2.928 ms | 2.659 ms | 23.1% | 381 | - | ok |
| `single.math_add` | 0.096 ms | 0.116 ms | 0.096 ms | 35.9% | 10,460 | - | ok |
| `single.num_integrate_10k` | 0.553 ms | 0.727 ms | 0.567 ms | 29.9% | 1,808 | - | ok |
| `single.ode_rk4_10k` | 1.708 ms | 1.797 ms | 1.728 ms | 4.1% | 586 | - | ok |
| `single.signal_fft_2048` | 1.431 ms | 3.450 ms | 1.472 ms | 20.0% | 699 | - | ok |
| `single.stat_mean_100` | 0.121 ms | 0.152 ms | 0.121 ms | 40.2% | 8,251 | - | ok |

### stateful

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `stateful.udo_formula_define` | 1.684 ms | 2.478 ms | 1.684 ms | 20.9% | 594 | - | ok |

## Process metrics (end of each run)

| Run | RSS | Peak RSS | CPU user | CPU sys | Threads |
|---:|---:|---:|---:|---:|---:|
| 1 | 12.48 MiB | 20.61 MiB | 1.203 s | 0.359 s | 25 |
| 2 | 10.84 MiB | 21.76 MiB | 1.234 s | 0.234 s | 24 |
| 3 | 10.59 MiB | 21.55 MiB | 1.156 s | 0.328 s | 24 |
| 4 | 10.36 MiB | 21.34 MiB | 1.141 s | 0.328 s | 24 |
| 5 | 11.43 MiB | 20.46 MiB | 1.250 s | 0.312 s | 23 |

## Integrity

- Non-deterministic responses across runs: **0** (none)
- Unexpected error envelopes: **0** (none)

## Comparison against baseline

Baseline: `bench_results\v1.0.0-frozen\result.json` (v1.0.0 FROZEN BASELINE (pristine source, sequential))

Regression rule: delta > 2.0% **and** greater than the workload's own run-to-run spread, for workloads whose baseline p50 is at least 0.2 ms. Faster-but-noisy and below-floor rows are reported as `watch`, not as regressions.

| Workload | baseline p50 | current p50 | delta | noise band | speedup | fp |
|---|---:|---:|---:|---:|---:|:--:|
| `protocol.spec` | 0.064 ms | 0.096 ms | +49.9% (watch) _below floor_ | +/-38% | 0.67x | ok |
| `single.math_add` | 0.065 ms | 0.096 ms | +49.2% (watch) _below floor_ | +/-36% | 0.67x | ok |
| `protocol.tools_list` | 0.087 ms | 0.130 ms | +49.1% (watch) _below floor_ | +/-48% | 0.67x | ok |
| `single.stat_mean_100` | 0.083 ms | 0.121 ms | +45.9% (watch) _below floor_ | +/-55% | 0.69x | ok |
| `single.dec_mul_exact` | 0.073 ms | 0.098 ms | +33.4% (watch) _below floor_ | +/-63% | 0.75x | ok |
| `iopath.count_1000` | 0.266 ms | 0.329 ms | +23.7% (watch) | +/-71% | 0.81x | ok |
| `cheap.scalar_8` | 0.088 ms | 0.106 ms | +21.0% (watch) _below floor_ | +/-36% | 0.83x | ok |
| `iopath.cumsum_1000` | 0.341 ms | 0.395 ms | +15.8% (watch) | +/-91% | 0.86x | ok |
| `single.int_pow_exact` | 0.108 ms | 0.124 ms | +14.9% (watch) _below floor_ | +/-90% | 0.87x | ok |
| `batch.scalar_10` | 0.097 ms | 0.111 ms | +14.5% (watch) _below floor_ | +/-39% | 0.87x | ok |
| `iopath.cumsum_10000` | 3.045 ms | 3.454 ms | +13.4% (watch) | +/-46% | 0.88x | ok |
| `protocol.floor_arg_error` | 0.083 ms | 0.094 ms | +13.1% (watch) _below floor_ | +/-46% | 0.88x | ok |
| `iopath.count_10000` | 2.338 ms | 2.619 ms | +12.0% (watch) | +/-52% | 0.89x | ok |
| `single.expr_eval` | 0.089 ms | 0.099 ms | +11.4% (watch) _below floor_ | +/-75% | 0.90x | ok |
| `cheap.scalar_32` | 0.120 ms | 0.134 ms | +11.2% (watch) _below floor_ | +/-33% | 0.90x | ok |
| `protocol.find` | 0.164 ms | 0.182 ms | +11.2% (watch) _below floor_ | +/-22% | 0.90x | ok |
| `single.mat_mul_100` | 5.663 ms | 6.290 ms | +11.1% (watch) | +/-23% | 0.90x | ok |
| `single.mat_shape_100` | 2.418 ms | 2.659 ms | +10.0% (watch) | +/-29% | 0.91x | ok |
| `pipeline.chain_10` | 0.102 ms | 0.109 ms | +7.3% (watch) _below floor_ | +/-41% | 0.93x | ok |
| `cheap.scalar_2` | 0.080 ms | 0.085 ms | +7.2% (watch) _below floor_ | +/-38% | 0.93x | ok |
| `cheap.scalar_4` | 0.081 ms | 0.086 ms | +5.7% (watch) _below floor_ | +/-43% | 0.95x | ok |
| `pipeline.chain_100` | 0.203 ms | 0.214 ms | +5.5% (watch) | +/-35% | 0.95x | ok |
| `rescan.cumsum100_x10` | 0.380 ms | 0.399 ms | +5.0% (watch) | +/-53% | 0.95x | ok |
| `dependent.chain_64` | 0.171 ms | 0.179 ms | +4.9% (watch) _below floor_ | +/-40% | 0.95x | ok |
| `single.signal_fft_2048` | 1.430 ms | 1.472 ms | +2.9% (watch) | +/-26% | 0.97x | ok |
| `cheap.scalar_16` | 0.113 ms | 0.116 ms | +2.0% (watch) _below floor_ | +/-77% | 0.98x | ok |
| `pipeline.cumsum100_all_10` | 0.393 ms | 0.398 ms | +1.1%  | +/-27% | 0.99x | ok |
| `batch.scalar_100` | 0.230 ms | 0.227 ms | -1.2%  | +/-36% | 1.01x | ok |
| `iopath.cumsum_50000` | 20.136 ms | 18.948 ms | -5.9%  | +/-31% | 1.06x | ok |
| `dependent.half_64` | 0.192 ms | 0.178 ms | -7.4%  _below floor_ | +/-31% | 1.08x | ok |
| `heavy.mixed_skew_16` | 7.416 ms | 6.825 ms | -8.0%  | +/-9% | 1.09x | ok |
| `heavy.fft512_x16` | 5.309 ms | 4.875 ms | -8.2%  | +/-41% | 1.09x | ok |
| `heavy.fft512_x8` | 2.578 ms | 2.351 ms | -8.8%  | +/-19% | 1.10x | ok |
| `rescan.cumsum100_x50` | 1.932 ms | 1.759 ms | -9.0%  | +/-35% | 1.10x | ok |
| `heavy.fft512_x32` | 11.763 ms | 10.375 ms | -11.8%  | +/-26% | 1.13x | ok |
| `iopath.count_50000` | 17.213 ms | 14.728 ms | -14.4%  | +/-27% | 1.17x | ok |
| `pipeline.chain_256` | 0.456 ms | 0.380 ms | -16.7%  | +/-20% | 1.20x | ok |
| `rescan.cumsum100_x100` | 4.464 ms | 3.574 ms | -19.9%  | +/-26% | 1.25x | ok |
| `pipeline.cumsum100_all_100` | 4.248 ms | 3.238 ms | -23.8%  | +/-34% | 1.31x | ok |
| `stateful.udo_formula_define` | 2.256 ms | 1.684 ms | -25.3%  | +/-30% | 1.34x | ok |
| `rescan.cumsum100_x200` | 10.765 ms | 6.951 ms | -35.4%  | +/-25% | 1.55x | ok |
| `batch.scalar_10000_chunked` | 23.723 ms | 14.808 ms | -37.6%  | +/-22% | 1.60x | ok |
| `batch.scalar_1000` | 2.383 ms | 1.486 ms | -37.6%  | +/-16% | 1.60x | ok |
| `pipeline.cumsum100_all_256` | 14.757 ms | 8.384 ms | -43.2%  | +/-34% | 1.76x | ok |
| `rescan.cumsum100_x400` | 29.279 ms | 13.496 ms | -53.9%  | +/-22% | 2.17x | ok |
| `heavy.integrate1000_x4` | 0.839 ms | 0.291 ms | -65.3%  | +/-15% | 2.88x | ok |
| `single.num_integrate_10k` | 1.855 ms | 0.567 ms | -69.4%  | +/-30% | 3.27x | ok |
| `heavy.integrate1000_x16` | 2.933 ms | 0.852 ms | -70.9%  | +/-9% | 3.44x | ok |
| `heavy.integrate1000_x64` | 11.368 ms | 2.982 ms | -73.8%  | +/-6% | 3.81x | ok |
| `single.ode_rk4_10k` | 6.758 ms | 1.728 ms | -74.4%  | +/-4% | 3.91x | ok |

- Significant regressions: **0**
- Watch (over threshold but within noise): **26** -> ['protocol.spec', 'single.math_add', 'protocol.tools_list', 'single.stat_mean_100', 'single.dec_mul_exact', 'iopath.count_1000', 'cheap.scalar_8', 'iopath.cumsum_1000', 'single.int_pow_exact', 'batch.scalar_10', 'iopath.cumsum_10000', 'protocol.floor_arg_error', 'iopath.count_10000', 'single.expr_eval', 'cheap.scalar_32', 'protocol.find', 'single.mat_mul_100', 'single.mat_shape_100', 'pipeline.chain_10', 'cheap.scalar_2', 'cheap.scalar_4', 'pipeline.chain_100', 'rescan.cumsum100_x10', 'dependent.chain_64', 'single.signal_fft_2048', 'cheap.scalar_16']
- Fingerprint mismatches: **0**

