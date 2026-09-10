from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEGACY_REGISTRY = ROOT / "src" / "registry.rs"
V13_REGISTRY = ROOT / "src" / "registry_v13.rs"
V13_MANIFEST = ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json"

legacy_text = LEGACY_REGISTRY.read_text(encoding="utf-8")
legacy_ops = re.findall(r'^\s*op\("([^"]+)"', legacy_text, re.M)

v13_text = V13_REGISTRY.read_text(encoding="utf-8")
xfmr_ops = re.findall(r'^\s*op\("(xfmr\.[^"]+)"', v13_text, re.M)
manifest = json.loads(V13_MANIFEST.read_text(encoding="utf-8"))

if len(legacy_ops) != 1410 or len(set(legacy_ops)) != 1410:
    raise SystemExit(f"FAIL: legacy registry is {len(legacy_ops)} entries / {len(set(legacy_ops))} unique, expected 1410")
if manifest.get("registry_status") != "registered":
    raise SystemExit("FAIL: v1.3 transformer manifest is not marked registered")
if manifest.get("count") != 15 or manifest.get("opcodes") != xfmr_ops:
    raise SystemExit("FAIL: registry_v13 transformer operations do not match the 15-op manifest")
if set(legacy_ops) & set(xfmr_ops):
    raise SystemExit("FAIL: v1.3 transformer operations collide with legacy names")

ops = legacy_ops + xfmr_ops
if len(ops) != 1425 or len(set(ops)) != 1425:
    raise SystemExit(f"FAIL: aggregate registry is {len(ops)} entries / {len(set(ops))} unique, expected 1425")

counts = Counter(op.split('.', 1)[0] for op in ops)
print(f"total={len(ops)}")
for family, count in sorted(counts.items()):
    print(f"{family:8s} {count:4d}")
