"""Yekaterina v1.1 static audit.

Relationship to ``scripts/static_audit.py``:

* The v1.0.0 audit is preserved unmodified as a frozen historical artifact and
  is NOT edited by v1.1. This script asserts that fact by hash.
* Every substantive v1.0.0 gate is reproduced here at equal or greater strength.
  The only intentional relaxation is the package version string, which is
  parameterised through EXPECTED_VERSION instead of being hard-coded to 1.0.0 --
  it remains a hard equality check, not a removed check.
* v1.1-specific gates are added for the concurrency work: unsafe code, thread
  spawn containment, fail-closed operation-safety classification, worker-count
  default, and benchmark/determinism infrastructure.

Gates that are staged (they activate once the module they govern exists) print
PENDING rather than PASS, and PENDING is reported in the summary so an
unfinished phase can never be mistaken for a satisfied gate.
"""
from __future__ import annotations

import hashlib
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# --------------------------------------------------------------- frozen facts

EXPECTED_VERSION = "1.1.0"
ACCEPTED_VERSIONS = {"1.0.0", "1.1.0"}  # 1.0.0 tolerated before the Phase 2 bump
EXPECTED_OPS = 1215
EXPECTED_TOOLS = ["yk.compute", "yk.find", "yk.spec"]
EXPECTED_TOOLCHAIN = "1.98.0"

# Frozen v1.0.0 MCP surface. Identical values to scripts/static_audit.py.
FROZEN_MODEL_SHA256 = "08a0222b5c646f8afc57932b8fb8d56fc36870fad25da014be06bca6180f6faf"
FROZEN_TOOL_ANNOTATION_SHA256 = "a000b4f460a1273846926031d2d5c831c5c5c9d04be3ef82068a46fd5ded64bc"
# The `#[tool_handler(...)]` block: the server name, advertised version and
# instructions string returned in the MCP `initialize` response. v1.0.0 had no
# gate on this, so the instructions the LLM reads could have been edited without
# any check noticing. Pinned here. The advertised version is deliberately NOT
# tied to the crate version: bumping Cargo.toml to 1.1.0 must not change a byte
# of what a client observes on the wire.
FROZEN_SERVER_IDENTITY_SHA256 = "864823192ea2e5f11baa634d39dea9d094c21dc0fbf950d6425944253c81f160"
ADVERTISED_VERSION = "1.0.0"
# The v1.0.0 audit itself, pinned so "preserved as a frozen artifact" is verifiable.
FROZEN_V10_AUDIT_SHA256 = "16360daed1e226855e41f0f845eec0dc0e1fa20a5afc5c1049867a747311ce8e"

EXPECTED_FIELDS = {
    "ComputeParams": ["op", "a", "ops", "pipe", "input", "all"],
    "FindParams": ["q", "l"],
    "SpecParams": ["op"],
}

FORBIDDEN_PATTERNS = [
    "std::process::Command",
    "TcpStream",
    "UdpSocket",
    "reqwest::",
    "unsafe {",
]

# Modules permitted to create OS threads. Everything else spawning threads is a
# gate failure: ad-hoc threading is exactly how ordering and determinism bugs
# enter a codebase that currently has none.
THREAD_ALLOWLIST = {"pool.rs"}
THREAD_PATTERNS = ["thread::spawn", "thread::Builder", "std::thread::spawn"]

BASE_FAMILIES = ["math.", "stat.", "vec.", "mat.", "geo.", "pct.", "fin.", "unit.", "signal.",
                 "prob.", "num.", "bit.", "base.", "int.", "dec.", "alg.", "cplx.", "reg.",
                 "test.", "phys.", "eng.", "disc.", "chem.", "net.", "color.", "info.",
                 "astro.", "time.", "geod.", "thermo.", "mech.", "fluid.", "elec.", "optics.",
                 "wave.", "data."]
TRUST_FAMILIES = ["verify.", "frame.", "curve.", "predicate."]
DEEP_FAMILIES = ["linalg.", "special.", "optimize.", "ode.", "series."]

