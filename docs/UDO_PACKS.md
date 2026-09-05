# Yekaterina UDO & Pack Contract — alpha.3

## Namespaces

```text
user.<name>          locally authored UDO
pack.<pack>.<name>   imported pack UDO
```

Built-in namespaces such as `math.*`, `stat.*`, `int.*`, `dec.*`, `udo.*`, and `expr.*` cannot be overwritten by UDO definitions.

## Formula UDO

```json
{"op":"user.energy","p":["m","v"],"expr":"0.5*m*v^2"}
```

Formula grammar is intentionally small and host-isolated.

## Composite UDO

```json
{
  "op":"user.quad",
  "p":["x"],
  "pipe":[
    ["user.double","$a0"],
    ["user.double","$0"]
  ]
}
```

Reference rules:

```text
$aN = Nth call argument
$N  = Nth prior Composite result
```

Forward result references are rejected at definition time.

## Dependency rules

A locally defined Composite may depend on:

- built-in compute operations
- already installed `user.*` operations
- already installed `pack.*` operations

It may not depend on:

- unknown operations
- `udo.*` control operations
- itself directly
- a graph that creates an indirect Composite cycle

## Pack export

Only `user.*` operations may be exported.

Export transforms names into an isolated pack namespace and rewrites internal UDO references. Any selected Composite dependency on an unselected `user.*` operation makes export fail with `PACK`, ensuring the result is self-contained.

## Pack import

Imported operations must all live below exactly:

```text
pack.<declared-pack-name>.*
```

Composite dependencies must resolve to either:

- built-ins, or
- operations included in the same pack

Conflicting installed names are rejected with `DUP`.

## Referential integrity

Removing an operation used by another Composite is rejected with `IN_USE`.

Uninstalling a pack used by an external `user.*` Composite is also rejected with `IN_USE`.

## Persistence

Any mutation operation is treated transactionally at the in-memory registry level:

```text
clone before state
-> mutate/validate
-> persist full snapshot
-> success: keep mutation
-> persistence failure: restore before state
```
