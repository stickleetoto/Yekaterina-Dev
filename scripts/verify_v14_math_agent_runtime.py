from __future__ import annotations

import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "golden"))
from mcp_client import StdioMcpClient, mcp_text

MANIFEST = ROOT / "full_audit" / "opcodes_v14_math_agent.json"
FIXTURES = ROOT / "full_audit" / "fixtures_v14_math_agent.json"


def decode(sample) -> dict:
    text = mcp_text(sample.response)
    if not text:
        raise AssertionError("empty MCP tool response")
    value = json.loads(text)
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object, got {value!r}")
    return value


def approx(value: float, expected: float, tol: float = 1e-10) -> None:
    if not math.isclose(value, expected, rel_tol=tol, abs_tol=tol):
        raise AssertionError(f"{value!r} != {expected!r}")


def first_hit(client: StdioMcpClient, query: str) -> str:
    found = decode(client.tool_call("yk.find", {"q": query, "l": 5}))
    hits = found.get("r")
    if not isinstance(hits, list) or not hits:
        raise AssertionError(f"yk.find({query!r}) returned no usable hits: {found!r}")
    return str(hits[0])


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: verify_v14_math_agent_runtime.py <yekaterina-executable>", file=sys.stderr)
        return 2

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    fixtures_doc = json.loads(FIXTURES.read_text(encoding="utf-8"))
    opcodes = manifest["opcodes"]
    fixtures = fixtures_doc["fixtures"]
    assert manifest.get("registry_status") == "registered"
    assert manifest.get("count") == 2 == len(opcodes)
    assert fixtures_doc.get("count") == 2
    assert set(fixtures) == set(opcodes)

    with StdioMcpClient(sys.argv[1], timeout=20.0) as client:
        # New operations are discoverable by canonical name and compact spec.
        for opcode in opcodes:
            spec = decode(client.tool_call("yk.spec", {"op": opcode}))
            if spec.get("op") != opcode or spec.get("s") != "b" or spec.get("c") != "d":
                raise AssertionError(f"bad v1.4 spec for {opcode}: {spec!r}")
            result = decode(client.tool_call("yk.compute", {"op": opcode, "a": fixtures[opcode]}))
            if "e" in result or "r" not in result:
                raise AssertionError(f"v1.4 fixture failed for {opcode}: {result!r}")

        # Independent semantic checks for the math itself.
        root = decode(client.tool_call("yk.compute", {"op": "alg.linear_root", "a": [2, -8]}))
        approx(float(root["r"]), 4.0)

        solved = decode(client.tool_call(
            "yk.compute",
            {"op": "linalg.solve", "a": [[[2, 1], [1, -1]], [5, 1]]},
        ))["r"]
        approx(float(solved[0]), 2.0)
        approx(float(solved[1]), 1.0)

        singular = decode(client.tool_call(
            "yk.compute",
            {"op": "linalg.solve", "a": [[[1, 2], [2, 4]], [3, 6]]},
        ))
        if singular.get("e") != "DOMAIN":
            raise AssertionError(f"singular linear system should return DOMAIN: {singular!r}")

        # Agent-friendly search: natural language resolves to the intended op
        # without adding a fourth MCP tool or changing request schemas.
        expected_search = {
            "solve linear equation": "alg.linear_root",
            "system of equations": "linalg.solve",
            "quadratic equation": "alg.quadratic_roots",
            "greatest common divisor": "alg.gcd_many",
            "matrix inverse": "mat.inverse",
        }
        for query, expected in expected_search.items():
            actual = first_hit(client, query)
            if actual != expected:
                raise AssertionError(f"yk.find({query!r}) -> {actual!r}, expected {expected!r}")

        # Exact alias ownership and old runtime paths remain intact.
        if first_hit(client, "add") != "math.add":
            raise AssertionError("legacy exact alias ownership changed for add")
        legacy = decode(client.tool_call("yk.compute", {"op": "math.add", "a": [20, 22]}))
        if legacy.get("r") != 42:
            raise AssertionError(f"legacy math.add regressed: {legacy!r}")
        xfmr = decode(client.tool_call("yk.compute", {"op": "xfmr.skin_depth", "a": [60.0, 1.724e-8, 1.0]}))
        if "r" not in xfmr:
            raise AssertionError(f"v1.3 transformer dispatch regressed: {xfmr!r}")

    print("V1.4 MATH + AGENT RUNTIME PASS")
    print("new math operations: 2/2")
    print("natural-language yk.find intents: PASS")
    print("legacy alias ownership: PASS")
    print("v1.3 transformer regression: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