_failures: list[str] = []
_pending: list[str] = []
_passes: list[str] = []


def fail(msg: str) -> None:
    _failures.append(msg)


def ok(msg: str) -> None:
    _passes.append(msg)


def pending(msg: str) -> None:
    _pending.append(msg)


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def struct_fields(text: str, name: str) -> list[str]:
    m = re.search(rf"pub struct {re.escape(name)}\s*\{{(.*?)\n\}}", text, re.S)
    if not m:
        fail(f"missing model struct {name}")
        return []
    return re.findall(r"^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:", m.group(1), re.M)


# ------------------------------------------------------------------ v1.0 gates

def check_v10_audit_preserved() -> None:
    p = ROOT / "scripts" / "static_audit.py"
    if not p.exists():
        fail("scripts/static_audit.py was deleted; the v1.0.0 audit must be preserved")
        return
    got = sha256_text(p.read_text(encoding="utf-8"))
    if got != FROZEN_V10_AUDIT_SHA256:
        fail(f"scripts/static_audit.py was modified (sha256 {got}); "
             "the v1.0.0 audit must remain a frozen historical artifact")
    else:
        ok("v1.0.0 static audit preserved unmodified (hash verified)")


def check_registry(registry: str, all_src: str) -> list[str]:
    ops = re.findall(r'^\s*op\("([^"]+)"', registry, re.M)
    dups = [o for o, n in Counter(ops).items() if n > 1]
    if dups:
        fail(f"duplicate opcodes: {dups}")
    if len(ops) != EXPECTED_OPS:
        fail(f"v1.1 requires exactly {EXPECTED_OPS} registered opcodes, got {len(ops)}")
    else:
        ok(f"{len(ops)} unique registered opcodes (frozen v1.0.0 count)")

    missing = [o for o in ops if f'"{o}"' not in all_src]
    if missing:
        fail(f"registered opcodes missing implementation/control reference: {missing[:8]}")
    else:
        ok("every registered opcode has an implementation/control reference")

    for needed in ["udo.composite", "udo.export", "udo.import", "udo.uninstall", "expr.eval"]:
        if needed not in ops:
            fail(f"missing required UDO/control opcode: {needed}")

    for fam in BASE_FAMILIES + TRUST_FAMILIES + DEEP_FAMILIES:
        if not any(o.startswith(fam) for o in ops):
            fail(f"missing operation family: {fam}")

    for fam in [f for f in BASE_FAMILIES + TRUST_FAMILIES + DEEP_FAMILIES
                if f not in ("math.", "stat.", "vec.", "mat.", "geo.", "pct.", "fin.",
                             "unit.", "signal.", "prob.", "num.", "bit.", "base.",
                             "int.", "dec.", "alg.", "cplx.", "reg.", "test.",
                             "phys.", "eng.")]:
        generic = [ln for ln in registry.splitlines()
                   if f'op("{fam}' in ln and '&["args..."]' in ln]
        if generic:
            fail(f"compact specs still use generic args for {fam}: {generic[:2]}")

    # Deterministic registry structure (identical assertions to v1.0.0).
    structural = [
        ("EXACT_LOOKUP: OnceLock<HashMap<&'static str, usize>>",
         "exact case-preserving opcode/alias lookup"),
        ("OnceLock<HashMap<String, usize>>", "case-insensitive fallback lookup"),
        ("FAMILY_INDEX: OnceLock<HashMap<String, Vec<usize>>>", "family discovery index"),
        ("if let Some(index) = exact_lookup().get(raw)", "exact-first resolve guard"),
        ("let preferred_index = exact_lookup().get(query.trim()).copied()"
         ".or_else(|| lookup().get(&q).copied());", "exact-first search precedence guard"),
    ]
    for needle, label in structural:
        if needle not in registry:
            fail(f"deterministic registry check failed: {label} is missing")
    if not _failures:
        ok("indexed lookup, alias precedence, family index and compact specs present")
    return ops


