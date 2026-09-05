# Yekaterina architecture — v0.1-alpha.6

```text
LLM
 |
 | MCP: exactly 3 tools
 v
+--------------------------------+
| yk.compute / yk.find / yk.spec |
+----------------+---------------+
                 |
          Unified Dispatch
                 |
      +----------+-----------+
      |          |           |
      v          v           v
  Built-ins   user.*      pack.*
      |          |           |
      |      +---+---+   +---+---+
      |      |       |   |       |
      |   Formula Composite Formula Composite
      |      |       |   |       |
      +------+-------+---+-------+
                 |
       Dependency/Cycle Guard
                 |
      +----------+-----------+
      |          |           |
  Family Ops   BigInt    BigDecimal
      |          |           |
      +----------+-----------+
                 |
        Batch / Pipeline / UDO
                 |
          Compact Result Gate
                 |
                 v
                LLM
```

## Invariants

1. Internal operation count never determines MCP tool count.
2. UDOs are data, not dynamically exposed MCP tools.
3. Formula/Expression execution never exposes arbitrary host code.
4. Composite UDOs may call compute operations but not `udo.*` control operations.
5. Direct and indirect Composite cycles are rejected before persistence.
6. Runtime depth is limited even if a malformed snapshot bypasses validation.
7. Persistent state is only published after a completed generation snapshot.
8. Removal preserves referential integrity (`IN_USE`).
9. Imported packs use isolated `pack.<name>.*` namespaces.
10. Result-size limits are enforced separately from request-size limits.

## Persistence

```text
mutation
  |
  v
clone previous in-memory registry
  |
  v
validate mutation
  |
  v
serialize complete registry
  |
  v
write unique .tmp file
  |
flush + fsync
  |
  v
rename to unique generation snapshot
  |
  v
keep latest 3 generations
```

If persistence fails, the in-memory mutation is rolled back.

Startup scans newest generation first and falls back to the newest valid older snapshot.

## UDO types

### Formula

```text
user.energy(m,v) = 0.5*m*v^2
```

Stored as a compact expression plus ordered parameters.

### Composite

```text
user.mean_x10(values)
  stat.mean($a0) -> $0
  math.mul($0,10) -> final
```

A Composite is effectively a reusable internal pipeline. It is a major token-saving primitive because repeated multi-call sequences collapse into one stable opcode.

### WASM

Reserved for a later sandbox milestone. Not executable in alpha.3.

## Pack model

A pack is a self-contained serializable group of Formula/Composite operations.

```text
user.* local graph
   |
 export(name)
   v
pack.<name>.* graph
```

Export rewrites dynamic inter-operation references. Import rejects dependencies outside built-ins + the pack itself.

## Capability codes

`yk.spec` returns compact source/capability/cost hints so the LLM does not need verbose descriptions for known operations.


## Alpha.6 family dispatch

`engine::execute` resolves a canonical opcode once, then routes it through bounded family handlers before the small scalar fast-path match. Alpha.6 adds matrix, probability, numerical, and radix handlers while preserving the same MCP surface.

```text
registry.resolve
   |
   +-- precision (int/dec)
   +-- math/stat/vector/matrix/geometry
   +-- practical/signal/probability/numerical/radix
   +-- scalar fast path
```

Operation-family growth therefore changes the internal registry and implementation modules, not `tools/list`.
