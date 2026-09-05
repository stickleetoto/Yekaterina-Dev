# Yekaterina Benchmark - v1.0.0 FROZEN BASELINE (pristine source, sequential)

Scope: **MCP-only; no LLM, no network.**

## Environment

| Field | Value |
|---|---|
| Harness / workload set | v1.0 / v1 |
| Platform | Windows-11-10.0.26200-SP0 |
| Logical / physical CPUs | 16 / 10 |
| rustc | rustc 1.98.0 (88d9e12ae 2026-08-18) |
| Workers arg | `(default)` |
| Binary | 4,822,528 B, sha256 `753b64f8087de628...` |
| Source fingerprint | `4c48025e92be8769...` (55 files) |
| Runs x reps | 5 |

## MCP surface (frozen invariant)

- Tools: **3** -> ['yk.compute', 'yk.find', 'yk.spec']
- Schema bytes: **1725**
- Schema tokens: **412** (tiktoken:o200k_base)
- Frozen surface (3 tools / 412 tokens / 1725 bytes): **PASS**
- Serialization: `json.dumps({'tools':...}, separators=(',',':'), sort_keys=True)`

## Cold start

- p50 **6.500 ms**, p95 8.162 ms, min 6.186 ms (n=20, process spawn through MCP initialize)

## Workloads

### batch

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `batch.scalar_10` | 0.101 ms | 0.178 ms | 0.097 ms | 38.8% | 98,814 | 10.12 | ok |
| `batch.scalar_100` | 0.239 ms | 0.376 ms | 0.230 ms | 36.5% | 418,060 | 2.39 | ok |
| `batch.scalar_1000` | 2.383 ms | 2.953 ms | 2.383 ms | 16.2% | 419,622 | 2.38 | ok |
| `batch.scalar_10000_chunked` | 23.548 ms | 27.282 ms | 23.723 ms | 16.7% | 424,657 | 2.35 | ok |

### cheap

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `cheap.scalar_16` | 0.111 ms | 0.299 ms | 0.113 ms | 76.9% | 144,404 | 6.93 | ok |
| `cheap.scalar_2` | 0.085 ms | 0.163 ms | 0.080 ms | 27.9% | 23,502 | 42.55 | ok |
| `cheap.scalar_32` | 0.123 ms | 0.212 ms | 0.120 ms | 30.8% | 259,109 | 3.86 | ok |
| `cheap.scalar_4` | 0.087 ms | 0.149 ms | 0.081 ms | 35.2% | 46,003 | 21.74 | ok |
| `cheap.scalar_8` | 0.092 ms | 0.164 ms | 0.088 ms | 29.1% | 87,051 | 11.49 | ok |

### dependent

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `dependent.chain_64` | 0.176 ms | 0.259 ms | 0.171 ms | 34.2% | 364,465 | 2.74 | ok |
| `dependent.half_64` | 0.184 ms | 0.328 ms | 0.192 ms | 31.4% | 346,883 | 2.88 | ok |

### heavy

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `heavy.fft512_x16` | 5.479 ms | 7.638 ms | 5.309 ms | 41.3% | 2,920 | 342.42 | ok |
| `heavy.fft512_x32` | 11.872 ms | 14.128 ms | 11.763 ms | 21.8% | 2,695 | 370.99 | ok |
| `heavy.fft512_x8` | 2.650 ms | 3.093 ms | 2.578 ms | 16.0% | 3,019 | 331.26 | ok |
| `heavy.integrate1000_x16` | 2.925 ms | 3.211 ms | 2.933 ms | 7.1% | 5,471 | 182.79 | ok |
| `heavy.integrate1000_x4` | 0.838 ms | 0.993 ms | 0.839 ms | 10.8% | 4,771 | 209.60 | ok |
| `heavy.integrate1000_x64` | 11.368 ms | 12.956 ms | 11.368 ms | 5.8% | 5,630 | 177.63 | ok |
| `heavy.mixed_skew_16` | 7.439 ms | 8.235 ms | 7.416 ms | 8.7% | 2,151 | 464.92 | ok |

### iopath

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `iopath.count_1000` | 0.271 ms | 0.458 ms | 0.266 ms | 71.0% | 3,685 | - | ok |
| `iopath.count_10000` | 2.384 ms | 3.481 ms | 2.338 ms | 51.5% | 420 | - | ok |
| `iopath.count_50000` | 15.206 ms | 19.024 ms | 17.213 ms | 27.2% | 66 | - | ok |
| `iopath.cumsum_1000` | 0.358 ms | 0.771 ms | 0.341 ms | 90.6% | 2,793 | - | ok |
| `iopath.cumsum_10000` | 3.045 ms | 4.126 ms | 3.045 ms | 46.1% | 328 | - | ok |
| `iopath.cumsum_50000` | 18.698 ms | 23.686 ms | 20.136 ms | 30.8% | 53 | - | ok |