def check_tool_surface(server: str, model: str) -> None:
    tools = re.findall(r'name\s*=\s*"(yk\.[^"]+)"', server)
    if tools != EXPECTED_TOOLS:
        fail(f"unexpected MCP tool surface: {tools}")
    else:
        ok(f"MCP tool surface is exactly {EXPECTED_TOOLS}")

    for name, expected in EXPECTED_FIELDS.items():
        got = struct_fields(model, name)
        if got != expected:
            fail(f"MCP schema field drift in {name}: {got} != {expected}")

    model_hash = sha256_text(model)
    if model_hash != FROZEN_MODEL_SHA256:
        fail(f"src/model.rs drifted from the frozen v1.0.0 surface: {model_hash}")
    else:
        ok("src/model.rs is byte-identical to the frozen v1.0.0 schema surface")

    chunks = re.findall(r'#\[tool\((.*?)\)\]\s*async fn (compute|find|spec)', server, re.S)
    norm = "|".join(re.sub(r"\s+", " ", c).strip() + ":" + fn for c, fn in chunks)
    tool_hash = sha256_text(norm)
    if tool_hash != FROZEN_TOOL_ANNOTATION_SHA256:
        fail(f"MCP tool annotation schema drifted: {tool_hash}")
    else:
        ok("MCP tool annotation schema is byte/semantic frozen")

    m = re.search(r"#\[tool_handler\((.*?)\)\]\s*impl ServerHandler", server, re.S)
    if not m:
        fail("the #[tool_handler(...)] server identity block is missing")
    else:
        identity = re.sub(r"\s+", " ", m.group(1)).strip()
        if sha256_text(identity) != FROZEN_SERVER_IDENTITY_SHA256:
            fail("MCP server identity (name/version/instructions returned by "
                 f"initialize) drifted: {sha256_text(identity)}")
        elif f'version = "{ADVERTISED_VERSION}"' not in identity:
            fail(f"advertised MCP version is not {ADVERTISED_VERSION!r}")
        else:
            ok("MCP server identity (name, advertised version, instructions) is frozen")


def check_dispatcher(engine: str) -> None:
    if "fn dispatch_module" not in engine or "split_once('.')" not in engine:
        fail("prefix dispatcher is missing")
    for branch in ['"verify" => verification::execute', '"frame" => frame::execute',
                   '"curve" => curve::execute', '"predicate" => predicate::execute',
                   '"linalg" => deep_linalg::execute', '"special" => special_functions::execute',
                   '"optimize" => optimization::execute', '"ode" => ode::execute',
                   '"series" => series::execute']:
        if branch not in engine:
            fail(f"missing dispatcher branch: {branch}")
    ok("prefix dispatcher and all v1.0.0 family branches present")


def check_manifest_and_pins() -> None:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.M)
    version = m.group(1) if m else None
    if version not in ACCEPTED_VERSIONS:
        fail(f"Cargo.toml version {version!r} is not an accepted v1.1 line version "
             f"{sorted(ACCEPTED_VERSIONS)}")
    elif version != EXPECTED_VERSION:
        pending(f"Cargo.toml version is still {version!r}; "
                f"bump to {EXPECTED_VERSION!r} before the v1.1 freeze")
    else:
        ok(f"Cargo.toml version is {version}")

    if 'schemars = { version = "=1.2.2", features = ["derive"] }' not in cargo:
        fail("schemars exact direct dependency pin missing")
    for pin in ['serde_json = "=1.0.151"', 'rmcp = { version = "=3.2.0"']:
        if pin not in cargo:
            fail(f"exact dependency pin missing or changed: {pin}")
    toolchain = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    if f'channel = "{EXPECTED_TOOLCHAIN}"' not in toolchain:
        fail(f"Rust toolchain is not pinned to {EXPECTED_TOOLCHAIN}")
    else:
        ok(f"exact dependency pins and Rust {EXPECTED_TOOLCHAIN} toolchain are correct")


