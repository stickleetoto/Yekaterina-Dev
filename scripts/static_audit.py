from __future__ import annotations

import re
import hashlib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"FAIL: {msg}")
    raise SystemExit(1)


def struct_fields(text: str, name: str) -> list[str]:
    m = re.search(rf"pub struct {re.escape(name)}\s*\{{(.*?)\n\}}", text, re.S)
    if not m:
        fail(f"missing model struct {name}")
    return re.findall(r"^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:", m.group(1), re.M)


def main() -> None:
    registry_path = ROOT / "src" / "registry.rs"
    registry = registry_path.read_text(encoding="utf-8")
    server = (ROOT / "src" / "server.rs").read_text(encoding="utf-8")
    model = (ROOT / "src" / "model.rs").read_text(encoding="utf-8")
    ops = re.findall(r'^\s*op\("([^"]+)"', registry, re.M)
    counts = Counter(ops)
    duplicates = [op for op, n in counts.items() if n > 1]
    if duplicates:
        fail(f"duplicate opcodes: {duplicates}")
    if len(ops) != 1215:
        fail(f"v1.0.0 requires exactly 1215 registered opcodes, got {len(ops)} from {registry_path}")

    tools = re.findall(r'name\s*=\s*"(yk\.[^"]+)"', server)
    if tools != ["yk.compute", "yk.find", "yk.spec"]:
        fail(f"unexpected MCP tool surface: {tools}")

    # Token-surface invariant: v1.0.0 preserves the frozen internal-opcode surface, but no new MCP parameters.
    expected_fields = {
        "ComputeParams": ["op", "a", "ops", "pipe", "input", "all"],
        "FindParams": ["q", "l"],
        "SpecParams": ["op"],
    }
    for name, expected in expected_fields.items():
        got = struct_fields(model, name)
        if got != expected:
            fail(f"v1.0.0 MCP schema field drift in {name}: {got} != {expected}")

    # Source-level schema freeze against the verified alpha.10 tool surface.
    model_hash = hashlib.sha256(model.encode()).hexdigest()
    if model_hash != "08a0222b5c646f8afc57932b8fb8d56fc36870fad25da014be06bca6180f6faf":
        fail(f"v1.0.0 model schema source drifted from the frozen alpha.10 surface: {model_hash}")
    tool_chunks = re.findall(r'#\[tool\((.*?)\)\]\s*async fn (compute|find|spec)', server, re.S)
    tool_norm = "|".join(re.sub(r"\s+", " ", chunk).strip() + ":" + fn for chunk, fn in tool_chunks)
    tool_hash = hashlib.sha256(tool_norm.encode()).hexdigest()
    if tool_hash != "a000b4f460a1273846926031d2d5c831c5c5c9d04be3ef82068a46fd5ded64bc":
        fail(f"v1.0.0 MCP tool annotation schema drifted from the frozen alpha.10 surface: {tool_hash}")

    rust_files = list((ROOT / "src").glob("*.rs")) + list((ROOT / "tests").glob("*.rs"))
    escaped_newline_test = []
    for rust_file in rust_files:
        text = rust_file.read_text(encoding="utf-8")
        if r"\n#[test]" in text or r"\nfn " in text:
            escaped_newline_test.append(str(rust_file.relative_to(ROOT)))
    if escaped_newline_test:
        fail(f"literal escaped newlines found in Rust source/test files: {escaped_newline_test}")

    source_files = [p for p in (ROOT / "src").glob("*.rs") if p.name != "registry.rs"]
    all_src = "\n".join(p.read_text(encoding="utf-8") for p in source_files)
    forbidden = ["std::process::Command", "TcpStream", "UdpSocket", "reqwest::", "unsafe {"]
    found = [needle for needle in forbidden if needle in all_src]
    if found:
        fail(f"forbidden host execution/network patterns: {found}")

    missing_impl = [op for op in ops if f'"{op}"' not in all_src]
    if missing_impl:
        fail(f"registered opcodes missing implementation/control reference: {missing_impl}")

    for needed in ["udo.composite", "udo.export", "udo.import", "udo.uninstall", "expr.eval"]:
        if needed not in ops:
            fail(f"missing required UDO/control opcode: {needed}")

    base_families = ["math.", "stat.", "vec.", "mat.", "geo.", "pct.", "fin.", "unit.", "signal.", "prob.", "num.", "bit.", "base.", "int.", "dec.", "alg.", "cplx.", "reg.", "test.", "phys.", "eng.", "disc.", "chem.", "net.", "color.", "info.", "astro.", "time.", "geod.", "thermo.", "mech.", "fluid.", "elec.", "optics.", "wave.", "data."]
    trust_families = ["verify.", "frame.", "curve.", "predicate."]
    deep_families = ["linalg.", "special.", "optimize.", "ode.", "series."]
    for family in base_families + trust_families + deep_families:
        if not any(op.startswith(family) for op in ops):
            fail(f"missing v1.0.0 operation family: {family}")

    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if 'version = "1.0.0"' not in cargo:
        fail("Cargo.toml version is not 1.0.0")
    if 'schemars = { version = "=1.2.2", features = ["derive"] }' not in cargo:
        fail("schemars exact direct dependency pin missing")
    toolchain=(ROOT/'rust-toolchain.toml').read_text(encoding='utf-8')
    if 'channel = "1.98.0"' not in toolchain:
        fail('Rust toolchain is not pinned to 1.98.0')

    if 'EXACT_LOOKUP: OnceLock<HashMap<&\'static str, usize>>' not in registry:
        fail("exact case-preserving opcode/alias lookup is missing")
    if 'OnceLock<HashMap<String, usize>>' not in registry:
        fail("case-insensitive fallback opcode/alias lookup is missing")
    if 'FAMILY_INDEX: OnceLock<HashMap<String, Vec<usize>>>' not in registry:
        fail("family discovery index is missing")
    if 'if let Some(index) = exact_lookup().get(raw)' not in registry:
        fail("exact-first resolve guard is missing")
    if 'let preferred_index = exact_lookup().get(query.trim()).copied().or_else(|| lookup().get(&q).copied());' not in registry:
        fail("exact-first search/resolve precedence guard is missing")
    for family in ["disc.", "chem.", "net.", "color.", "info.", "astro.", "time.", "geod.", "thermo.", "mech.", "fluid.", "elec.", "optics.", "wave.", "data.", "verify.", "frame.", "curve.", "predicate.", "linalg.", "special.", "optimize.", "ode.", "series."]:
        generic = [line for line in registry.splitlines() if f'op("{family}' in line and '&["args..."]' in line]
        if generic:
            fail(f"compact specs still use generic args for {family}: {generic[:3]}")

    engine = (ROOT / 'src' / 'engine.rs').read_text(encoding='utf-8')
    if 'fn dispatch_module' not in engine or "split_once('.')" not in engine:
        fail('prefix dispatcher is missing')
    for branch in ['"verify" => verification::execute', '"frame" => frame::execute', '"curve" => curve::execute', '"predicate" => predicate::execute', '"linalg" => deep_linalg::execute', '"special" => special_functions::execute', '"optimize" => optimization::execute', '"ode" => ode::execute', '"series" => series::execute']:
        if branch not in engine:
            fail(f"missing v1.0.0 dispatcher branch: {branch}")

    print(f"PASS: {len(ops)} unique registered built-in/control opcodes")
    print(f"PASS: registry source = {registry_path}")
    print("PASS: MCP tool surface is exactly yk.compute / yk.find / yk.spec")
    print("PASS: alpha.10 MCP model + tool annotation schema sources are byte/semantic frozen")
    print("PASS: every registered opcode has an implementation/control reference")
    print("PASS: forbidden host execution/network patterns not found")
    print("PASS: v1.0.0 trust + deep numerical families present")
    print("PASS: indexed lookup, deterministic alias precedence, family index, prefix dispatcher, and compact specs present")
    print("PASS: version metadata, exact dependency pins, and Rust 1.98.0 toolchain are correct")


if __name__ == "__main__":
    main()
