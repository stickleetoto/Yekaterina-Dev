from __future__ import annotations
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "src" / "registry.rs"

text = REGISTRY.read_text(encoding="utf-8")
ops = re.findall(r'^\s*op\("([^"]+)"', text, re.M)
counts = Counter(op.split('.', 1)[0] for op in ops)
print(f"total={len(ops)}")
for family, count in sorted(counts.items()):
    print(f"{family:8s} {count:4d}")