def check_lexical(rust_files: list[Path]) -> None:
    bad = []
    for f in rust_files:
        text = f.read_text(encoding="utf-8")
        if r"\n#[test]" in text or r"\nfn " in text:
            bad.append(str(f.relative_to(ROOT)))
    if bad:
        fail(f"literal escaped newlines found in Rust source/test files: {bad}")
    else:
        ok("no literal escaped newlines in Rust sources")


# ------------------------------------------------------------- v1.1 new gates

def check_no_unsafe(rust_files: list[Path]) -> None:
    """Stricter than v1.0.0, which only checked ``unsafe {`` and skipped registry.rs."""
    hits = []
    for f in rust_files:
        for i, line in enumerate(f.read_text(encoding="utf-8").splitlines(), 1):
            stripped = line.split("//")[0]
            if re.search(r"\bunsafe\b", stripped):
                hits.append(f"{f.relative_to(ROOT)}:{i}")
    if hits:
        fail(f"unsafe Rust found (v1.1 forbids it crate-wide): {hits}")
    else:
        ok("no unsafe Rust anywhere in src/ or tests/ (stricter than v1.0.0)")


def check_forbidden(src_files: list[Path]) -> None:
    all_src = "\n".join(p.read_text(encoding="utf-8") for p in src_files)
    found = [n for n in FORBIDDEN_PATTERNS if n in all_src]
    if found:
        fail(f"forbidden host execution/network patterns: {found}")
    else:
        ok("forbidden host execution/network patterns not found "
           "(includes std::process::Command, so no process backend has leaked in)")


def check_thread_containment(src_files: list[Path]) -> None:
    offenders = []
    for f in src_files:
        if f.name in THREAD_ALLOWLIST:
            continue
        text = f.read_text(encoding="utf-8")
        for pat in THREAD_PATTERNS:
            if pat in text:
                offenders.append(f"{f.relative_to(ROOT)} contains {pat}")
    if offenders:
        fail(f"OS threads may only be created in {sorted(THREAD_ALLOWLIST)}: {offenders}")
    elif (ROOT / "src" / "pool.rs").exists():
        ok(f"OS thread creation is contained to {sorted(THREAD_ALLOWLIST)}")
    else:
        ok("no OS thread creation anywhere yet (worker pool not implemented)")


def check_safety_classification(registry: str, server: str) -> None:
    p = ROOT / "src" / "safety.rs"
    if not p.exists():
        pending("operation safety classification (src/safety.rs) not implemented yet "
                "[Phase 4]")
        return
    text = p.read_text(encoding="utf-8")
    if "Serialized" not in text:
        fail("src/safety.rs has no Serialized variant; fail-closed default is unverifiable")

    # Fail-closed: no catch-all may produce a parallel classification, and the
    # unregistered path must be serialized.
    for m in re.finditer(r"_\s*=>\s*([A-Za-z:]*Safety::)?(\w+)", text):
        if m.group(2) in {"Parallel", "Pure"}:
            fail(f"src/safety.rs catch-all arm yields {m.group(2)}; "
                 "unknown operations must default to serialized execution")
    if "return Safety::Serialized;" not in text:
        fail("src/safety.rs does not serialize unregistered opcodes; the fail-closed "
             "default must be an explicit early return")

    # Prefix matching is fine for presentation (registry::capability_code) but
    # can silently misclassify a future operation if used for scheduling. The
    # exhaustiveness test below is allowed to use it.
    body = text.split("mod tests", 1)[0]
    if re.search(r"starts_with\(\s*\"", body):
        fail("src/safety.rs uses a prefix heuristic outside its tests; prefix matching "
             "can silently misclassify future operations")
    if "FAIL_CLOSED" not in text and "fail-closed" not in text.lower():
        fail("src/safety.rs does not document its fail-closed contract")

    # The real guard: a new control operation cannot be registered without being
    # classified. Every udo.* opcode in the registry must appear as an arm of
    # `control_op` specifically.
    #
    # Checking the whole file is not enough: an opcode also appears in
    # `ControlOp::opcode()` and in the tests, so deleting only its `control_op`
    # arm -- which still compiles, and silently classifies it Pure -- would slip
    # past a substring search. That exact mutation was used to verify this gate.
    udo_ops = sorted(set(re.findall(r'^\s*op\("(udo\.[^"]+)"', registry, re.M)))
    m = re.search(r"pub fn control_op\s*\([^)]*\)[^{]*\{(.*?)\n\}", text, re.S)
    if not m:
        fail("src/safety.rs has no control_op function to audit")
        control_arms = ""
    else:
        control_arms = m.group(1)
    unclassified = [op for op in udo_ops if f'"{op}" =>' not in control_arms]
    if unclassified:
        fail(f"registered control opcodes with no control_op arm: {unclassified}. "
             "A new udo.* operation must be given a ControlOp variant, or it falls "
             "through to engine::execute and is silently classified Pure")

    # And the dispatcher must consult the classification rather than duplicating
    # it, so the two cannot drift apart.
    if "safety::control_op(" not in server:
        fail("src/server.rs does not dispatch through safety::control_op; the "
             "dispatcher and the safety classifier must be the same code")
    stale = [op for op in udo_ops if f'"{op}" =>' in server]
    if stale:
        fail(f"src/server.rs still matches control opcodes as string literals: {stale}")

    ok(f"safety classification is fail-closed, prefix-free, and covers all "
       f"{len(udo_ops)} registered control opcodes")
    ok("dispatcher dispatches through safety::control_op (classifier cannot drift)")


