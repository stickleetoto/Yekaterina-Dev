from __future__ import annotations
import json, sys
from pathlib import Path

p=Path(sys.argv[1] if len(sys.argv)>1 else 'golden_results/latest/result.json')
d=json.loads(p.read_text(encoding='utf-8-sig'))
errors=[]
if d.get('total',0) < 527: errors.append(f"golden corpus too small: {d.get('total')}")
if not d.get('tool_surface_pass'): errors.append('MCP tool surface is not exactly 3 expected tools')
if d.get('accuracy') != 1.0: errors.append(f"overall accuracy is {d.get('accuracy')}")
for name,c in d.get('categories',{}).items():
    if c.get('accuracy') != 1.0: errors.append(f"{name} accuracy is {c.get('accuracy')}")
if errors:
    for e in errors: print('FAIL:',e)
    raise SystemExit(1)
print(f"PASS: golden gate {d.get('passed')}/{d.get('total')} cases, 100% accuracy, 3-tool MCP surface")
