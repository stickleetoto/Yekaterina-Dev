from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

legacy_text = (ROOT / "src" / "registry.rs").read_text(encoding="utf-8")
v12_ops = re.findall(r'^\s*op\("([^"]+)"', legacy_text, re.M)

v13_text = (ROOT / "src" / "registry_v13.rs").read_text(encoding="utf-8")
v13_ops = re.findall(r'^\s*op\("(xfmr\.[^"]+)"', v13_text, re.M)
v13_manifest = json.loads(
    (ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json").read_text(encoding="utf-8")
)

v14_text = (ROOT / "src" / "registry_v14.rs").read_text(encoding="utf-8")
v14_ops = re.findall(r'^\s*op\(\s*\n?\s*"([^"]+)"', v14_text, re.M)
v14_manifest = json.loads(
    (ROOT / "full_audit" / "opcodes_v14_math_agent.json").read_text(encoding="utf-8")
)

if len(v12_ops) != 1410 or len(set(v12_ops)) != 1410:
    raise SystemExit(f"FAIL: v1.2 registry is {len(v12_ops)} entries / {len(set(v12_ops))} unique")
if v13_manifest.get("registry_status") != "registered" or v13_manifest.get("opcodes") != v13_ops:
    raise SystemExit("FAIL: v1.3 registry/manifest mismatch")
if v14_manifest.get("registry_status") != "registered" or v14_manifest.get("opcodes") != v14_ops:
    raise SystemExit(f"FAIL: v1.4 registry/manifest mismatch: {v14_ops}")

ops = v12_ops + v13_ops + v14_ops
if len(ops) != 1427 or len(set(ops)) != 1427:
    raise SystemExit(f"FAIL: aggregate registry is {len(ops)} entries / {len(set(ops))} unique, expected 1427")

counts = Counter(op.split('.', 1)[0] for op in ops)
print(f"total={len(ops)}")
for family, count in sorted(counts.items()):
    print(f"{family:8s} {count:4d}")