def check_worker_default() -> None:
    main = (ROOT / "src" / "main.rs").read_text(encoding="utf-8")
    if "--workers" not in main and "workers" not in main.lower():
        pending("worker-count configuration not implemented yet [Phase 5]")
        return
    m = re.search(r"DEFAULT_WORKERS\s*:\s*usize\s*=\s*(\d+)", main)
    if not m:
        fail("main.rs parses --workers but declares no DEFAULT_WORKERS constant")
    elif m.group(1) != "1":
        fail(f"DEFAULT_WORKERS is {m.group(1)}; v1.1 ships with a default of 1 until the "
             "1/2/4/8 benchmark, determinism stress and RSS checks are complete")
    else:
        ok("worker count is configurable and defaults to 1")


def check_bench_infrastructure() -> None:
    required = [
        ROOT / "bench" / "run_bench.py",
        ROOT / "bench" / "workloads.py",
        ROOT / "bench" / "bench_client.py",
    ]
    missing = [str(p.relative_to(ROOT)) for p in required if not p.exists()]
    if missing:
        fail(f"benchmark infrastructure missing: {missing}")
        return
    wl = (ROOT / "bench" / "workloads.py").read_text(encoding="utf-8")
    if "WORKLOAD_SET_VERSION" not in wl:
        fail("bench/workloads.py declares no WORKLOAD_SET_VERSION")
    rb = (ROOT / "bench" / "run_bench.py").read_text(encoding="utf-8")
    for needle, label in [("FROZEN_SCHEMA_TOKENS = 412", "frozen 412-token schema assertion"),
                          ("FROZEN_SCHEMA_BYTES = 1725", "frozen 1725-byte schema assertion"),
                          ("response_sha256", "per-workload response fingerprint")]:
        if needle not in rb:
            fail(f"bench/run_bench.py is missing the {label}")
    if (ROOT / "golden" / "mcp_client.py").exists():
        ok("benchmark infrastructure present, reuses the frozen golden MCP client")

    baseline = ROOT / "bench_results" / "v1.0.0-frozen" / "result.json"
    if not baseline.exists():
        pending("frozen v1.0.0 performance baseline not captured yet [Phase 1]")
    else:
        ok("frozen v1.0.0 performance baseline is present")


