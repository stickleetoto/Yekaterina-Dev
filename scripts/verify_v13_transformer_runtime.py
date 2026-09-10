from __future__ import annotations

import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "golden"))
from mcp_client import StdioMcpClient, mcp_text

MANIFEST = ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json"
FIXTURES = ROOT / "full_audit" / "fixtures_v13_transformer.json"
EXPECTED_COUNT = 15


def decode(sample) -> dict:
    text = mcp_text(sample.response)
    if not text:
        raise AssertionError("empty MCP tool response")
    value = json.loads(text)
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object, got {value!r}")
    return value


def approx(value: float, expected: float, rel: float = 1e-9, abs_: float = 1e-12) -> None:
    if not math.isclose(value, expected, rel_tol=rel, abs_tol=abs_):
        raise AssertionError(f"{value!r} != {expected!r}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: verify_v13_transformer_runtime.py <yekaterina-executable>", file=sys.stderr)
        return 2

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    fixtures_doc = json.loads(FIXTURES.read_text(encoding="utf-8"))
    opcodes = manifest["opcodes"]
    fixtures = fixtures_doc["fixtures"]

    assert manifest.get("registry_status") == "registered"
    assert manifest.get("count") == EXPECTED_COUNT == len(opcodes)
    assert fixtures_doc.get("count") == EXPECTED_COUNT
    assert set(fixtures) == set(opcodes)

    with StdioMcpClient(sys.argv[1], timeout=20.0) as client:
        found = decode(client.tool_call("yk.find", {"q": "xfmr", "l": 20}))
        hits = found.get("r")
        if not isinstance(hits, list):
            raise AssertionError(f"yk.find returned malformed result: {found!r}")
        if set(hits) != set(opcodes) or len(hits) != EXPECTED_COUNT:
            raise AssertionError(f"yk.find xfmr mismatch: {hits!r}")

        for opcode in opcodes:
            spec = decode(client.tool_call("yk.spec", {"op": opcode}))
            if spec.get("op") != opcode:
                raise AssertionError(f"yk.spec did not resolve {opcode}: {spec!r}")
            if spec.get("s") != "b" or spec.get("c") != "d":
                raise AssertionError(f"unexpected built-in/capability contract for {opcode}: {spec!r}")
            if not isinstance(spec.get("a"), list) or not spec.get("r"):
                raise AssertionError(f"malformed compact spec for {opcode}: {spec!r}")

            result = decode(client.tool_call("yk.compute", {"op": opcode, "a": fixtures[opcode]}))
            if "e" in result:
                raise AssertionError(f"{opcode} fixture failed: {result!r}")
            if "r" not in result:
                raise AssertionError(f"{opcode} fixture returned no result: {result!r}")

        # Independent semantic spot checks make this more than a reachability test.
        bh = decode(client.tool_call(
            "yk.compute",
            {"op": "xfmr.bh_field_strength", "a": fixtures["xfmr.bh_field_strength"]},
        ))["r"]
        approx(float(bh), 250.0)

        mag = decode(client.tool_call(
            "yk.compute",
            {"op": "xfmr.magnetizing_current_bh", "a": fixtures["xfmr.magnetizing_current_bh"]},
        ))["r"]
        approx(float(mag), 0.4)

        geometry = decode(client.tool_call(
            "yk.compute",
            {"op": "xfmr.winding_geometry", "a": fixtures["xfmr.winding_geometry"]},
        ))["r"]
        if geometry.get("layers") != 10 or geometry.get("last_layer_turns") != 100:
            raise AssertionError(f"winding geometry integer occupancy drifted: {geometry!r}")
        approx(float(geometry["radial_build_mm"]), 13.8)
        approx(float(geometry["axial_height_mm"]), 210.0)

        thermal = decode(client.tool_call(
            "yk.compute",
            {"op": "xfmr.thermal_two_node", "a": fixtures["xfmr.thermal_two_node"]},
        ))["r"]
        approx(float(thermal["oil_c"]), 40.0)
        approx(float(thermal["winding_c"]), 60.0)
        approx(float(thermal["core_c"]), 55.0)

        ranked = decode(client.tool_call(
            "yk.compute",
            {"op": "xfmr.candidate_rank", "a": fixtures["xfmr.candidate_rank"]},
        ))["r"]["ranking"]
        if [entry["index"] for entry in ranked] != [1, 0, 2]:
            raise AssertionError(f"candidate ranking policy drifted: {ranked!r}")
        if not ranked[0]["pass"] or ranked[2]["pass"]:
            raise AssertionError(f"feasible-first ranking drifted: {ranked!r}")

        legacy = decode(client.tool_call("yk.compute", {"op": "math.add", "a": [20, 22]}))
        if legacy.get("r") != 42:
            raise AssertionError(f"legacy dispatch regressed: {legacy!r}")

    print("V1.3 TRANSFORMER RUNTIME PASS")
    print(f"discoverable native xfmr operations: {EXPECTED_COUNT}/{EXPECTED_COUNT}")
    print("yk.find / yk.spec / yk.compute: PASS")
    print("semantic spot checks: PASS")
    print("legacy math.add dispatch: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
