from __future__ import annotations
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
EXPECTED=1387

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

# Loaded and checked before anything that depends on it: the deep-family gate
# below scopes itself to this list, so a corrupt list would surface there as a
# confusing failure in an unrelated check.
frozen=json.loads((ROOT/'full_audit/opcodes_v11_frozen.json').read_text(encoding='utf-8-sig'))
v11=frozen.get('opcodes',[])
if frozen.get('count')!=1215 or len(v11)!=1215 or len(set(v11))!=1215:
    print('FAIL: full_audit/opcodes_v11_frozen.json is corrupt')
    raise SystemExit(1)
v11_set=set(v11)

over=json.loads((ROOT/'full_audit/overrides_alpha12.json').read_text(encoding='utf-8-sig'))
# The alpha.12 override file covers the deep families as they stood in the v1.1
# line: exactly those 85 operations, no more and no fewer. Deep operations added
# after v1.1 are fixtured in fixtures_v12.json with everything else, so this gate
# keeps its original strength over the frozen set instead of being loosened to
# accommodate growth.
DEEP_FAMILIES={'linalg','special','optimize','ode','series'}
deep={op for op in ops if op.split('.',1)[0] in DEEP_FAMILIES and op in v11_set}
missing=sorted(deep-set(over));extra=sorted(set(over)-deep)
if missing or extra:
    if missing: print('FAIL: missing alpha.12 full-audit fixtures:',missing)
    if extra: print('FAIL: unknown alpha.12 full-audit fixtures:',extra)
    raise SystemExit(1)
if len(deep)!=85:
    print(f'FAIL: expected 85 alpha.12 deep ops, got {len(deep)}')
    raise SystemExit(1)

# The v1.1 line is frozen: every one of its 1,215 operations must still be
# registered, spelled identically, and in the same relative order. An expansion
# that quietly drops or renames an existing operation fails here.
dropped=[op for op in v11 if op not in set(ops)]
if dropped:
    print('FAIL: v1.1 operations missing from the registry:',dropped)
    raise SystemExit(1)
kept=[op for op in ops if op in set(v11)]
if kept!=v11:
    for i,(a,b) in enumerate(zip(kept,v11)):
        if a!=b:
            print(f'FAIL: v1.1 operation order changed at index {i}: {a!r} != {b!r}')
            break
    raise SystemExit(1)

V12_OPS=EXPECTED-1215
added=[op for op in ops if op not in set(v11)]
v12=json.loads((ROOT/'full_audit/fixtures_v12.json').read_text(encoding='utf-8-sig'))
fx=v12.get('fixtures',{})
if v12.get('count')!=len(fx) or len(fx)!=V12_OPS:
    print(f'FAIL: fixtures_v12.json declares {v12.get("count")} and holds {len(fx)}, expected {V12_OPS}')
    raise SystemExit(1)
overlap=sorted(set(fx)&set(over))
if overlap:
    print('FAIL: an opcode is fixtured in both files:',overlap)
    raise SystemExit(1)
if sorted(fx)!=sorted(added):
    print('FAIL: fixtures_v12.json does not cover exactly the new operations;',
          'uncovered:',sorted(set(added)-set(fx)),'unregistered:',sorted(set(fx)-set(added)))
    raise SystemExit(1)
print(f'PASS: full-audit opcode manifest = {EXPECTED}/{EXPECTED}')
print('PASS: alpha.12 full-audit curated fixtures = 85/85')
print(f'PASS: v1.2 curated fixtures = {V12_OPS}/{V12_OPS}, disjoint from alpha.12')
print('PASS: all 1215 v1.1 operations still registered, unrenamed and in order')
print(f'PASS: src/registry.rs exactly matches the fixed {EXPECTED}-op audit manifest')
print(f'NOTE: runtime agreement is additionally verified by live yk.spec {EXPECTED}/{EXPECTED} inside the audit.')
