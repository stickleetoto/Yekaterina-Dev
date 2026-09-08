# Yekaterina Electronics / PCB Engineering Pack — v0.2 (repo-aligned)

Status: **design specification, not implemented.** No code, tests, dependencies
or gates were changed to produce this document.

Supersedes `YEKATERINA_PCB_ENGINEERING_MATH_PACK_v0.1.md`. That draft was written
without reference to the registry; roughly two thirds of its P0 tier already
exists, and four of its cross-cutting sections conflict with invariants this
repository gate-verifies. This revision keeps the engineering content and drops
everything the engine already provides or structurally cannot accept.

Every "already exists" and "absent" claim below was checked by searching
`src/registry.rs` directly, at registry size **1,410**. Line numbers are cited
where a signature matters.

---

## 1. What changed from v0.1, and why

| v0.1 | v0.2 | Reason |
|---|---|---|
| new `ee.*` family | **dropped**; extend `elec.*` | 19 opcodes across `elec.*`/`eng.*` already cover Ohm, power, dividers, τ, reactance, series/parallel. `ee.*` would be a *third* name for each. |
| new `thermal.*` family | **dropped**; extend `thermo.*` | `thermal.` vs existing `thermo.` is a near-collision that degrades `yk.find` ranking. |
| new `pcb.*` family | **kept** | `grep -c 'op("pcb\.'` = 0. Nothing in the registry covers PCB geometry. |
| 3-level names (`ee.ohm.v`) | **2-level only** (`pcb.trace_width_ipc2221`) | All 1,410 existing opcodes are exactly two levels. `registry::search` uses the first segment as its family hint. |
| per-result `accuracy`/`model`/`warnings` | **`yk.spec` only** | Contradicts the pack's own token-efficiency goal, and duplicates how `capability_code`/`cost_code` already work. |
| `compact` / `verbose` flag | **dropped** | Needs a `ComputeParams` field; `src/model.rs` is byte-identical-frozen to v1.0.0 and hash-pinned. Compact is already the default. |
| `ERR:*` codes | **mapped to the existing 30** | "No new error code" is gate-verified on the v1.1 and v1.2 lines. |
| `f(I=2A, copper=1oz)` kwargs with units | **SI positional args** | `ComputeParams.a` is a positional `Vec<Value>`. Unit conversion reuses `unit.*` and the two copper ops below. |
| Newton / Brent solvers | **fixed-bracket bisection only** | Precedent: `fin.irr`. Newton makes the result depend on the starting guess and the floating-point path, breaking byte-identical output. |

---

## 2. Duplicate elimination — what was removed and what replaces it

### 2.1 Already implemented (do not add)

| v0.1 operation | Existing | Evidence |
|---|---|---|
| `ee.ohm.v` / `.i` / `.r` | `elec.voltage` / `elec.current` / `elec.resistance` | registry.rs:1126-1128 — *and* `eng.ohm_voltage`/`ohm_current`/`ohm_resistance` |
| `ee.power` (VI, I²R, V²/R) | `elec.power_vi` / `power_i2r` / `power_v2r` | registry.rs:1129-1131 — *and* `eng.power_vi`/`power_ir`/`power_vr` |
| `ee.resistor.series` / `.parallel` | `eng.resistance_series` / `eng.resistance_parallel` (take `args...`, N elements) | registry.rs:421-422 |
| two-resistor form | `elec.series_two` / `elec.parallel_two` | registry.rs:1155-1156 |
| `ee.divider` | `elec.voltage_divider(vin, r1, r2)` | registry.rs:1157 — *and* `eng.voltage_divider` |
| `ee.rc.tau` / `ee.rl.tau` | `elec.rc_tau` / `elec.rl_tau` | registry.rs:1139-1140 — *and* `eng.rc_tau`/`eng.rl_tau` |
| `ee.lc.f0` | `elec.resonant_frequency(l, c)` | registry.rs:1145 |
| `ee.reactance.capacitive` / `.inductive` | `elec.xc(f, c)` / `elec.xl(f, l)` | registry.rs:1141-1142 — *and* `eng.reactance_*` |
| `ee.mosfet.conduction_loss` (I²R) | `elec.power_i2r` | registry.rs:1130 — identical computation |
| `pcb.trace.velocity` (c/√εeff) | `optics.refractive_speed(n)` = `C/n` | optics.rs:40 — call with `n = sqrt(er_eff)` |

