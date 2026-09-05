# alpha.11 Verification Contract

## Purpose

Alpha.11 does not try to replace domain judgment. It catches cheap, repeatable numerical and geometric failure modes before an LLM or engineer confidently accepts a bad result.

## Invariants

1. Exactly three MCP tools remain exposed.
2. No alpha.11 verification parameter is added to the MCP schemas.
3. Verification is deterministic and local; no network, shell, random sampling, or hidden external state.
4. Existing alpha.10 opcodes and default output shapes are unchanged.
5. Verification errors use existing compact error style (`ARG`, `TYPE`, `SHAPE`, `DOMAIN`, `FRAME`, `DEGENERATE`, `LIMIT`).
6. Expensive topology operations are bounded (`curve` input <= 2048 points).
7. `frame.*` uses right-handed active rotation matrices and transform direction `from -> to`.

## Convergence semantics

`verify.convergence(values, abs_tol?, rel_tol?)` treats the final two values as the acceptance pair. It reports:

- `ok`: final absolute change <= `max(abs_tol, rel_tol * max(|a|,|b|))`
- `v`: latest value
- `da`: final absolute change
- `dr`: symmetric relative change
- `p`: estimated order from the final three differences when defined, assuming a factor-2 sequence

`verify.grid_convergence(values, resolutions, abs_tol?, rel_tol?)` additionally requires strictly increasing positive resolutions and reports latest refinement ratio `r`. Order `p` is emitted only when the final refinement ratios agree within 5%.

This is a diagnostic, not a proof of convergence.

## Frame semantics

Tagged vector:

```json
{"f":"leg","t":"v","v":[1,2,3]}
```

Tagged point:

```json
{"f":"leg","t":"p","v":[1,2,3]}
```

Rigid transform:

```json
{"from":"leg","to":"world","r":[[...],[...],[...]],"p":[x,y,z]}
```

Vectors receive rotation only. Points receive rotation + translation. Arithmetic between different frame tags returns `FRAME`.

## Curve semantics

Alpha.11 curves are bounded 2D polylines/polygons, not arbitrary symbolic parametric functions. A repeated final point within tolerance is treated as closure. `curve.audit` is deliberately compact:

- `c`: closed
- `s`: simple closed
- `x`: nonadjacent self-intersection count
- `a`: absolute enclosed shoelace area
- `l`: input polyline length
- `ce`: closure error
- `bt`: immediate backtrack count

For non-simple polygons, area is a shoelace diagnostic and must not be interpreted as a unique physical enclosed region.

## Predicate semantics

Boundary points count as contained. Clearance values are signed:

- positive: inside with remaining margin
- zero: tangent/on boundary
- negative: containment failure/outside

`predicate.circle_polygon_clearance` is specifically intended for checks such as a circular hole staying within a plate outline.

## Non-goals

Alpha.11 does not determine whether a transmission-angle threshold is physically relevant, whether a tolerance distribution is realistic, or whether a mechanical model is adequate. Those remain domain-judgment problems.
