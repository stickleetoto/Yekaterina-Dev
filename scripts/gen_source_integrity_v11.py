"""Regenerate SOURCE_INTEGRITY_V11.txt.

The v1.1 source integrity record. Covers everything the v1.0.0 record covered,
plus the modules, gates and harness v1.1 introduced.

SOURCE_INTEGRITY.txt -- the v1.0.0 record -- is listed here and is never
rewritten, so "the v1.0.0 record is preserved unmodified" is a checkable claim
rather than an assertion. Its own listed hashes still pin the v1.0.0 tree; the
files v1.1 deliberately changed (Cargo.toml, src/server.rs, CHANGELOG.md) will
not match it, which is the correct behaviour of a historical record.

`scripts/static_audit_v11.py` verifies this file, so run this generator after
any change to a listed file and commit both.
"""
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECORD = ROOT / "SOURCE_INTEGRITY_V11.txt"

SECTIONS = [
    ("manifest and toolchain", [
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]),
    ("frozen v1.0.0 MCP surface and engine", [
        "src/model.rs", "src/server.rs", "src/registry.rs", "src/engine.rs"]),
    ("v1.1 modules", [
        "src/lib.rs", "src/main.rs", "src/limits.rs", "src/safety.rs",
        "src/pool.rs", "src/scheduler.rs"]),
    ("acceptance corpora", [
        "golden/cases.json", "full_audit/run_full_audit.py",
        "full_audit/opcodes_alpha12.json", "full_audit/overrides_alpha12.json"]),
    ("gates", [
        "scripts/static_audit.py", "scripts/static_audit_v11.py",
        "scripts/validate_golden_manifest.py", "scripts/validate_full_audit.py",
        "scripts/gen_source_integrity_v11.py"]),
    ("benchmark harness", [
        "bench/bench_client.py", "bench/workloads.py", "bench/run_bench.py",
        "bench/paired_ab.py", "bench/stress.py"]),
    ("documentation and prior record", [
        "README.md", "CHANGELOG.md", "docs/V1_RELEASE.md", "docs/V11_BASELINE.md",
        "docs/V11_SAFETY_MODEL.md", "docs/V11_PARALLEL_MODEL.md",
        "docs/V11_DEVELOPMENT_STATUS.md", "SOURCE_INTEGRITY.txt"]),
]

HEADER = [
    "Yekaterina v1.1.0 source integrity",
    "",
    "Regenerate with: python scripts/gen_source_integrity_v11.py",
    "Verified by:     python scripts/static_audit_v11.py",
    "",
    "The v1.0.0 record is SOURCE_INTEGRITY.txt and is unchanged; it is hashed",
    "in the last section here so that claim is verifiable.",
    "",
]


def tracked_files() -> list[str]:
    return [rel for _, files in SECTIONS for rel in files]


def digest(rel: str) -> str:
    return hashlib.sha256((ROOT / rel).read_bytes()).hexdigest()


def main() -> None:
    missing = [rel for rel in tracked_files() if not (ROOT / rel).is_file()]
    if missing:
        raise SystemExit(f"missing files, refusing to write a partial record: {missing}")

    lines = list(HEADER)
    for title, files in SECTIONS:
        lines.append(f"# {title}")
        lines.extend(f"{digest(rel)}  {rel}" for rel in files)
        lines.append("")
    RECORD.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {RECORD.name}: {len(tracked_files())} files")


if __name__ == "__main__":
    main()
