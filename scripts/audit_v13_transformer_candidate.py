from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "full_audit" / "opcodes_v13_transformer_candidate.json"
REGISTRY = ROOT / "src" / "registry.rs"
ENGINE = ROOT / "src" / "engine.rs"

EXPECTED = [
    "xfmr.bh_field_strength",
    "xfmr.magnetizing_current_bh",
    "xfmr.core_loss_specific_interp",
    "xfmr.core_loss_from_grid",
    "xfmr.winding_geometry",
    "xfmr.skin_depth",
    "xfmr.thermal_two_node",
    "xfmr.rogowski_factor",
    "xfmr.effective_leakage_height",
    "xfmr.leakage_inductance_concentric",
    "xfmr.leakage_reactance_concentric",
    "xfmr.dowell_foil_ac_factor",
    "xfmr.harmonic_copper_loss",
    "xfmr.candidate_evaluate",
    "xfmr.candidate_rank",
]

OWNERS = {
    "src/transformer.rs": EXPECTED[:7],
    "src/transformer_winding.rs": EXPECTED[7:13],
    "src/transformer_candidate.rs": EXPECTED[13:],
}

failures: list[str] = []
passes: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def ok(msg: str) -> None:
    passes.append(msg)


def main() -> int:
    if not MANIFEST.is_file():
        fail("candidate manifest missing")
        return finish()

    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    opcodes = data.get("opcodes")
    if opcodes != EXPECTED:
        fail("candidate manifest opcode list drifted from the staged v1.3 contract")
    else:
        ok("candidate manifest opcode order matches staged v1.3 contract")

    if data.get("count") != len(EXPECTED):
        fail(f"candidate manifest count must be {len(EXPECTED)}")
    elif len(set(EXPECTED)) != len(EXPECTED):
        fail("candidate manifest contains duplicate opcodes")
    else:
        ok(f"candidate manifest contains {len(EXPECTED)} unique native opcodes")

    if data.get("registry_status") != "unregistered":
        fail("candidate manifest must remain unregistered before migration")
    else:
        ok("candidate manifest is explicitly staged as unregistered")

    for rel, owned in OWNERS.items():
        path = ROOT / rel
        if not path.is_file():
            fail(f"missing candidate source: {rel}")
            continue
        text = path.read_text(encoding="utf-8")
        for opcode in owned:
            if text.count(f'"{opcode}"') != 1:
                fail(f"{opcode} must appear exactly once in owner source {rel}")
        if not any(msg.startswith(tuple(owned)) for msg in []):
            # no-op branch keeps the audit intentionally lexical and deterministic
            pass
        ok(f"{rel} owns {len(owned)} staged opcode literals")

    registry_text = REGISTRY.read_text(encoding="utf-8")
    registered = re.findall(r'^\s*op\("([^"]+)"', registry_text, re.M)
    if len(registered) != 1410:
        fail(f"frozen v1.2 registry must remain at 1410 operations, got {len(registered)}")
    else:
        ok("frozen v1.2 registry remains at 1410 operations")

    leaked = [opcode for opcode in EXPECTED if opcode in registered]
    if leaked:
        fail(f"candidate opcodes leaked into frozen v1.2 registry: {leaked}")
    else:
        ok("no staged xfmr opcode is discoverable through the v1.2 registry")

    engine_text = ENGINE.read_text(encoding="utf-8")
    if '"xfmr" =>' in engine_text:
        fail("xfmr dispatcher branch appeared before formal v1.3 migration")
    else:
        ok("engine has no xfmr dispatcher branch before migration")

    return finish()


def finish() -> int:
    for msg in passes:
        print(f"PASS: {msg}")
    for msg in failures:
        print(f"FAIL: {msg}")
    print(f"SUMMARY: {len(passes)} pass / {len(failures)} fail")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
