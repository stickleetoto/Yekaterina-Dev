from __future__ import annotations
import json, re
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
registry=(ROOT/'src/registry.rs').read_text(encoding='utf-8')
ops=set(re.findall(r'^\s*op\("([^"]+)"',registry,re.M))
data=json.loads((ROOT/'golden/cases.json').read_text(encoding='utf-8-sig'))
cases=data.get('cases',[])
errors=[]
ids=[]
for c in cases:
    ids.append((c.get('category'),c.get('id')))
    if c.get('op') not in ops: errors.append(f"unregistered op in golden case {c.get('id')}: {c.get('op')}")
    if ('expect' in c)==('error' in c): errors.append(f"case must have exactly one of expect/error: {c.get('id')}")
if len(ids)!=len(set(ids)): errors.append('duplicate category/id in golden corpus')
if len(cases)<527: errors.append(f'v1.0.0 requires >=527 golden cases, got {len(cases)}')
cats=Counter(c.get('category') for c in cases)
if len(cats)<44: errors.append(f'v1.0.0 requires >=44 golden categories, got {len(cats)}')
if errors:
    for e in errors: print('FAIL:',e)
    raise SystemExit(1)
print(f'PASS: golden manifest {len(cases)} cases / {len(cats)} categories')
print('PASS: every golden opcode is registered')
