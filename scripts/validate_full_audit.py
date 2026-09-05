from __future__ import annotations
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
EXPECTED=1215

manifest=json.loads((ROOT/'full_audit/opcodes_alpha12.json').read_text(encoding='utf-8-sig'))
ops=manifest.get('opcodes',[])
if manifest.get('count')!=EXPECTED or len(ops)!=EXPECTED or len(set(ops))!=EXPECTED:
    print(f'FAIL: full_audit/opcodes_alpha12.json corrupt: count={manifest.get("count")}, entries={len(ops)}, unique={len(set(ops))}, expected={EXPECTED}')
    raise SystemExit(1)
if not all(isinstance(op,str) and op and '.' in op for op in ops):
    print('FAIL: opcode manifest contains invalid entries')
    raise SystemExit(1)

registry=(ROOT/'src/registry.rs').read_text(encoding='utf-8')
registry_ops=re.findall(r'\bop\(\s*"([^"]+)"', registry)
if len(registry_ops)!=EXPECTED or len(set(registry_ops))!=EXPECTED:
    print(f'FAIL: registry enumeration {len(registry_ops)} entries / {len(set(registry_ops))} unique, expected {EXPECTED}')
    raise SystemExit(1)
if registry_ops != ops:
    for i,(a,b) in enumerate(zip(registry_ops,ops)):
        if a!=b:
            print(f'FAIL: registry/manifest mismatch at index {i}: {a!r} != {b!r}')
            break
    else:
        print('FAIL: registry/manifest opcode sets differ')
    raise SystemExit(1)

allowed_returns={'value','number','integer','u64','boolean','string','opcode','number|null','object','pack','matrix','array','value[]','opcode[]','number[]','complex[]','number[2]','number[3]','integer[2]','integer[3]','point2'}
return_specs=[]
for line in registry.splitlines():
    if 'op("' not in line: continue
    m=re.search(r'\],\s*"([^"]+)",\s*"[^"]*"\),?\s*$',line)
    if not m:
        print('FAIL: unable to statically parse return spec for:',line[:160])
        raise SystemExit(1)
    return_specs.append(m.group(1))
unknown=sorted(set(return_specs)-allowed_returns)
if len(return_specs)!=EXPECTED or unknown:
    print(f'FAIL: return-contract vocabulary mismatch: parsed={len(return_specs)}, unknown={unknown}')
    raise SystemExit(1)

over=json.loads((ROOT/'full_audit/overrides_alpha12.json').read_text(encoding='utf-8-sig'))
deep={op for op in ops if op.split('.',1)[0] in {'linalg','special','optimize','ode','series'}}
missing=sorted(deep-set(over));extra=sorted(set(over)-deep)
if missing or extra:
    if missing: print('FAIL: missing alpha.12 full-audit fixtures:',missing)
    if extra: print('FAIL: unknown alpha.12 full-audit fixtures:',extra)
    raise SystemExit(1)
if len(deep)!=85:
    print(f'FAIL: expected 85 alpha.12 deep ops, got {len(deep)}')
    raise SystemExit(1)
print(f'PASS: full-audit opcode manifest = {EXPECTED}/{EXPECTED}')
print('PASS: alpha.12 full-audit curated fixtures = 85/85')
print('PASS: src/registry.rs exactly matches the fixed 1215-op audit manifest')
print('NOTE: runtime agreement is additionally verified by live yk.spec 1215/1215 inside the audit.')
