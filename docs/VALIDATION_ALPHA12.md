# Validation — alpha.12

## Static gates

- exactly 1,215 unique registered opcodes
- exactly 3 MCP tools
- frozen MCP request schema source and tool annotations
- implementation reference for every registered opcode
- no forbidden shell/network/unsafe host patterns
- all five alpha.12 deep families present
- 85/85 curated deep-op Full Audit fixtures
- >= 527 golden cases / >= 44 categories

## Local Rust gate

Run `VERIFY_WINDOWS.bat` and require:

- `cargo test --locked --all-targets`
- `cargo clippy --locked --all-targets`
- `cargo build --locked --release`
- real release-binary MCP golden result 527/527

## Full capability gate

Run `RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat` after the local gate.

Target: every one of the 1,215 registered opcodes must receive a valid fixture, execute successfully through real `yk.compute`, match its `yk.spec` return-type contract, and succeed again in a clean replay. The audit also reruns all 527 Golden cases as a separate oracle.

This all-opcode audit proves execution/type coverage. The Golden/property suites provide correctness evidence; neither claim proves every possible input for every operation.

## Protocol regression gate

After acceptance, rerun the MCP Real-World Benchmark. Required protocol invariant: tool count remains 3 and schema tokens/bytes do not increase from the verified 412 / 1,725 baseline.
