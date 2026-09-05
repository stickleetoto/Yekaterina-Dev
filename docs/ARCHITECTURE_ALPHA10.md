# Alpha.10 scaling architecture

```text
MCP (3 tools)
   ↓
yk.compute / yk.find / yk.spec
   ↓
O(1) opcode/alias index
   ↓
canonical opcode
   ↓
prefix dispatcher ─────────────→ domain module

Discovery path:
query → optional family index → bounded fuzzy ranking → ≤20 opcode names
```

Alpha.10 intentionally separates three scaling concerns:

- **catalog size**: 1054 internal opcodes
- **MCP surface size**: fixed at 3 tools
- **dispatch cost**: indexed canonical lookup + direct family routing

Generic discovery can still scan the full catalog because it is an explicit, bounded operation. Family-qualified discovery narrows candidates first.