### 2.2 Sufficient as a composition of existing ops (do not add)

Rule applied: **a new opcode is justified only when at least one holds** —
(a) it is not expressible in two or fewer existing ops without the caller
supplying a non-obvious constant or formula shape; (b) it encodes a published
empirical or approximate model that cannot be derived from primitives; (c) it
needs a deterministic solver; (d) the composition exists but the *choice* among
near-identical formulas is a documented error source.

| v0.1 operation | Composition | Note |
|---|---|---|
| `pcb.trace.resistance` | `math.mul(w, t)` → `elec.resistance_from_resistivity(rho, len, area)` | registry.rs:1148. Two steps, no hidden constant except ρ, which the caller supplies anyway. |
| `pcb.trace.vdrop` | `elec.voltage(i, r)` | Ohm's law under another name |
| `pcb.trace.loss` | `elec.power_i2r(i, r)` | |
| `pcb.trace.delay` | `optics.refractive_speed(sqrt(er_eff))` → `net.propagation_delay(length, v)` | optics.rs:40, networking.rs:28 |
| `pcb.diffpair.skew` | same pair, called with ΔL | |
| `pcb.trace.ac_impedance` (R + j2πfL) | `elec.xl` + `elec.impedance_rl` | registry.rs:1142-1143 |
| `pcb.pdn.target_impedance` | `math.div(dv, di)`; ripple form `math.mul` then `math.div` | ΔV/ΔI carries no domain knowledge beyond its own name |
| `ee.buck.duty` (Vout/Vin) | `math.div` | see §9 open questions |
| `ee.ldo.efficiency` (Vout/Vin) | `math.div` | see §9 |
| `ee.buck.esr_ripple` (ESR·ΔI) | `math.mul` | |
| `ee.mosfet.total_loss` | `stat.sum([...])` | registry.rs:64 |
| N-resistor parallel from a list | `stat.hmean(rs)` → `math.div(hmean, n)` | registry.rs:80; `Rp = hmean/n` |

**Removed from v0.1: 22 named functions.** Fourteen already exist; eight are
compositions carrying no formula knowledge.

---

## 3. New operations — complete list

**32 new opcodes.** `pcb.*` 14 (new family), `elec.*` 15 (extension),
`thermo.*` 3 (extension). Every name and alias below returned 0 matches against
`src/registry.rs`.

Conventions for the whole table:

- **All lengths metres, all times seconds, all SI base units.** `unit.length`
  (registry.rs, `unit.*` family) converts mil/inch/mm; `pcb.copper_thickness`
  converts copper weight. No unit strings are parsed.
- Layer selector is a numeric code: `0` = outer, `1` = inner.
- Accuracy grade is metadata for `yk.spec` only (§6), never in a result.

### 3.1 `pcb.*` — new family (14)

