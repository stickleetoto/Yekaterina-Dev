# Yekaterina v1.2 — operation expansion

v1.1 froze the operation set at 1,215 deliberately: it was a performance and
concurrency release, and adding operations at the same time would have made any
regression ambiguous. That work is finished and frozen, so v1.2 does the thing
v1.1 was not allowed to do.

**1,215 → 1,318. One hundred and three operations, seven families, no
other change.**

| | |
|---|---|
| MCP tools | **3** — unchanged |
| Schema footprint | **412 tokens / 1,725 bytes** — unchanged |
| `src/model.rs` | byte-identical to the frozen v1.0.0 schema surface |
| Advertised MCP version | still `1.0.0` |
| Error codes | **30** — no new code |
| Golden | **527/527**, expectations untouched |

The schema footprint is unchanged because `tools/list` never enumerated
operations; only `yk.find` and `yk.spec` see them, and only when asked.

## What was added

| family | before | after | added | what it is for |
|---|---:|---:|---:|---|
| `int` | 8 | **26** | 18 | exact arbitrary-precision integers |
| `geo` | 8 | **30** | 22 | solids and plane figures |
| `fin` | 8 | **29** | 21 | money over time |
| `dec` | 4 | **20** | 16 | exact decimals |
| `vec` | 9 | **22** | 13 | vector geometry |
| `unit` | 13 | **20** | 7 | unit conversion |
| `pct` | 3 | **9** | 6 | percentages |

These were the thinnest families against how often they come up: `stat` had 60
operations and `prob` 55, while finance had eight and exact decimals had four.

**`fin`** — `npv`, `irr`, `annuity_pv`, `annuity_fv`, `perpetuity_pv`,
`loan_balance`, `loan_total_interest`, `amort_interest`, `amort_principal`,
`depreciation_straight`, `depreciation_declining`, `depreciation_syd`,
`break_even_units`, `margin`, `markup`, `effective_rate`, `nominal_rate`,
`rule72`, `payback_period`, `bond_price`, `real_rate`.

**`geo`** — solids (`cylinder`, `cone`, `cube`, `box`, `pyramid`, `torus`,
each volume and area), plane figures (`ellipse_area`, `ellipse_perimeter`,
`trapezoid_area`, `parallelogram_area`, `regular_polygon_area`,
`regular_polygon_perimeter`), circle parts (`circle_sector_area`,
`circle_segment_area`), and coordinate geometry (`midpoint3d`,
`point_line_distance`, `triangle_area_points`).

**`vec`** — `normalize`, `angle`, `reject`, `reflect`, `lerp`, `manhattan`,
`chebyshev`, `minkowski`, `hadamard`, `negate`, `abs`, `triple3`, `rotate2d`.

**`unit`** — `force`, `torque`, `density`, `flow`, `acceleration`, `charge`,
`illuminance`.

**`pct`** — `increase`, `decrease`, `ratio`, `reverse`, `point_change`, `apply`.

**`int`** — `abs`, `neg`, `sign`, `cmp`, `divmod`, `div_floor`, `mod_floor`,
`divmod_floor`, `sqrt`, `shl`, `shr`, `bit_length`, `min`, `max`, `sum`,
`product`, `mod_pow`, `mod_inverse`.

**`dec`** — `mod`, `abs`, `neg`, `cmp`, `round`, `round_even`, `floor`, `ceil`,
`trunc`, `scale`, `min`, `max`, `sum`, `product`, `pow`, `to_number`.

## Why exact arithmetic got the largest share

`int` and `dec` are where a compute engine earns its place. A model asked to add
`0.1 + 0.2` will say `0.3`; a float64 engine answers `0.30000000000000004`. Both
are wrong for an invoice. `dec.sum` answers `0.3`, and `dec.round("2.675", 2)`
answers `2.68` where anything float-backed answers `2.67`, because 2.675 is
exactly representable as a decimal and is not as a float.

`dec` previously had add, subtract, multiply and divide and nothing else — no
rounding, no comparison, no aggregation. It could not be used for money at all.
It now has two rounding modes, because the choice between them is a real
accounting decision rather than a default anyone should inherit silently:
`dec.round` takes ties away from zero, `dec.round_even` uses banker's rounding,
and a test asserts they agree everywhere except on ties.

`dec.sqrt` was drafted and dropped. A square root is irrational, and this
family's contract is that every result is exact; `math.sqrt` already covers the
approximate case. `dec.pow` refuses a negative exponent for the same reason.

**`alg.*` already had `mod_pow`, `mod_inverse`, `ext_gcd`, `floor_div`,
`factorial` and `is_prime`, and they are not duplicated — they are extended.**
Reading their implementations showed every one is capped at `u64`, `i64` or
`u128` and returns `LIMIT` or `TYPE` above that. The `int.*` versions have no
ceiling: a test does modular exponentiation with a 128-bit modulus that
`alg.mod_pow` cannot even accept. `alg.factorial` and `alg.combination_exact`
return exact big integers but take bounded inputs, so they were left alone.

## Nine candidates were dropped

112 operations were drafted and 103 shipped. Eight were removed after checking
the whole registry rather than just the target families, and one on principle:

| dropped | already in the engine |
|---|---|
| `pct.error` | `num.percent_error` |
| `geo.sphere_volume` | `geod.sphere_volume` — identical formula |
| `geo.sphere_area` | `geod.sphere_area` — identical formula |
| `geo.circle_arc` | `geod.arc_length` — same product, arguments swapped |
| `vec.project` | `linalg.project` |
| `vec.mean` / `vec.min` / `vec.max` | `stat.mean` / `stat.min` / `stat.max` — same signature over `number[]` |
| `dec.sqrt` | dropped on principle, not duplication: irrational results do not belong in an exact family |

