"""Register new operations: registry.rs, audit manifest, fixture file, counts.

Import it and call `register(new, anchors)` where `new` is a list of
(opcode, aliases, args, returns, summary, fixture) tuples and `anchors` maps each
family to an existing `op("...")` line to insert after.

Refuses to write if a new opcode or alias collides with anything already
registered. That check found two of the nine duplicates in the previous round,
so it runs before every batch.

Usage: import and call `register(NEW)` where NEW is a list of
(opcode, aliases, args, returns, summary, fixture) and every opcode's family
already has at least one entry in registry.rs to anchor the insertion.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def rust_list(items):
    return "&[" + ", ".join('"%s"' % i for i in items) + "]"


def register(new, anchors):
    reg_path = ROOT / "src/registry.rs"
    reg = reg_path.read_text(encoding="utf-8")

    existing = set(re.findall(r'op\("([^"]+)"', reg))
    aliases = set()
    for m in re.finditer(r'op\("([^"]+)",\s*&\[([^\]]*)\]', reg):
        aliases.update(re.findall(r'"([^"]+)"', m.group(2)))

    clashes, seen = [], set()
    for opcode, al, *_ in new:
        if opcode in existing:
            clashes.append("opcode already registered: " + opcode)
        if opcode in seen:
            clashes.append("duplicate opcode in the new set: " + opcode)
        seen.add(opcode)
        for a in al:
            if a in aliases or a in existing:
                clashes.append("alias collides with an existing name: %s (for %s)" % (a, opcode))
            if a in seen:
                clashes.append("duplicate alias in the new set: " + a)
            seen.add(a)
    if clashes:
        raise SystemExit("REFUSING TO WRITE:\n  " + "\n  ".join(clashes))

    by_family = {}
    for opcode, al, args, ret, summary, _fx in new:
        line = '    op("%s", %s, %s, "%s", "%s"),' % (
            opcode, rust_list(al), rust_list(args), ret, summary)
        by_family.setdefault(opcode.split(".", 1)[0], []).append(line)

    for family, lines in by_family.items():
        anchor = anchors[family]
        idx = reg.index(anchor)
        end = reg.index("\n", idx) + 1
        reg = reg[:end] + "\n".join(lines) + "\n" + reg[end:]
    reg_path.write_text(reg, encoding="utf-8")

    body = reg[reg.index("pub const OPERATIONS"):]
    ops = re.findall(r'^\s*op\("([^"]+)"', body, re.M)
    assert len(ops) == len(set(ops)), "duplicate opcode in registry.rs"

    man_path = ROOT / "full_audit/opcodes_alpha12.json"
    man = json.loads(man_path.read_text(encoding="utf-8"))
    man["opcodes"] = ops
    man["count"] = len(ops)
    man_path.write_text(json.dumps(man, indent=2) + "\n", encoding="utf-8")

    fx_path = ROOT / "full_audit/fixtures_v12.json"
    fx = json.loads(fx_path.read_text(encoding="utf-8"))
    for opcode, _al, _ar, _r, _s, fixture in new:
        fx["fixtures"][opcode] = fixture
    fx["count"] = len(fx["fixtures"])
    fx_path.write_text(json.dumps(fx, indent=2) + "\n", encoding="utf-8")

    # Keep every count that names the operation total in step.
    for rel, pattern in [
        ("full_audit/run_full_audit.py", r"EXPECTED_OPS=\d+"),
        ("scripts/validate_full_audit.py", r"EXPECTED=\d+"),
        ("scripts/static_audit_v12.py", r"EXPECTED_OPS = \d+"),
    ]:
        p = ROOT / rel
        t = p.read_text(encoding="utf-8")
        sep = " = " if " = " in re.search(pattern, t).group(0) else "="
        t = re.sub(pattern, pattern.split(r"\d")[0].replace("\\", "") + str(len(ops)), t, count=1)
        p.write_text(t, encoding="utf-8")

    p = ROOT / "src/safety.rs"
    t = p.read_text(encoding="utf-8")
    t = re.sub(r"assert_eq!\(pure, \d+\);", "assert_eq!(pure, %d);" % (len(ops) - 7), t, count=1)
    p.write_text(t, encoding="utf-8")

    print("registered %d operations; registry now %d; fixtures now %d"
          % (len(new), len(ops), fx["count"]))
    return len(ops)
