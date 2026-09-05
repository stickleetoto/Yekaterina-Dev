# NEXT — alpha.12 acceptance

1. Run `VERIFY_WINDOWS.bat` on Windows.
2. Fix any Rust compile/test/clippy issue without weakening numerical contracts.
3. Require 527/527 real MCP golden cases.
4. Run `RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat`.
5. Use its exact missing-op list to harden fixtures/runtime until 1,215/1,215 clean replay.
6. Rerun MCP Real-World Benchmark and confirm the 3-tool / 412-token / 1,725-byte schema baseline is unchanged.
7. Freeze alpha.12 before adding any new family.
