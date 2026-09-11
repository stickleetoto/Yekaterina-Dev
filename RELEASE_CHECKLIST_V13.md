# Yekaterina v1.3.0 Release Checklist

## Frozen release contract

The v1.3.0 release candidate is defined by the following invariants:

- `Cargo.toml` package version: **1.3.0**.
- `Cargo.lock` root `yekaterina` package version: **1.3.0**.
- Live built-in/control operation surface: **1,425**.
- Transformer-native operation surface: **15 `xfmr.*` operations**.
- MCP tools: exactly **3** — `yk.compute`, `yk.find`, `yk.spec`.
- Advertised MCP initialize version: **1.0.0**, intentionally compatibility-gated.
- Golden regression corpus: **527/527**.
- Frozen v1.2 Full Capability Audit baseline: **1,410/1,410**.
- Default workers: **1**.
- The compatibility-frozen v1.2 source/audit artifacts checked by `scripts/static_audit_v13.py` must remain byte-identical.

No new compute feature is a v1.3.0 release blocker. Any additional calculator family, transformer model, standards/compliance layer, or unrelated optimization belongs to a later development line.

## Automated acceptance

The authoritative gate is `.github/workflows/ci.yml` on the exact finalization commit. It must complete successfully without skipping or weakening any of these checks:

```text
python scripts/static_audit_v13.py
python scripts/lexical_rust_audit.py
python scripts/operation_manifest.py
python scripts/validate_golden_manifest.py
python scripts/validate_full_audit.py
cargo test --locked --all-targets
cargo clippy --locked --all-targets
cargo build --locked --release
python tools/demo.py ./target/release/yekaterina
python scripts/verify_v13_transformer_runtime.py ./target/release/yekaterina
python scripts/verify_v12_operations.py
python scripts/verify_statistics.py
python scripts/verify_multiplicity.py
python golden/run_golden.py --exe ./target/release/yekaterina --out ./golden_results/ci
python scripts/check_golden_result.py ./golden_results/ci/result.json
python full_audit/run_full_audit.py --exe ./target/release/yekaterina --out ./full_audit_results/ci --strict
python bench/run_bench.py --exe ./target/release/yekaterina --out ./bench_results/ci --runs 2 --cold-reps 3 --label "ci"
```

Expected release-contract results include:

```text
static_audit_v13.py       19 pass / 0 fail
aggregate operations      1425 total / 15 xfmr
MCP tools                 3
golden                    527/527
frozen v1.2 full audit    1410/1410
crate package             1.3.0
Cargo.lock root package   1.3.0
```

The transformer runtime verifier must independently confirm discovery, spec access, and execution for all 15 promoted `xfmr.*` operations against the built release executable.

## Promotion rule

1. Run the complete CI gate on the exact release-finalization head.
2. Do **not** merge or promote if any required job fails, is cancelled, or is missing.
3. If the candidate changes after a green run, rerun the complete gate on the new exact head.
4. Once green, merge the finalization branch into `Yekaterina-Dev/main` without adding feature changes.
5. Treat that merged source state as the v1.3.0 promotion candidate for `stickleetoto/Yekaterina`.
6. Tag/release and stable-repository promotion should point to the verified source state; do not rebuild the release from an unverified feature commit.

## Release hygiene

- Keep `Cargo.lock` committed.
- Preserve `src/registry.rs` and `src/engine.rs` as the frozen v1.2 historical baseline.
- Preserve the three-tool MCP request schema and compatibility-gated initialize identity.
- Do not reinterpret transformer candidate scoring as standards certification or factory approval.
- Keep generated CI verification artifacts for the release record when available.