def check_test_corpus() -> None:
    """The existing test corpus must not shrink."""
    expected = {"engine.rs": 31, "formula.rs": 3, "golden_categories.rs": 36,
                "registry.rs": 12, "user_ops.rs": 7}
    for name, minimum in expected.items():
        p = ROOT / "tests" / name
        if not p.exists():
            fail(f"existing test file tests/{name} was removed")
            continue
        n = len(re.findall(r"^\s*fn ", p.read_text(encoding="utf-8"), re.M))
        if n < minimum:
            fail(f"tests/{name} shrank from {minimum} to {n} functions; "
                 "existing tests must not be weakened")
    if not any("test" in f for f in _failures):
        ok("existing v1.0.0 test corpus is intact or grown")



def check_source_integrity() -> None:
    """Verify SOURCE_INTEGRITY_V11.txt against the tree it claims to pin.

    An integrity record nobody checks is decoration. This makes it load-bearing:
    edit a tracked file without regenerating and the audit fails.
    """
    record = ROOT / "SOURCE_INTEGRITY_V11.txt"
    if not record.is_file():
        pending("SOURCE_INTEGRITY_V11.txt not written yet [Phase 11]")
        return
    entries = []
    for line in record.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^([0-9a-f]{64})\s\s(\S.*)$", line)
        if m:
            entries.append((m.group(1), m.group(2)))
    if not entries:
        fail("SOURCE_INTEGRITY_V11.txt lists no files")
        return
    drifted, absent = [], []
    for expected, rel in entries:
        f = ROOT / rel
        if not f.is_file():
            absent.append(rel)
        elif hashlib.sha256(f.read_bytes()).hexdigest() != expected:
            drifted.append(rel)
    if absent:
        fail(f"SOURCE_INTEGRITY_V11.txt pins files that do not exist: {absent}")
    elif drifted:
        fail(f"source integrity drift in {drifted}; "
             "regenerate with scripts/gen_source_integrity_v11.py and commit both")
    else:
        ok(f"SOURCE_INTEGRITY_V11.txt verifies {len(entries)} files against the tree")

    v10 = ROOT / "SOURCE_INTEGRITY.txt"
    if not v10.is_file():
        fail("the v1.0.0 source integrity record was deleted")
    elif not any(rel == "SOURCE_INTEGRITY.txt" for _, rel in entries):
        fail("SOURCE_INTEGRITY_V11.txt does not pin the v1.0.0 record")
    else:
        ok("the v1.0.0 source integrity record is preserved and pinned")


# ---------------------------------------------------------------------- main

def main() -> None:
    registry = (ROOT / "src" / "registry.rs").read_text(encoding="utf-8")
    server = (ROOT / "src" / "server.rs").read_text(encoding="utf-8")
    model = (ROOT / "src" / "model.rs").read_text(encoding="utf-8")
    engine = (ROOT / "src" / "engine.rs").read_text(encoding="utf-8")

    src_files = sorted((ROOT / "src").glob("*.rs"))
    rust_files = src_files + sorted((ROOT / "tests").glob("*.rs"))
    non_registry = [p for p in src_files if p.name != "registry.rs"]
    all_src = "\n".join(p.read_text(encoding="utf-8") for p in non_registry)

    check_v10_audit_preserved()
    check_registry(registry, all_src)
    check_tool_surface(server, model)
    check_dispatcher(engine)
    check_manifest_and_pins()
    check_lexical(rust_files)
    check_no_unsafe(rust_files)
    check_forbidden(non_registry)
    check_thread_containment(src_files)
    check_safety_classification(registry, server)
    check_worker_default()
    check_bench_infrastructure()
    check_test_corpus()
    check_source_integrity()

    print("=" * 62)
    print(" Yekaterina v1.1 static audit")
    print("=" * 62)
    for m in _passes:
        print(f"PASS: {m}")
    for m in _pending:
        print(f"PENDING: {m}")
    for m in _failures:
        print(f"FAIL: {m}")
    print("-" * 62)
    print(f"pass={len(_passes)} pending={len(_pending)} fail={len(_failures)}")
    if _failures:
        raise SystemExit(1)
    if _pending:
        print("NOTE: pending gates are unimplemented phases, not satisfied gates.")
    raise SystemExit(0)


if __name__ == "__main__":
    main()
