# Yekaterina MCP Real-World Benchmark

Scope: **MCP-only; no LLM/API**
Tokenizer: `tiktoken:o200k_base`

## 1. MCP surface

| Engine | Tools | Schema tokens | Schema bytes |
|---|---:|---:|---:|
| Arithma | 174 | 16896 | 67797 |
| Yekaterina | 3 | 412 | 1725 |

Schema token reduction: **97.56%**

## 2. Cold start / initialize

| Engine | Success | Start p50 | Start p95 | tools/list p50 | Mean RSS |
|---|---:|---:|---:|---:|---:|
| Arithma | 10/10 | 8.16 ms | 101.74 ms | 1.85 ms | 8.06 MiB |
| Yekaterina | 10/10 | 6.70 ms | 62.40 ms | 0.40 ms | 6.00 MiB |

## 3. Real stdio arithmetic workloads

| Workload | Engine | Calls | Wire tokens | Total MCP ms | p50/call | p95/call | Accuracy |
|---|---|---:|---:|---:|---:|---:|---:|
| mixed_arithmetic_100 | arithma | 100 | 7935 | 61.33 | 0.594 | 0.677 | 100.00% |
| mixed_arithmetic_100 | yekaterina | 1 | 1666 | 0.55 | 0.550 | 0.550 | 100.00% |
| ↳ reduction | YK vs Arithma | **99.00%** | **79.00%** |  |  |  |  |
| mixed_arithmetic_1000 | arithma | 1000 | 79422 | 673.69 | 0.629 | 0.804 | 100.00% |
| mixed_arithmetic_1000 | yekaterina | 1 | 15987 | 2.97 | 2.969 | 2.969 | 100.00% |
| ↳ reduction | YK vs Arithma | **99.90%** | **79.87%** |  |  |  |  |
| mixed_arithmetic_10000 | arithma | 10000 | 812100 | 6242.05 | 0.610 | 0.733 | 100.00% |
| mixed_arithmetic_10000 | yekaterina | 10 | 159794 | 25.91 | 2.655 | 2.789 | 100.00% |
| ↳ reduction | YK vs Arithma | **99.90%** | **80.32%** |  |  |  |  |

## 4. Yekaterina protocol modes

### Independent operations (1000 ops)

| Mode | Calls | Wire tokens | MCP ms | Accuracy |
|---|---:|---:|---:|---:|
| Serial | 1000 | 80950 | 74.61 | 100.00% |
| Batch | 1 | 15993 | 2.43 | 100.00% |

Batch call reduction vs serial: **99.90%**  
Batch wire-token reduction vs serial: **80.24%**

### Repeated 4-step workflow (250 tasks)

| Mode | Calls | Wire tokens | MCP ms | Accuracy |
|---|---:|---:|---:|---:|
| Serial | 1000 | 82113 | 70.88 | 100.00% |
| Pipeline | 250 | 27247 | 19.63 | 100.00% |
| Composite UDO | 250 | 19747 | 19.91 | 100.00% |

Pipeline vs serial: **75.00% fewer calls**, **66.82% fewer wire tokens**.  
Composite UDO vs serial: **75.00% fewer calls**, **75.95% fewer wire tokens**.  
Composite UDO vs pipeline wire-token reduction: **27.53%**.  
UDO one-time definition overhead: **131 wire tokens**.

## 5. Lazy discovery

| Tool | Calls | Wire tokens | p50 | p95 | Valid |
|---|---:|---:|---:|---:|---:|
| yk.find | 100 | 9300 | 0.077 ms | 0.139 ms | 100/100 |
| yk.spec | 100 | 9700 | 0.060 ms | 0.111 ms | 100/100 |

## 6. Resilience / recovery

### Arithma — PASS
- `unknown_tool` → recovery: **PASS**
- `divide_zero` → recovery: **PASS**

### Yekaterina — PASS
- `unknown_tool` → recovery: **PASS**
- `divide_zero` → recovery: **PASS**
- `bad_ref` → recovery: **PASS**
- `batch_limit` → recovery: **PASS**

## 7. UDO restart persistence

Persistence: **PASS**
Reloaded composite result: `28.0`
Listed after restart: `True`
Restart/initialize wall time: `11.21 ms`

## Interpretation rules

- This suite is **MCP-only**. It does not use OpenAI, Anthropic, Gemini, LM Studio, or any LLM API.
- Token counts describe serialized MCP/JSON-RPC protocol payloads using the selected tokenizer; they are not provider billing claims.
- Cold-start measurements include process creation and MCP `initialize`.
- Yekaterina UDO tests run under an isolated temporary `YEKATERINA_HOME`; the user's real UDO registry is not modified.
- Compare accuracy/recovery together with token/call reductions. A faster incorrect result is not counted as success.
