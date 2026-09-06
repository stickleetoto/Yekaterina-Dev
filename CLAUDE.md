# CLAUDE.md

Yekaterina — token-efficient compute engine for LLMs, served over MCP.
Rust crate `yekaterina`, edition 2024, toolchain pinned to 1.98.0.

## Repository navigation policy

Use `docs-first-project-navigation` as the default for ordinary tasks.

- **Do not begin with a full repository scan.** `src/registry.rs` alone is 156 KB
  of declaration table; reading it whole is almost never the right move.
- Read the compact docs first, in this order:
  `REPO_MAP.md` → `ARCHITECTURE.md` → `CURRENT_STATE.md` → `DECISIONS.md` →
  `KNOWN_ISSUES.md`, then the relevant file under `docs/`.
- Treat these documents as a **navigation index, not truth**. Use them to name
  the module, file and symbol; verify actual behaviour against the code.
- Prefer symbol/opcode search over directory reading. An opcode string
  (`"stat.mean"`, `"test.p_t"`) is the fastest entry point this repo has: grep it
  in `src/registry.rs` for the declaration, then in the owning module for the
  implementation.
- Follow dependency edges (imports, dispatch arms, call sites), not adjacency.
- Do not expand exploration "for completeness" or "to be safe". Expand only on
  concrete new evidence.
- Do not reread unchanged files.
- Run targeted verification first (see below), broader suites only when needed.
- Stop once the requested behaviour is implemented and its verification passes.

Core rule:

> Docs first. Code as verification. Expand only with evidence.

## Excluded from analysis by default

`target/`, `bench_results/`, `golden_results/`, `full_audit_results/`,
`benchmark_results/`, `__pycache__/`, `_scratch/` (one level up), and every
`*_Source.zip` / release-bundle directory in the parent folder. These are build
output, captured measurement artifacts, and archived snapshots.

## Verification, cheapest first

```bash
cargo test --locked --all-targets
python scripts/static_audit_v12.py
python scripts/validate_full_audit.py
```

The full gate set is `.github/workflows/ci.yml`. Golden, full-audit and bench
runs need a release binary and are slow; use them only when the change could
affect operation results or the MCP surface.

## Hard constraints on any change

- The MCP surface is frozen: exactly 3 tools (`yk.compute`, `yk.find`,
  `yk.spec`), 412 tokens / 1,725 bytes of schema. `src/model.rs` must stay
  byte-identical to the v1.0.0 record.
- `#[tool_handler(...)]` in `src/server.rs` — server name, advertised version
  `1.0.0`, and the instructions string — is hash-pinned. Changing it is a
  deliberate release decision.
- Adding a stateful operation requires a `safety::ControlOp` variant *and* a
  `server.rs` dispatch arm. See `docs/V11_SAFETY_MODEL.md`.
- Thread creation is allowed only in `src/pool.rs`; the audit fails the build
  otherwise.
- Editing any file tracked by `SOURCE_INTEGRITY_V12.txt` requires regenerating
  it (`python scripts/gen_source_integrity_v12.py`) or the audit fails.