### pipeline

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `pipeline.chain_10` | 0.100 ms | 0.148 ms | 0.102 ms | 41.4% | 99,602 | 10.04 | ok |
| `pipeline.chain_100` | 0.204 ms | 0.338 ms | 0.203 ms | 25.0% | 489,237 | 2.04 | ok |
| `pipeline.chain_256` | 0.457 ms | 0.605 ms | 0.456 ms | 6.3% | 560,666 | 1.78 | ok |
| `pipeline.cumsum100_all_10` | 0.394 ms | 0.529 ms | 0.393 ms | 23.1% | 25,394 | 39.38 | ok |
| `pipeline.cumsum100_all_100` | 4.111 ms | 4.808 ms | 4.248 ms | 14.7% | 24,323 | 41.11 | ok |
| `pipeline.cumsum100_all_256` | 14.859 ms | 18.725 ms | 14.757 ms | 15.4% | 17,228 | 58.04 | ok |

### protocol

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `protocol.find` | 0.163 ms | 0.221 ms | 0.164 ms | 13.7% | 6,129 | - | ok |
| `protocol.floor_arg_error` | 0.082 ms | 0.143 ms | 0.083 ms | 45.8% | 12,262 | - | ok |
| `protocol.spec` | 0.064 ms | 0.106 ms | 0.064 ms | 12.0% | 15,576 | - | ok |
| `protocol.tools_list` | 0.093 ms | 0.172 ms | 0.087 ms | 47.6% | 10,695 | - | ok |

### rescan

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `rescan.cumsum100_x10` | 0.399 ms | 0.673 ms | 0.380 ms | 52.6% | 25,075 | 39.88 | ok |
| `rescan.cumsum100_x100` | 4.464 ms | 5.248 ms | 4.464 ms | 13.2% | 22,401 | 44.64 | ok |
| `rescan.cumsum100_x200` | 10.906 ms | 13.826 ms | 10.765 ms | 24.6% | 18,339 | 54.53 | ok |
| `rescan.cumsum100_x400` | 29.144 ms | 33.378 ms | 29.279 ms | 13.4% | 13,725 | 72.86 | ok |
| `rescan.cumsum100_x50` | 2.006 ms | 2.742 ms | 1.932 ms | 34.9% | 24,920 | 40.13 | ok |

### single

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `single.dec_mul_exact` | 0.085 ms | 0.159 ms | 0.073 ms | 63.3% | 11,779 | - | ok |
| `single.expr_eval` | 0.092 ms | 0.164 ms | 0.089 ms | 75.5% | 10,858 | - | ok |
| `single.int_pow_exact` | 0.123 ms | 0.276 ms | 0.108 ms | 90.1% | 8,143 | - | ok |
| `single.mat_mul_100` | 5.653 ms | 7.593 ms | 5.663 ms | 22.7% | 177 | - | ok |
| `single.mat_shape_100` | 2.518 ms | 3.664 ms | 2.418 ms | 28.8% | 397 | - | ok |
| `single.math_add` | 0.064 ms | 0.097 ms | 0.065 ms | 24.5% | 15,699 | - | ok |
| `single.num_integrate_10k` | 1.855 ms | 2.816 ms | 1.855 ms | 12.0% | 539 | - | ok |
| `single.ode_rk4_10k` | 6.763 ms | 7.280 ms | 6.758 ms | 2.8% | 148 | - | ok |
| `single.signal_fft_2048` | 1.475 ms | 2.058 ms | 1.430 ms | 26.1% | 678 | - | ok |
| `single.stat_mean_100` | 0.084 ms | 0.150 ms | 0.083 ms | 54.5% | 11,919 | - | ok |

### stateful

| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |
|---|---:|---:|---:|---:|---:|---:|:--:|
| `stateful.udo_formula_define` | 2.259 ms | 3.562 ms | 2.256 ms | 30.3% | 443 | - | ok |

## Process metrics (end of each run)

| Run | RSS | Peak RSS | CPU user | CPU sys | Threads |
|---:|---:|---:|---:|---:|---:|
| 1 | 11.65 MiB | 19.76 MiB | 1.656 s | 0.453 s | 23 |
| 2 | 11.32 MiB | 19.44 MiB | 1.797 s | 0.438 s | 22 |
| 3 | 11.85 MiB | 20.62 MiB | 1.844 s | 0.391 s | 23 |
| 4 | 11.52 MiB | 20.43 MiB | 1.656 s | 0.531 s | 23 |
| 5 | 12.17 MiB | 21.30 MiB | 1.906 s | 0.234 s | 24 |

## Integrity

- Non-deterministic responses across runs: **0** (none)
- Unexpected error envelopes: **0** (none)