A second name for an existing operation makes `yk.find` worse, not better: it
splits the results for one concept across two entries and gives the model a
choice that does not matter. `geo.polygon_area` and friends were never drafted
for the same reason — `curve.*` already covers polygon area, perimeter, centroid
and arc length from a point list.

Where a name is reused across families it is because the *type* differs, not the
concept: `vec.lerp` interpolates vectors where `math.lerp` interpolates scalars,
and `vec.hadamard` matches `mat.hadamard` one dimension down.

## Correctness

Each operation is checked three ways, and the checks are written from the
definition rather than from the implementation.

**Independent recomputation** — `scripts/verify_v12_operations.py` drives the
real binary over MCP and compares against a value computed by a different route.
Where the engine uses a closed form, the check iterates instead: `loan_balance`
is verified by amortising month by month, `annuity_pv` by summing the discounted
payments, `bond_price` by summing coupons one at a time. For the exact families
the oracle is Python's own arbitrary-precision `int` and `decimal.Decimal` — a
genuinely separate implementation, not a restatement of the Rust. 164
assertions across 99 operations.

Exact results are compared as numbers, not as strings, so a formatting choice
cannot make a wrong value look right; a separate assertion checks that no exact
result ever comes back in exponent notation, since a caller re-reading `1E+2`
as an integer would be surprised.

**Identities that a wrong formula would break** — these catch errors that a
hand-picked expected value can hide:

* `minkowski` at p=1 must equal `manhattan` and at p=2 must equal `distance`;
* `sector_area` over a full turn must equal `circle_area`, and `segment_area`
  over a half turn must equal half the disc;
* an ellipse with equal axes must reproduce both circle formulas;
* `cube_volume(l)` must equal `box_volume(l,l,l)`, and `cone_volume` must equal
  `pyramid_volume` over the same circular base;
* `triangle_area_points` on a 3-4-5 triangle must equal Heron's formula on it;
* reflection must preserve length and be its own inverse;
* `reject` must be orthogonal to the reference vector;
* four quarter turns of `rotate2d` must return the original vector;
* `effective_rate` and `nominal_rate` must round-trip;
* both integer division conventions must satisfy `quotient * divisor +
  remainder == dividend`, with the truncated remainder taking the sign of the
  dividend and the floor remainder the sign of the divisor;
* `int.sqrt(n)` must satisfy `root^2 <= n < (root+1)^2`;
* `int.shl` then `int.shr` by the same amount must return the original, and a
  left shift must equal multiplication by that power of two;
* `a * int.mod_inverse(a, m) mod m` must be 1;
* `dec.pow` must equal repeated `dec.mul`;
* `dec.floor` and `dec.ceil` must bracket the value within a width of one, with
  `dec.trunc` inside that bracket;
* `amort_interest + amort_principal` must equal the scheduled payment, and the
  balance after the final payment must be zero;
* `irr` is checked against its own definition: `npv` at the returned rate is zero.

**Guards** — 20 error cases assert the exact code: a polygon with two sides or a
fractional side count, a torus whose tube is thicker than its hole, a sector past
a full turn, a degenerate line, a zero-length vector to normalize, Minkowski with
p below 1, salvage above cost, a period past the loan term, and `irr` on cash
flows with no sign change, which returns `NO_CONVERGE` rather than inventing a
root at whichever bracket end is closer to zero.

42 Rust unit tests carry the same checks so `cargo test` runs them.

## Determinism

Two of the new operations have a size ceiling by construction rather than by
accident. `int.shl` is bounded by the digits it can produce rather than by the
shift count, and `dec.pow` checks the digit count each iteration, so neither can
be asked to allocate its way out of the result budget.

`fin.irr` is the only new operation that iterates toward an answer. It uses bisection over a fixed
bracket with a fixed iteration count, so it is a pure function of its input on
every platform. Newton's method would converge faster and would make the result
depend on the starting guess and the floating-point path; that is not a trade
worth making in an engine whose contract is byte-identical output.

Every new operation is routed through `engine::execute`, which takes no `&self`.
By the argument in `docs/V11_SAFETY_MODEL.md` they are therefore `Pure` **by
type** — they cannot reach server state, and they parallelise safely with no
change to `src/safety.rs`. New operations are safe by construction here, not by
a classification someone remembered to update.

One consequence worth knowing: `scheduler::estimated_cost` reads cost by family
prefix, so the new operations inherit their family's cost class. All five are
cheap families, so none of them is a parallel candidate. A wrong cost estimate
can only pick a worse schedule, never a different result.

## What proves the old operations survived

Adding to a 1,215-entry table risks quietly dropping or reordering something.
`full_audit/opcodes_v11_frozen.json` records the exact v1.1 list, and
`scripts/validate_full_audit.py` proves against it that all 1,215 are still
registered, still spelled the same, and still in the same relative order. The
generator that inserted the new entries also refuses to run if a new opcode or
alias collides with an existing one — that check is what found two of the eight
duplicates above.

Fixtures live in `full_audit/fixtures_v12.json`, deliberately **not** in
`overrides_alpha12.json`, whose own gate requires it to hold exactly the 85
deep-family operations. The validator now gates the new file just as strictly:
it must cover exactly the new operations, be disjoint from the alpha.12 set, and
name nothing unregistered.

Every new operation carries an explicit fixture rather than relying on the
runner's 192-candidate inference sweep, so discovery is a lookup instead of a
search, and the fixture doubles as the documented example.

## Audits

`scripts/static_audit.py` (v1.0.0) and `scripts/static_audit_v11.py` (v1.1) are
untouched and hash-pinned by `scripts/static_audit_v12.py`. Neither can pass on
this tree, and that is correct: each gates the operation count of its own release
line. They remain runnable against their own frozen trees.
