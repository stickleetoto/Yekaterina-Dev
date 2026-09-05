# Architecture — alpha.11 Verification & Numerical Trust

```text
LLM / agent
   |
   | MCP (unchanged)
   v
+------------------+
| yk.compute       |
| yk.find          |   exactly 3 tools
| yk.spec          |
+------------------+
   |
   v
indexed registry -> prefix dispatcher
   |
   +-- existing 1054 operations
   +-- verify.*
   +-- frame.*
   +-- curve.*
   +-- predicate.*
```

Verification is a compute capability, not a new protocol surface. This is why alpha.11 adds 76 opcodes but zero MCP parameter fields.

## Trust composition

Existing batch/pipeline primitives are the orchestration layer. A caller can compute a result, feed intermediate values into verification operations, and return only the final compact diagnostic without adding another MCP tool.

## Cost boundaries

- verification sequences: <= 100,000 scalar values
- curve/polygon inputs: <= 2,048 points
- frame names: <= 128 bytes
- server-wide existing request/result node and byte limits remain in force

No operation gains shell, filesystem or network execution authority.
