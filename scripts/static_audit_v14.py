from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V12_COUNT = 1410
V13_COUNT = 15
V14_COUNT = 2
TOTAL_COUNT = V12_COUNT + V13_COUNT + V14_COUNT
EXPECTED_DEV_PACKAGE_VERSION = "1.3.0"
EXPECTED_TOOLS = ["yk.compute", "yk.find", "yk.spec"]

# Git blob ids from the final v1.3.0 development release commit. v1.4 is
# additive: these historical layer files are not rewritten in place.
FROZEN_V13_BLOBS = {
    "src/registry_v13.rs": "00cec483efa0200cb21b44e58b993df857893a30",
    "src/engine_v13.rs": "9f58404388cbb99e154a77852485bc0d53c11d06",
    "scripts/static_audit_v13.py": "3f136bf898463b3ed48f62ae13725dbcfe51fe61",
}

failures: list[str] = []
passes: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def ok(msg: str) -> None:
    passes.append(msg)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_blob_sha1(path: Path) -> str:
    data = path.read_bytes()
    header = f"blob {len(data)}\0".encode()
    return hashlib.sha1(header + data).hexdigest()


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
    frozen_v12 = [
        "src/model.rs",
        "src/server.rs",
        "src/registry.rs",
        "src/engine.rs",
        "scripts/static_audit.py",
        "scripts/static_audit_v11.py",
        "scripts/static_audit_v12.py",
    ]
    for rel in frozen_v12:
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

    for rel, expected in FROZEN_V13_BLOBS.items():
        path = ROOT / rel
        if not path.is_file():
            fail(f"frozen v1.3 layer artifact missing: {rel}")
        elif git_blob_sha1(path) != expected:
            fail(f"frozen v1.3 layer artifact drifted: {rel}")
        else:
            ok(f"frozen v1.3 layer artifact preserved: {rel}")

    # Early v1.4 development intentionally retains 1.3.0 package metadata so
    # the release version is not promoted before the feature line is accepted.
    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    package_version = re.search(r'^\s*version\s*=\s*"([^"]+)"', cargo_toml, re.M)
    if package_version is None or package_version.group(1) != EXPECTED_DEV_PACKAGE_VERSION:
        fail("v1.4 dev branch unexpectedly changed Cargo.toml release metadata")
    else:
        ok("v1.4 dev branch keeps package metadata at 1.3.0 until promotion")

    cargo_lock = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
    locked_version = re.search(
        r'\[\[package\]\]\s+name\s*=\s*"yekaterina"\s+version\s*=\s*"([^"]+)"',
        cargo_lock,
        re.M,
    )
    if locked_version is None or locked_version.group(1) != EXPECTED_DEV_PACKAGE_VERSION:
        fail("v1.4 dev branch unexpectedly changed Cargo.lock release metadata")
    else:
        ok("Cargo.lock remains synchronized with the dev package metadata")

    legacy_manifest = json.loads(
        (ROOT / "full_audit" / "opcodes_alpha12.json").read_text(encoding="utf-8-sig")
    )
    v12_ops = legacy_manifest.get("opcodes", [])
    if legacy_manifest.get("count") != V12_COUNT or len(v12_ops) != V12_COUNT:
        fail("v1.2 opcode manifest is not the expected 1410-op baseline")
    elif len(set(v12_ops)) != V12_COUNT:
        fail("v1.2 opcode manifest contains duplicates")
    else:
        ok("v1.2 opcode manifest remains 1410 unique operations")

    legacy_registry = (ROOT / "src" / "registry.rs").read_text(encoding="utf-8")
    registered_v12 = re.findall(r'^\s*op\("([^"]+)"', legacy_registry, re.M)
    if registered_v12 != v12_ops:
        fail("frozen src/registry.rs no longer exactly matches the v1.2 manifest")
    else:
        ok("frozen src/registry.rs exactly matches the v1.2 manifest in order")

    v13_manifest = json.loads(
        (ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json").read_text(encoding="utf-8")
    )
    v13_ops = v13_manifest.get("opcodes", [])
    v13_registry = (ROOT / "src" / "registry_v13.rs").read_text(encoding="utf-8")
    registered_v13 = re.findall(r'^\s*op\("(xfmr\.[^"]+)"', v13_registry, re.M)
    if v13_manifest.get("count") != V13_COUNT or v13_manifest.get("registry_status") != "registered":
        fail("v1.3 transformer manifest is not the frozen 15-op registered layer")
    elif v13_ops != registered_v13:
        fail("v1.3 registry layer no longer matches its manifest in order")
    else:
        ok("v1.3 transformer layer remains 15 registered operations")

    v14_manifest = json.loads(
        (ROOT / "full_audit" / "opcodes_v14_math_agent.json").read_text(encoding="utf-8")
    )
    v14_ops = v14_manifest.get("opcodes", [])
    v14_registry = (ROOT / "src" / "registry_v14.rs").read_text(encoding="utf-8")
    registered_v14 = re.findall(r'^\s*op\(\s*\n?\s*"([^"]+)"', v14_registry, re.M)
    if v14_manifest.get("count") != V14_COUNT or v14_manifest.get("registry_status") != "registered":
        fail("v1.4 math manifest is not exactly two registered operations")
    elif v14_ops != registered_v14:
        fail(f"v1.4 registry specs do not match manifest order: {registered_v14}")
    else:
        ok("v1.4 math layer is exactly two registered operations")

    all_ops = v12_ops + v13_ops + v14_ops
    if len(all_ops) != TOTAL_COUNT or len(set(all_ops)) != TOTAL_COUNT:
        fail("v1.4 aggregate registry names are not 1427 unique operations")
    else:
        ok("v1.4 aggregate registry is 1427 unique operations")

    if "pub const BUILTIN_COUNT: usize = V13_BUILTIN_COUNT + V14_MATH_COUNT;" not in v14_registry:
        fail("registry_v14 aggregate count expression missing")
    else:
        ok("registry_v14 count expression is additive")

    lib = (ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
    required_wiring = [
        '#[path = "registry.rs"]\npub mod registry_v12;',
        '#[path = "registry_v13.rs"]\npub mod registry_v13;',
        '#[path = "registry_v14.rs"]\npub mod registry;',
        '#[path = "engine.rs"]\npub mod engine_v12;',
        '#[path = "engine_v13.rs"]\npub mod engine_v13;',
        '#[path = "engine_v14.rs"]\npub mod engine;',
        "pub mod math_v14;",
    ]
    missing_wiring = [needle for needle in required_wiring if needle not in lib]
    if missing_wiring:
        fail(f"v1.4 additive module wiring incomplete: {missing_wiring}")
    else:
        ok("v1.4 additive registry/engine/math wiring present")

    dispatch = (ROOT / "src" / "engine_v14.rs").read_text(encoding="utf-8")
    missing_dispatch = [op for op in v14_ops if f'"{op}"' not in dispatch and f'"{op}"' not in (ROOT / "src" / "math_v14.rs").read_text(encoding="utf-8")]
    if missing_dispatch:
        fail(f"v1.4 dispatch is missing operations: {missing_dispatch}")
    else:
        ok("all v1.4 operations are represented in the new dispatch layer")

    fixture_doc = json.loads(
        (ROOT / "full_audit" / "fixtures_v14_math_agent.json").read_text(encoding="utf-8")
    )
    fixtures = fixture_doc.get("fixtures", {})
    if fixture_doc.get("count") != V14_COUNT or set(fixtures) != set(v14_ops):
        fail("v1.4 runtime fixtures do not cover exactly the two new math operations")
    else:
        ok("v1.4 runtime fixtures cover both new math operations")

    for phrase in ["solve linear equation", "system of equations", "quadratic equation"]:
        if phrase not in v14_registry:
            fail(f"agent discovery hint missing: {phrase}")
        else:
            ok(f"agent discovery hint present: {phrase}")

    server = (ROOT / "src" / "server.rs").read_text(encoding="utf-8")
    tools = re.findall(r'name\s*=\s*"(yk\.[^"]+)"', server)
    if tools != EXPECTED_TOOLS:
        fail(f"MCP tool surface drifted: {tools}")
    else:
        ok("MCP tool surface remains exactly three tools")

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
