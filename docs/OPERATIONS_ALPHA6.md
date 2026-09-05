# Yekaterina alpha.6 operation surface

Internal opcodes are intentionally **not** exposed as individual MCP tools. They are discovered lazily through `yk.find` and inspected with `yk.spec`.

## Family counts

| Family | Count |
|---|---:|
| math | 40 |
| stat | 31 |
| prob | 18 |
| mat | 16 |
| unit | 13 |
| signal | 12 |
| bit | 11 |
| num | 11 |
| vec | 9 |
| int | 8 |
| fin | 8 |
| geo | 8 |
| udo | 7 |
| base | 4 |
| dec | 4 |
| pct | 3 |
| expr | 1 |
| **Total** | **204** |

## Matrix examples

```json
{"op":"mat.mul","a":[[[1,2],[3,4]],[[5,6],[7,8]]]}
```

```json
{"op":"mat.inverse","a":[[[4,7],[2,6]]]}
```

## Probability examples

```json
{"op":"prob.binomial_pmf","a":[10,3,0.5]}
```

```json
{"op":"prob.normal_cdf","a":[1.96,0,1]}
```

## Numerical expression examples

All expression-based numerical solvers reuse Yekaterina's safe arithmetic expression parser; no host code execution is introduced.

```json
{"op":"num.bisect","a":[{"e":"x^2-2"},1,2]}
```

```json
{"op":"num.integrate","a":[{"e":"x^2"},0,3,2048]}
```

## Radix / bit examples

```json
{"op":"base.convert","a":["ff",16,2]}
```

```json
{"op":"bit.popcount","a":[255]}
```

## Limits

- matrix dimensions are bounded; expensive determinant/inverse paths are capped
- matrix multiply has an operation budget
- probability loops and numerical iterations are bounded
- radix strings and collection sizes are bounded
- expression solvers inherit the expression parser depth/length guards

These limits are part of the MCP resilience model, not optional performance hints.