| # | opcode | positional args (SI) | returns | formula | grade | source | oracle | round-trip |
|---|---|---|---|---|---|---|---|---|
| 1 | `pcb.copper_thickness` | `oz` | m | t = m_oz / (A_ft² · ρ_Cu) | analytical | derived; ρ per §7.3 | exact rational derivation | with #2 |
| 2 | `pcb.copper_weight` | `thickness_m` | oz | inverse of #1 | analytical | as #1 | exact rational derivation | with #1 |
| 3 | `pcb.trace_width_ipc2221` | `current_a, delta_t_c, thickness_m, layer_code` | m | A = (I/(k·ΔT^0.44))^(1/0.725); w = A/t | empirical | IPC-2221B §6.2 | IPC chart vectors + round-trip | with #4, #5 |
| 4 | `pcb.trace_current_ipc2221` | `width_m, thickness_m, delta_t_c, layer_code` | A | I = k·ΔT^0.44·A^0.725 | empirical | IPC-2221B §6.2 | IPC chart vectors + round-trip | with #3 |
| 5 | `pcb.trace_temp_rise_ipc2221` | `current_a, width_m, thickness_m, layer_code` | K | ΔT = (I/(k·A^0.725))^(1/0.44) | empirical | IPC-2221B §6.2 | algebraic inverse of #4 | with #4 |
| 6 | `pcb.via_inductance` | `height_m, drill_d_m` | H | L = (h/5)·[1 + ln(4h/d)] nH, h,d in mm | approximation | Johnson & Graham, *High-Speed Digital Design*, App. C | published worked example | — |
| 7 | `pcb.via_capacitance` | `height_m, pad_d_m, clearance_d_m, er` | F | C = 0.0555·εr·h·d₁/(d₂−d₁) pF, mm | approximation | as #6 | published worked example; C→∞ as d₂→d₁ guarded | — |
| 8 | `pcb.trace_inductance` | `length_m, width_m, thickness_m` | H | see §7.2 — **form unresolved** | approximation | **must be pinned before implementation** | reference vector required | — |
| 9 | `pcb.microstrip_eeff` | `er, height_m, width_m` | — | §7.1 | approximation | Hammerstad 1975; IPC-2141A | 1 < εeff < εr; → εr as w/h → ∞ | — |
| 10 | `pcb.microstrip_z0` | `er, height_m, width_m, thickness_m` | Ω | §7.1 piecewise | approximation | Hammerstad 1975; IPC-2141A | branch continuity at w/h = 1; monotone in w; published tables | with #11 |
| 11 | `pcb.microstrip_width` | `target_z_ohm, er, height_m, thickness_m` | m | bisection on #10 | approximation | — | round-trip against #10 | with #10 |
| 12 | `pcb.diffpair_z` | `z0_ohm, spacing_m, height_m` | Ω | Zd = 2·Z0·(1 − 0.48·e^(−0.96·s/h)) | approximation | IPC-2141A | Zd → 2·Z0 as s/h → ∞; monotone in s | with #13 |
| 13 | `pcb.diffpair_spacing` | `target_zdiff_ohm, z0_ohm, height_m` | m | bisection on #12 | approximation | — | round-trip against #12 | with #12 |
| 14 | `pcb.skin_depth` | `resistivity_ohm_m, permeability_h_per_m, frequency_hz` | m | δ = √(ρ/(π·f·μ)) | analytical | standard | independent numpy evaluation; copper at 1 MHz ≈ 66 µm | — |

### 3.2 `elec.*` — extension (15)

| # | opcode | positional args (SI) | returns | formula | grade | oracle | round-trip |
|---|---|---|---|---|---|---|---|
| 15 | `elec.resistance_at_temp` | `r0_ohm, alpha_per_k, t_c, t0_c` | Ω | R = R₀[1 + α(T−T₀)] | analytical | independent evaluation; R(T₀) = R₀ | self-inverse at T = T₀ |
| 16 | `elec.adc_lsb` | `vfs_v, bits` | V | V_LSB = V_FS / 2^N | analytical | `elec.adc_lsb · 2^N = vfs` | — |
| 17 | `elec.adc_code` | `vin_v, vfs_v, bits` | code | ⌊V_in/V_FS·(2^N − 1)⌉ | analytical | — | with #18 |
| 18 | `elec.adc_voltage` | `code, vfs_v, bits` | V | V = code/(2^N − 1)·V_FS | analytical | — | with #17 |
| 19 | `elec.opamp_inverting_gain` | `rf_ohm, rin_ohm` | — | A = −R_f/R_in | analytical | — | with #21 |
| 20 | `elec.opamp_noninverting_gain` | `rf_ohm, rg_ohm` | — | A = 1 + R_f/R_g | analytical | — | with #22 |
| 21 | `elec.opamp_rf_inverting` | `gain, rin_ohm` | Ω | R_f = −A·R_in | analytical | — | with #19 |
| 22 | `elec.opamp_rf_noninverting` | `gain, rg_ohm` | Ω | R_f = (A−1)·R_g | analytical | — | with #20 |
| 23 | `elec.buck_ripple_current` | `vin_v, vout_v, inductance_h, fs_hz` | A | ΔI = V_o(V_in−V_o)/(V_in·L·f_s) | analytical | independent evaluation | with #24 |
| 24 | `elec.buck_inductor` | `vin_v, vout_v, fs_hz, ripple_a` | H | L = V_o(V_in−V_o)/(V_in·f_s·ΔI) | analytical | — | with #23 |
| 25 | `elec.buck_cout_min` | `ripple_a, fs_hz, dvout_v` | F | C = ΔI/(8·f_s·ΔV) | approximation | independent evaluation | — |
| 26 | `elec.ldo_loss` | `vin_v, vout_v, iout_a, ignd_a` | W | P = (V_in−V_o)I_o + V_in·I_gnd | analytical | independent evaluation; reduces to (V_in−V_o)I_o at I_gnd = 0 | — |
| 27 | `elec.mosfet_switching_loss` | `v_v, i_a, tr_s, tf_s, fs_hz` | W | P = ½·V·I·(t_r+t_f)·f_s | approximation | independent evaluation | — |
| 28 | `elec.mosfet_coss_loss` | `coss_f, v_v, fs_hz` | W | P = ½·C_oss·V²·f_s | approximation | independent evaluation | — |
| 29 | `elec.mosfet_gate_loss` | `vdrive_v, qg_c, fs_hz` | W | P = V·Q_g·f_s | analytical | independent evaluation | — |

