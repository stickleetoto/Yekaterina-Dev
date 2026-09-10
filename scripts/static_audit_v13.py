from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V12_COUNT = 1410
XFMR_COUNT = 15
TOTAL_COUNT = 1425
EXPECTED_TOOLS = ["yk.compute", "yk.find", "yk.spec"]

failures: list[str] = []
passes: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def ok(msg: str) -> None:
    passes.append(msg)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_integrity_map() -> dict[str, str]:
    record = ROOT / "SOURCE_INTEGRITY_V12.txt"
    out: dict[str, str] = {}
    for line in record.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^([0-9a-f]{64})\s\s(\S.*)$", line)
        if m:
            out[m.group(2)] = m.group(1)
    return out


def main() -> int:
    integrity = source_integrity_map()
    frozen = [
        "src/model.rs",
        "src/server.rs",
        "src/registry.rs",
        "src/engine.rs",
        "scripts/static_audit.py",
        "scripts/static_audit_v11.py",
        "scripts/static_audit_v12.py",
    ]
    for rel in frozen:
        expected = integrity.get(rel)
        path = ROOT / rel
        if expected is None:
            fail(f"v1.2 integrity record does not pin {rel}")
        elif not path.is_file():
            fail(f"frozen v1.2 artifact missing: {rel}")
        elif sha256(path) != expected:
            fail(f"frozen v1.2 artifact drifted: {rel}")
        else:
            ok(f"frozen v1.2 artifact preserved: {rel}")

    legacy_manifest = json.loads(
        (ROOT / "full_audit" / "opcodes_alpha12.json").read_text(encoding="utf-8-sig")
    )
    legacy_ops = legacy_manifest.get("opcodes", [])
    if legacy_manifest.get("count") != V12_COUNT or len(legacy_ops) != V12_COUNT:
        fail("v1.2 opcode manifest is not the expected 1410-op baseline")
    elif len(set(legacy_ops)) != V12_COUNT:
        fail("v1.2 opcode manifest contains duplicates")
    else:
        ok("v1.2 opcode manifest remains 1410 unique operations")

    legacy_registry = (ROOT / "src" / "registry.rs").read_text(encoding="utf-8")
    registered_v12 = re.findall(r'^\s*op\("([^"]+)"', legacy_registry, re.M)
    if registered_v12 != legacy_ops:
        fail("frozen src/registry.rs no longer exactly matches the v1.2 opcode manifest")
    else:
        ok("frozen src/registry.rs exactly matches the v1.2 manifest in order")

    staged = json.loads(
        (ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json").read_text(encoding="utf-8")
    )
    xfmr_ops = staged.get("opcodes", [])
    if staged.get("count") != XFMR_COUNT or len(xfmr_ops) != XFMR_COUNT:
        fail("v1.3 transformer manifest is not exactly 15 operations")
    elif len(set(xfmr_ops)) != XFMR_COUNT:
        fail("v1.3 transformer manifest contains duplicates")
    elif staged.get("registry_status") != "registered":
        fail("v1.3 transformer manifest is not marked registered")
    else:
        ok("v1.3 transformer manifest is 15 unique registered operations")

    overlap = sorted(set(legacy_ops) & set(xfmr_ops))
    if overlap:
        fail(f"v1.3 transformer names collide with v1.2 operations: {overlap}")
    else:
        ok("v1.3 transformer names are disjoint from the v1.2 registry")

    aggregate = (ROOT / "src" / "registry_v13.rs").read_text(encoding="utf-8")
    aggregate_ops = re.findall(r'^\s*op\("(xfmr\.[^"]+)"', aggregate, re.M)
    if aggregate_ops != xfmr_ops:
        fail("registry_v13 transformer specs do not exactly match manifest order")
    else:
        ok("registry_v13 transformer specs exactly match manifest order")
    if "pub const BUILTIN_COUNT: usize = V12_BUILTIN_COUNT + V13_TRANSFORMER_COUNT;" not in aggregate:
        fail("registry_v13 aggregate count expression missing")
    elif V12_COUNT + len(aggregate_ops) != TOTAL_COUNT:
        fail("aggregate built-in count arithmetic does not reach 1425")
    else:
        ok("aggregate built-in count is 1425")

    lib = (ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
    required_wiring = [
        '#[path = "registry.rs"]\npub mod registry_v12;',
        '#[path = "registry_v13.rs"]\npub mod registry;',
        '#[path = "engine.rs"]\npub mod engine_v12;',
        '#[path = "engine_v13.rs"]\npub mod engine;',
        "pub mod transformer;",
        "pub mod transformer_winding;",
        "pub mod transformer_candidate;",
    ]
    missing_wiring = [needle for needle in required_wiring if needle not in lib]
    if missing_wiring:
        fail(f"v1.3 module wiring incomplete: {missing_wiring}")
    else:
        ok("v1.3 aggregate registry/engine/module wiring present")

    dispatch = (ROOT / "src" / "engine_v13.rs").read_text(encoding="utf-8")
    missing_dispatch = [op for op in xfmr_ops if f'"{op}"' not in dispatch]
    if missing_dispatch:
        fail(f"v1.3 dispatcher is missing transformer operations: {missing_dispatch}")
    else:
        ok("all 15 transformer operations are represented in v1.3 dispatcher")

    fixture_doc = json.loads(
        (ROOT / "full_audit" / "fixtures_v13_transformer.json").read_text(encoding="utf-8")
    )
    fixtures = fixture_doc.get("fixtures", {})
    if fixture_doc.get("count") != XFMR_COUNT or set(fixtures) != set(xfmr_ops):
        fail("v1.3 runtime fixtures do not cover exactly the 15 transformer operations")
    else:
        ok("v1.3 runtime fixtures cover all 15 transformer operations")

    server = (ROOT / "src" / "server.rs").read_text(encoding="utf-8")
    tools = re.findall(r'name\s*=\s*"(yk\.[^"]+)"', server)
    if tools != EXPECTED_TOOLS:
        fail(f"MCP tool surface drifted: {tools}")
    else:
        ok("MCP tool surface remains exactly three tools")

    if TOTAL_COUNT != V12_COUNT + XFMR_COUNT:
        fail("internal count invariant failed")

    return finish()


def finish() -> int:
    for msg in passes:
        print("PASS:", msg)
    for msg in failures:
        print("FAIL:", msg)
    print(f"SUMMARY: {len(passes)} pass / {len(failures)} fail")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
