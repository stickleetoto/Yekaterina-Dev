# Yekaterina alpha.12 handoff

Candidate baseline:

- 1,215 registered built-in/control opcodes
- 3 MCP tools
- 527 MCP golden cases / 44 categories
- 85 alpha.12 curated Full Capability Audit fixtures
- five new deep families: `linalg`, `special`, `optimize`, `ode`, `series`
- alpha.10/11 MCP request schema source frozen

## Acceptance order

1. `VERIFY_WINDOWS.bat`
2. require 527/527 golden and release build PASS
3. `RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat`
4. drive any fixture/runtime misses to 1,215/1,215
5. rerun MCP Real-World Benchmark and require 412 schema tokens / 1,725 bytes remain unchanged
6. freeze alpha.12 only after all three gates pass

Do not claim full-opcode live coverage until the real Windows Full Capability Audit reaches 100%.