### 3.3 `thermo.*` — extension (3)

`thermo.thermal_resistance` (registry.rs:1015) is `thickness/(k·A)`, a *conduction*
resistance — a different quantity from θJA. Verified at thermodynamics.rs:22.

| # | opcode | positional args (SI) | returns | formula | grade | oracle | round-trip |
|---|---|---|---|---|---|---|---|
| 30 | `thermo.junction_temp` | `ambient_c, theta_ja_k_per_w, power_w` | °C | T_j = T_a + θ·P | estimate | — | with #31, #32 |
| 31 | `thermo.max_power_theta` | `tj_max_c, ambient_c, theta_ja_k_per_w` | W | P = (T_j,max−T_a)/θ | estimate | — | with #30 |
| 32 | `thermo.required_theta` | `tj_max_c, ambient_c, power_w` | K/W | θ = (T_j,max−T_a)/P | estimate | — | with #30 |

The three are exact algebraic inverses of one another, which is their oracle: any
two determine the third, and the composition must return the input.

---

## 4. Deterministic solver convention

Applies to `pcb.microstrip_width` (#11) and `pcb.diffpair_spacing` (#13).

- **Bisection only.** No Newton, no secant, no Brent. Rationale is already
  recorded for `fin.irr` in `DECISIONS.md`: an iterative method whose path
  depends on a starting guess makes the result depend on the floating-point
  path, and this engine's contract is byte-identical output.
- **Fixed bracket, stated in the operation's own documentation.**
  `microstrip_width`: w ∈ [1e-6 m, 100·h]. `diffpair_spacing`: s ∈ [1e-6 m, 50·h].
- **Fixed iteration count, independent of the input.** No convergence-based
  early exit — an early exit makes the iteration count data-dependent, which is
  the thing that must not vary. 80 iterations of bisection on those brackets
  reaches the f64 floor.
- Both objective functions are monotone over their bracket (Z0 falls as w rises;
  Zdiff rises as s rises), so a bracket that straddles the target is unique.
- If the bracket does not straddle the target, return `NO_CONVERGE`. The code
  already exists and `fin.irr` already uses it for exactly this case.
- Same input must give the same bits on every platform. This is testable and must
  be tested: the acceptance check is that the result is bit-identical across
  worker counts 1/2/4/8, which the existing fingerprint gate already enforces for
  every operation.

---

## 5. Error code mapping

No new codes. The engine's set is closed at 30 and gate-verified.

| v0.1 proposal | Existing code | When |
|---|---|---|
| `ERR:R_ZERO` | `DIV0` | any zero denominator |
| `ERR:I_NEGATIVE` | `DOMAIN` | negative current, width, thickness, frequency, ΔT |
| `ERR:NON_PHYSICAL_INPUT` | `DOMAIN` | εr < 1, negative geometry, T_j,max ≤ T_a |
| `ERR:INVALID_UNIT` | `UNIT` | reserved; this pack takes SI numbers, so it should not arise |
| `ERR:UNSUPPORTED_MODEL` | `OP` | unknown opcode, e.g. an IPC-2152 name before it exists |
| — | `ARG` | wrong argument count, bad layer code |
| — | `TYPE` | non-numeric argument |
| — | `SHAPE` | `d₂ ≤ d₁` in `via_capacitance` |
| — | `NO_CONVERGE` | solver bracket does not straddle the target |
| — | `NONFINITE` | overflow to inf/NaN |
| — | `LIMIT` | argument beyond the model's stated validity range |

Return format is unchanged: `{"r": <value>}` or `{"e": "<CODE>"}`.

---

## 6. Accuracy grade exposure

The grade (`exact` / `analytical` / `empirical` / `approximation` / `estimate`)
is real and worth carrying — but not in every result.

Proposal: a static `registry::accuracy_code(opcode) -> &'static str` alongside the
existing `capability_code` and `cost_code`, surfaced as one extra field in
`yk.spec` next to the current `"c"` and `"k"`. One letter: `x`, `a`, `e`, `p`,
`s`.

Why this and not a result field:

- `tools/list` never enumerates operations, so the grade costs the model nothing
  until it asks — the same reason 1,410 operations cost 412 schema tokens.
- `yk.spec` output is not part of the frozen schema, so adding a field there does
  not touch `src/model.rs`.
- It is one lookup, not a per-call string allocation on the hot path.

`verify_required` is dropped as a separate flag: `empirical`, `approximation` and
`estimate` already say it, and a boolean that is a function of another field is a
second source of truth.

**Open item:** adding a field to `yk.spec` changes what the model sees at the
point it asks. That is a smaller change than the `initialize` block, which is
hash-pinned, but it is still observable output and should be a deliberate release
decision rather than a side effect of this pack.

---

## 7. The three formulas v0.1 got wrong or left unsourced

### 7.1 Microstrip impedance — v0.1 shipped one branch of a two-branch model

v0.1 §20 gives only `Z0 ≈ (60/√εeff)·ln(8h/w + w/(4h))`. That is the Hammerstad
**narrow-trace** branch, valid for w/h ≤ 1. A 50 Ω trace on ordinary FR-4 stack-ups
usually lands at w/h > 1, so as written the model is wrong in its main use case.

The full Hammerstad model, which v0.2 specifies:

```
eeff = (er+1)/2 + (er-1)/2 * (1 + 12h/w)^(-1/2)      + 0.04*(1 - w/h)^2   if w/h < 1
eeff = (er+1)/2 + (er-1)/2 * (1 + 12h/w)^(-1/2)                           if w/h >= 1

w/h <= 1 :  Z0 = (60 / sqrt(eeff)) * ln(8h/w + w/(4h))
w/h >= 1 :  Z0 = (120*pi / sqrt(eeff)) / (w/h + 1.393 + 0.667*ln(w/h + 1.444))
```

Source: E. Hammerstad, "Equations for Microstrip Circuit Design", *Proc. 5th
European Microwave Conference*, 1975; reproduced in IPC-2141A.

**Oracle, in order of strength:**

1. **Branch continuity at w/h = 1** — both expressions are defined there and must
   agree to within Hammerstad's stated accuracy. This is a pure identity, needs no
   external reference, and would catch a transcription error in either branch.
2. **εeff bounds** — 1 < εeff < εr for all inputs, and εeff → εr as w/h → ∞.
3. **Monotonicity** — Z0 strictly decreasing in w for fixed h, εr.
4. **Round-trip** against `pcb.microstrip_width`.
5. Published reference tables (IPC-2141A worked examples).

Thickness correction: the `t` argument is accepted and applied as the standard
effective-width correction. If the correction is not implemented in the first
cut, `t` must still be validated and the omission documented — silently ignoring
an argument is worse than rejecting it.

### 7.2 Trace inductance — v0.1's form is unsourced and differs from the common one

v0.1 §18: `L ≈ 2l·[ln(2l/w) − 0.5 + 0.2235(w/l)]`.

The widely reproduced rectangular-conductor form (Rosa 1908, via Johnson &
Graham) is:

```
L[nH] = 2*l * ( ln(2l/(w+t)) + 0.5 + 0.2235*(w+t)/l )     l, w, t in cm
```

Two differences: the sign of the 0.5 term, and whether conductor thickness enters
as `w+t`. These are not cosmetic — at l = 2.54 cm the sign alone moves the result
by about 5 nH, which is the same order as the value itself for short traces.

**This operation is blocked until the source is pinned.** Acceptance requires one
published worked example with numeric inputs and output; the two candidate forms
differ enough that a single reference vector decides it. Do not implement from
either form on the strength of it looking familiar.

### 7.3 Copper weight — 35 µm is 2.8 % high

v0.1 §50 uses 1 oz ≈ 35 µm. Derived from first principles, with every conversion
exact by definition (1 lb = 453.59237 g, 1 in = 2.54 cm):

```
m_oz   = 453.59237 / 16          = 28.349523125 g      (exact)
A_ft²  = 30.48^2                 = 929.0304 cm^2        (exact)
t      = m_oz / (A_ft² * rho_Cu)
```

| ρ_Cu | t (1 oz) | in mil |
|---|---|---|
| 8.96 g/cm³ (CRC, 20 °C) | **34.057 µm** | 1.3408 |
| 8.93 g/cm³ | 34.172 µm | 1.3453 |
| 8.89 g/cm³ (IACS annealed) | **34.325 µm** | 1.3514 |

Commonly quoted industry nominals: 1.37 mil = 34.798 µm; IPC-4562 lists 1 oz
nominal foil at ≈ 34.3 µm, which matches the IACS-annealed density almost
exactly.

v0.1's 35 µm is **2.77 % above** the density-exact value at ρ = 8.96. Trace width
from IPC-2221 scales as A/t, so that error passes straight into every width.

**Recommendation:** implement `pcb.copper_thickness` from the exact rational
derivation with **ρ = 8.89 g/cm³ (IACS annealed)**, giving 34.325 µm for 1 oz and
agreeing with the IPC-4562 nominal. Put the density in one named constant with the
source in a comment, and state in the operation summary that fabricator data
overrides it. Do not hard-code 35, and do not hard-code a per-weight lookup table
— `pcb.copper_weight` must be its exact inverse, which a table cannot guarantee.

Note for implementation: v1.2 found three unit constants that were one ULP wrong,
caught by clippy's `excessive_precision` lint after the round-trip tests missed
them. Write the constant to full f64 precision or as an explicit division.

---

## 8. Priorities

### P0 — the pack's reason to exist (11 ops)

Everything here is either absent from the registry or encodes a standard the
model would otherwise have to recall.

```
pcb.copper_thickness
pcb.copper_weight
pcb.trace_width_ipc2221
pcb.trace_current_ipc2221
pcb.trace_temp_rise_ipc2221
elec.resistance_at_temp
thermo.junction_temp
thermo.max_power_theta
thermo.required_theta
pcb.microstrip_eeff
pcb.microstrip_z0
```

### P1 — controlled-impedance and via work (6 ops)

```
pcb.microstrip_width        (first deterministic solver)
pcb.diffpair_z
pcb.diffpair_spacing        (second solver)
pcb.via_inductance
pcb.via_capacitance
pcb.trace_inductance        (blocked on §7.2)
```

### P2 — power and mixed-signal (15 ops)

```
elec.buck_ripple_current   elec.buck_inductor      elec.buck_cout_min
elec.ldo_loss              elec.mosfet_switching_loss
elec.mosfet_coss_loss      elec.mosfet_gate_loss
elec.adc_lsb               elec.adc_code           elec.adc_voltage
elec.opamp_inverting_gain  elec.opamp_noninverting_gain
elec.opamp_rf_inverting    elec.opamp_rf_noninverting
pcb.skin_depth
```

**Total: 32 new opcodes.** Registry would go 1,410 → 1,442.

Deferred to a later revision, all confirmed absent but none load-bearing for a
first pack: stripline and coplanar Z0, via current and resistance, thermal via
arrays, trace fusing current, creepage and clearance, PDN capacitor impedance and
resonance, crystal load capacitance, I²C pull-up sizing, USB VBUS drop.

---

## 9. Open questions for a decision before implementation

1. **The eight compositions in §2.2 that carry a formula name.** `buck_duty`
   (Vout/Vin), `ldo_efficiency` (Vout/Vin), `pdn_target_impedance` (ΔV/ΔI) and
   `esr_ripple` (ESR·ΔI) are each one arithmetic op, so §2.2 excludes them. But
   the registry already accepts one-multiplication named operations for exactly
   the opposite reason — `elec.power_vi` is `math.mul` with a name that carries
   the physics. The line drawn here is that a *ratio of two quantities the caller
   already named* teaches the model nothing, whereas `power_vi` disambiguates
   which of three power formulas applies. That is a judgement call; say so if you
   want them in.

2. **`yk.spec` gaining an accuracy field** (§6) changes observable output. Worth
   doing, but it is a release decision.

3. **`pcb.trace_inductance` stays blocked** until §7.2 is settled with a cited
   worked example.

4. **IPC-2152.** v0.1 correctly notes IPC-2221 ignores stack-up, copper pour and
   airflow. The `_ipc2221` name suffix exists so a 2152-based operation can be
   added later without renaming or silently changing an existing result.

---

## 10. Verification plan

Every new operation needs an oracle before it is written; the table in §3 names
one per operation. Consolidated by kind:

| Kind | Operations | Notes |
|---|---|---|
| Exact rational derivation | copper_thickness, copper_weight | computed in §7.3, reproducible |
| Algebraic round-trip | the three IPC-2221 ops; adc_code↔adc_voltage; the four opamp ops; buck_ripple↔buck_inductor; the three thermo ops; microstrip_z0↔width; diffpair_z↔spacing | strongest available for closed-form inverses |
| Structural identity | microstrip branch continuity at w/h = 1; εeff bounds; Zdiff → 2·Z0 as s/h → ∞; monotonicity of Z0 in w | needs no external reference |
| Independent implementation | skin_depth, buck_*, ldo_loss, mosfet_*, resistance_at_temp | numpy evaluation of the same closed form by a different expression path |
| Published reference vector | IPC-2221 chart points; Hammerstad/IPC-2141A tables; Johnson & Graham via examples | **must be transcribed with the citation next to each vector** |

The gap v0.1 did not address: for `empirical` and `approximation` grades there is
no scipy-equivalent library to check against, so the reference-vector row above is
the only external oracle and it has to be sourced by hand. That work is a
prerequisite, not a follow-up — this repository's standing rule is that an
operation is verified against a value produced by a different route than the
implementation, and "I typed the same formula into Python" does not satisfy it.

---

## 11. Invariants this pack does not touch

Stated explicitly so a reviewer can check them off:

- `src/model.rs` unchanged — no new `ComputeParams` field, no flags.
- 3 MCP tools, 412 tokens / 1,725 bytes of schema, unchanged.
- `initialize` still advertises `1.0.0`; the hash-pinned block is not edited.
- 30 error codes, none added.
- Result envelope `{"r":…}` / `{"e":…}` unchanged.
- Every new operation routes through `engine::execute`, which takes no `&self`,
  so all 32 are `Pure` by type and need no change to `src/safety.rs`.
- New opcodes go in via `scripts/register_ops.py`, which refuses to write on an
  opcode or alias collision, and updates the manifest, fixtures and every count.
- `full_audit/fixtures_v12.json` gets one curated fixture per new operation.
- `SOURCE_INTEGRITY_V12.txt` regenerated; `scripts/mutate_gates.py` re-run.

## 12. Status of unrelated in-flight work

This document changed nothing in the working tree beyond adding itself. The
statistics expansion is mid-flight and untouched: batch 1 (multiplicity,
post-hoc, effect sizes — 23 operations, registry at 1,410) is complete and
verified; the GLM and time-series batches are not started. Both are independent
of this pack — different families, different modules, no shared symbols.
