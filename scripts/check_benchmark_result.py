from __future__ import annotations
import json, sys
from pathlib import Path

if len(sys.argv) != 2:
    raise SystemExit("usage: python scripts/check_benchmark_result.py <result.json>")

data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8-sig"))
errors: list[str] = []

def require(cond: bool, msg: str) -> None:
    if not cond:
        errors.append(msg)

schema = data.get("schema", {})
yk = schema.get("yekaterina", {})
require(yk.get("tools") == 3, f"MCP tool count must be 3, got {yk.get('tools')}")
require(yk.get("tokens") == 412, f"alpha.12 schema token drift: expected frozen 412, got {yk.get('tokens')}")
require(schema.get("token_reduction_pct", 0) >= 95.0, f"schema reduction below 95%: {schema.get('token_reduction_pct')}")

arithmetic = data.get("arithmetic") or data.get("workloads") or []
require(bool(arithmetic), "missing arithmetic/workload results")
for row in arithmetic:
    y = row.get("yekaterina", {})
    require(y.get("accuracy") == 1.0, f"accuracy regression in {row.get('name')}: {y.get('accuracy')}")

large = next((r for r in arithmetic if r.get("n") == 10000), None)
if large:
    reduction = large.get("wire_token_reduction_pct", 0)
    require(reduction >= 78.0, f"10k wire-token reduction below 78%: {reduction}")

res = data.get("resilience", {}).get("yekaterina")
if res is not None:
    require(res.get("all_survived") is True, "resilience/recovery regression")

persist = data.get("udo_persistence")
if persist is not None:
    require(persist.get("pass") is True, "UDO restart persistence regression")

if errors:
    print("BENCHMARK GATE: FAIL")
    for e in errors:
        print(f"- {e}")
    raise SystemExit(1)

print("BENCHMARK GATE: PASS")
print(f"- tools: {yk.get('tools')}")
print(f"- schema tokens: {yk.get('tokens')} (frozen alpha.10 baseline)")
print(f"- schema reduction: {schema.get('token_reduction_pct'):.2f}%")
if large:
    print(f"- 10k wire-token reduction: {large.get('wire_token_reduction_pct'):.2f}%")
