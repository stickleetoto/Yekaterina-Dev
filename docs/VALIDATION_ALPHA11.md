# Validation — alpha.11

## Static gates

- >= 1130 unique opcodes
- exactly three MCP tools
- alpha.10 `model.rs` schema source hash unchanged
- normalized `#[tool]` annotations unchanged
- no shell/network/unsafe host paths introduced
- all registered opcodes have implementation references
- all four trust families have named compact argument specs

## Rust gates

```powershell
cargo fmt --all
cargo test --all-targets
cargo clippy --all-targets
cargo build --release
```

## Real MCP golden gate

```powershell
python .\golden\run_golden.py --exe .\target\release\yekaterina.exe --out .\golden_results\latest
python .\scripts\check_golden_result.py .\golden_results\latest\result.json
```

Required: 453/453, 39/39 categories, 3-tool surface.

## Protocol regression gate

After the above passes, run the external MCP Real-World Benchmark. The alpha.10-hotfix2 reference is 3 tools / 412 schema tokens. Alpha.11 intentionally changed neither model schema source nor tool annotations, so any schema-token drift must be investigated rather than accepted as expected growth.
