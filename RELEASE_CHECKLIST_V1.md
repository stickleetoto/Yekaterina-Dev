# Yekaterina v1.0.0 Release Checklist

## Local acceptance

```powershell
.\VERIFY_WINDOWS.bat
.\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat
```

Expected:

```text
registered opcodes          1215
MCP tools                   3
Golden                      527/527
spec coverage               1215/1215
fixture coverage            1215/1215
clean replay/type contract  1215/1215
Golden oracle               100%
```

## Self-regression

Run `Yekaterina Self-Regression Benchmark v0.1` against `target\release\yekaterina.exe` with opcode count `1215`.

Frozen V1 reference acceptance:

```text
Hard gate             PASS
Verdict               CURRENT WINS
Capability            1054 -> 1215 (+15.28%)
Schema tokens         412 -> 412
10k wire tokens       159794 -> 159794
10k arithmetic        100%
```

## Release hygiene

- Keep the generated `Cargo.lock` for the final Git repository/release snapshot.
- Do not add new opcodes or MCP fields to the V1 freeze branch.
- Re-run verification after any correctness/security/compatibility patch.
