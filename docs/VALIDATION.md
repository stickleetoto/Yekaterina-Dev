# Validation — v0.1.0-alpha.10 candidate

## Completed in the artifact environment

- static registry audit: PASS
- registered built-in/control opcodes: **1054 unique**
- MCP tool surface: exactly 3 (`yk.compute`, `yk.find`, `yk.spec`)
- all 1054 registered opcodes have implementation/control references
- all nine alpha.10 domain families are present
- exact opcode/alias index present
- family discovery index present
- prefix compute dispatcher present
- alpha.10 compact specs use named arguments rather than generic `args...`
- golden manifest: **391 cases / 35 categories**, every referenced opcode registered
- no shell/process execution, TCP/UDP sockets, `reqwest`, or `unsafe {` found in source scan
- Python verification/golden scripts compile successfully

## Mandatory local Rust gate

The artifact environment does not contain Rust. On Windows run:

```powershell
.\VERIFY_WINDOWS.bat
```

Acceptance target:

```text
1054 operations
3 MCP tools
391/391 golden cases
35/35 categories at 100%
release build PASS
```

## Mandatory post-build protocol gate

Rerun `Yekaterina_MCP_RealWorld_Benchmark_v0.1` with the alpha.10 release binary.

Regression budget:

```text
tools                  = 3
schema tokens          <= 600
schema reduction       >= 95%
10k accuracy           = 100%
10k wire reduction     >= 78%
recovery               PASS
UDO persistence        PASS
```

Do not update performance claims until the fresh alpha.10 benchmark result is recorded.
