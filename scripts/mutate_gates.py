"""Break what the gates guard, and check that each one notices.

Run:  python scripts/mutate_gates.py

A gate that has never failed is a guess. Every mutation below is applied to the
real tree, the gate is run, and the tree is restored in a `finally` block.

Two of these exist because the first attempt at them proved nothing: deleting or
renaming an operation was caught, but by the manifest count and order checks that
run earlier, not by the frozen-list gate under test. The mutations regenerate the
manifest to match, which is what a maintainer would do, so only the frozen list
can see the loss.

A gate that has never failed is a guess. These mutations are applied to the real
tree, the validator is run, and the tree is restored in a finally block.
"""
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ["python", "scripts/validate_full_audit.py"]
AUDIT = ["python", "scripts/static_audit_v12.py"]


def run(cmd):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def check(label, expect_fragment, mutate, cmd=VALIDATOR):
    """Apply `mutate` to the tree, run `cmd`, restore, and report."""
    targets = mutate()
    saved = {p: (ROOT / p).read_bytes() for p in targets}
    try:
        for p, data in targets.items():
            (ROOT / p).write_bytes(data)
        code, out = run(cmd)
        caught = code != 0 and expect_fragment in out
        print(f"{'CAUGHT ' if caught else 'MISSED!'} {label}")
        if not caught:
            print("    exit", code)
            for line in out.strip().splitlines()[-4:]:
                print("    " + line)
    finally:
        for p, data in saved.items():
            (ROOT / p).write_bytes(data)


def reg_text():
    return (ROOT / "src/registry.rs").read_text(encoding="utf-8")


def _manifest_from(registry_bytes):
    """Regenerate the manifest from a registry, as a maintainer would."""
    import re as _re
    text = registry_bytes.decode("utf-8")
    body = text[text.index("pub const OPERATIONS"):]
    ops = _re.findall(r'^\s*op\("([^"]+)"', body, _re.M)
    man = json.loads((ROOT / "full_audit/opcodes_alpha12.json").read_text(encoding="utf-8"))
    man["opcodes"] = ops
    man["count"] = len(ops)
    return (json.dumps(man, indent=2) + "\n").encode("utf-8")


def rename_existing_operation():
    """Rename a v1.1 operation AND regenerate the manifest -- the count and the
    manifest still agree, so only the frozen-list gate can see the loss."""
    s = reg_text().replace('op("stat.median"', 'op("stat.median50"', 1).encode("utf-8")
    return {"src/registry.rs": s, "full_audit/opcodes_alpha12.json": _manifest_from(s)}


def reorder_existing_operations():
    """Reorder two v1.1 operations AND regenerate the manifest. A set comparison
    would pass; only the order check catches this."""
    lines = reg_text().splitlines(keepends=True)
    i = next(k for k, l in enumerate(lines) if 'op("math.sin"' in l)
    j = next(k for k, l in enumerate(lines) if 'op("math.cos"' in l)
    lines[i], lines[j] = lines[j], lines[i]
    s = "".join(lines).encode("utf-8")
    return {"src/registry.rs": s, "full_audit/opcodes_alpha12.json": _manifest_from(s)}


def drop_a_v12_fixture():
    p = ROOT / "full_audit/fixtures_v12.json"
    d = json.loads(p.read_text(encoding="utf-8"))
    d["fixtures"].pop("fin.irr")
    d["count"] = len(d["fixtures"])
    return {"full_audit/fixtures_v12.json": (json.dumps(d, indent=2) + "\n").encode("utf-8")}


def fixture_for_an_unregistered_op():
    p = ROOT / "full_audit/fixtures_v12.json"
    d = json.loads(p.read_text(encoding="utf-8"))
    d["fixtures"].pop("fin.irr")
    d["fixtures"]["fin.not_an_operation"] = [1]
    return {"full_audit/fixtures_v12.json": (json.dumps(d, indent=2) + "\n").encode("utf-8")}


def tamper_with_the_v11_audit():
    p = ROOT / "scripts/static_audit_v11.py"
    return {"scripts/static_audit_v11.py": (p.read_text(encoding="utf-8") + "\n# touched\n").encode("utf-8")}


def tamper_with_the_frozen_list():
    p = ROOT / "full_audit/opcodes_v11_frozen.json"
    d = json.loads(p.read_text(encoding="utf-8"))
    d["opcodes"] = d["opcodes"][:-1]
    return {"full_audit/opcodes_v11_frozen.json": (json.dumps(d, indent=2) + "\n").encode("utf-8")}


def main():
    print("mutation tests for the v1.2 gates\n")
    check("a v1.1 operation renamed, with the manifest regenerated to match",
          "v1.1 operations missing from the registry", rename_existing_operation)
    check("two v1.1 operations reordered, with the manifest regenerated to match",
          "v1.1 operation order changed", reorder_existing_operations)
    check("a v1.2 operation left without a fixture",
          "fixtures_v12.json declares", drop_a_v12_fixture)
    check("a fixture naming an unregistered opcode",
          "does not cover exactly the new operations", fixture_for_an_unregistered_op)
    check("the frozen v1.1 opcode list edited",
          "is corrupt", tamper_with_the_frozen_list)
    check("the frozen v1.1 audit script edited",
          "static_audit_v11.py was modified", tamper_with_the_v11_audit, cmd=AUDIT)

    code, out = run(VALIDATOR)
    print("\ntree restored, validator:", out.strip().splitlines()[0] if code == 0 else f"FAILED\n{out}")


if __name__ == "__main__":
    main()
