#!/usr/bin/env python3
"""Fast release-candidate surface gate for Yekaterina v1.2."""
from __future__ import annotations

import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_VERSION = "1.2.0"
EXPECTED_OPS = 1410
EXPECTED_TOOLS = ["yk.compute", "yk.find", "yk.spec"]
EXPECTED_ADVERTISED_VERSION = "1.0.0"
FROZEN_MODEL_SHA256 = "08a0222b5c646f8afc57932b8fb8d56fc36870fad25da014be06bca6180f6faf"

REQUIRED_RC_FILES = [
    "SHOWCASE.md",
    "DEMO_WINDOWS.bat",
    "DEMO_UNIX.sh",
    "tools/demo.py",
    "docs/CODEX_SETUP.md",
    "docs/V12_RC1.md",
]


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def main() -> int:
    failures: list[str] = []

    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    version_match = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.M)
    version = version_match.group(1) if version_match else None
    if version != EXPECTED_VERSION:
        fail(f"crate version {version!r} != {EXPECTED_VERSION!r}", failures)

    registry = (ROOT / "src" / "registry.rs").read_text(encoding="utf-8")
    ops = re.findall(r'^\s*op\("([^"]+)"', registry, re.M)
    if len(ops) != EXPECTED_OPS:
        fail(f"registered opcode count {len(ops)} != {EXPECTED_OPS}", failures)
    if len(set(ops)) != len(ops):
        fail("registered opcode list contains duplicates", failures)

    server = (ROOT / "src" / "server.rs").read_text(encoding="utf-8")
    tools = re.findall(r'name\s*=\s*"(yk\.[^"]+)"', server)
    if tools != EXPECTED_TOOLS:
        fail(f"MCP tool surface {tools!r} != {EXPECTED_TOOLS!r}", failures)

    identity = re.search(
        r"#\[tool_handler\((.*?)\)\]\s*impl ServerHandler",
        server,
        re.S,
    )
    if identity is None:
        fail("missing #[tool_handler(...)] server identity", failures)
    elif f'version = "{EXPECTED_ADVERTISED_VERSION}"' not in identity.group(1):
        fail(
            "advertised MCP version drifted from "
            f"{EXPECTED_ADVERTISED_VERSION!r}",
            failures,
        )

    model_bytes = (ROOT / "src" / "model.rs").read_bytes()
    model_hash = hashlib.sha256(model_bytes).hexdigest()
    if model_hash != FROZEN_MODEL_SHA256:
        fail(f"src/model.rs schema hash drifted: {model_hash}", failures)

    missing = [rel for rel in REQUIRED_RC_FILES if not (ROOT / rel).is_file()]
    if missing:
        fail(f"missing RC/showcase files: {missing}", failures)

    if not failures:
        showcase = (ROOT / "SHOWCASE.md").read_text(encoding="utf-8")
        if "1,410" not in showcase or "3" not in showcase:
            fail("SHOWCASE.md no longer states the frozen RC surface", failures)

    if failures:
        print("RC GATE FAIL")
        for message in failures:
            print("FAIL:", message)
        return 1

    print("RC GATE PASS")
    print(f"crate version           {EXPECTED_VERSION}")
    print(f"registered opcodes      {EXPECTED_OPS}")
    print("MCP tools               " + ", ".join(EXPECTED_TOOLS))
    print(f"advertised MCP version  {EXPECTED_ADVERTISED_VERSION}")
    print("frozen request schema   PASS")
    print("showcase/demo surface   PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
