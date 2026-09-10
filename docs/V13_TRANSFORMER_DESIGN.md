# Yekaterina v1.3 Transformer Design Pack

This is the first manufacturing-oriented transformer calculation layer for Yekaterina.
It is intentionally delivered as an importable UDO pack so the frozen v1.2 built-in
registry remains untouched while the equations are validated in real workflows.

## Scope

Target: 50/60 Hz sinusoidal power/distribution transformer preliminary design.
The pack is a calculation kernel, not a certification engine. It does not assert
IEC/IEEE/DOE compliance, dielectric safety, mechanical short-circuit withstand,
or a guaranteed thermal design.

The pack exposes 20 operations under `pack.xfmr.*` covering:

- effective core area and volts/turn
- primary/secondary turns and back-calculated flux density
- saturation margin
- magnetic reluctance and magnetizing inductance
- magnetizing current from a material B-H field-strength sample
- conductor area sizing from target current density
- winding length and hot resistance correction
- Steinmetz core-loss density and material-specific core loss
- core mass and window fill
- three-phase rated current
- lagging/leading voltage-regulation approximation
- short-circuit impedance percentage from equivalent R/X

## Deliberate non-duplication

The pack does not reimplement existing Yekaterina operations when the existing
operation already has the required semantics. Use the existing built-ins for:

- `elec.transformer_voltage`
- `elec.transformer_current`
- `elec.transformer_impedance`
- `elec.resistance_from_resistivity`
- `elec.current_density`
- `elec.power_i2r` for winding copper loss after hot resistance is known
- `eng.efficiency` when input and output power are already known

This keeps the transformer layer focused on calculations that are genuinely
missing from the current engine.

## Import

Read `packs/xfmr_design_v1.json` and pass the parsed JSON object as the only
argument to `udo.import`.

Conceptually:

```json
{"op":"udo.import","a":[<contents of packs/xfmr_design_v1.json>]}
```

A successful import returns `20`.

Then operations are directly available through normal MCP dispatch, `yk.find`,
and `yk.spec`, for example:

```json
{"op":"pack.xfmr.turns_sine","a":[13200,60,1.55,0.019]}
```

Expected result is approximately `1682.497438`, before the designer applies an
integer-turn rounding policy and re-checks actual flux density.

## Units and assumptions

Unless an opcode name states otherwise, SI units are used.

`volts_per_turn_sine`, `turns_sine`, and `flux_density_sine` use the conventional
sinusoidal RMS relationship:

`E = 4.44 * f * N * Bmax * Ae`

This relationship is only valid for sinusoidal excitation under the stated RMS
and peak-flux convention. Do not use it as-is for PWM or non-sinusoidal SMPS
magnetics.

`magnetic_reluctance` and `magnetizing_inductance` use a scalar relative
permeability. Real electrical steel is nonlinear; for serious design, obtain the
manufacturer B-H data near the intended operating flux density. The companion
`magnetizing_current_from_h` operation exists so a measured/interpolated H value
can be used instead of pretending permeability is constant.

`core_loss_density_steinmetz` deliberately takes `k`, `alpha`, and `beta` as
inputs. Their numerical values and units are material/model dependent and must
come from a compatible fitted data set. The engine must not ship one universal
Steinmetz constant.

`resistance_at_temperature` expects a resistance referenced to 20 C and a linear
temperature coefficient. Use the conductor manufacturer's value where available.

Voltage-regulation operations use the common approximate equivalent-series
R/X model and assume `0 <= power_factor <= 1`. Leading and lagging loads have
separate opcodes so sign convention is explicit rather than hidden in a flag.

## Recommended manufacturing calculation chain

For each candidate design:

1. `core_effective_area`
2. `volts_per_turn_sine`
3. `turns_sine` for each winding
4. round turns according to the design policy
5. `flux_density_sine` to recompute the actual operating B
6. `saturation_margin_pct`
7. `rated_current_three_phase` where applicable
8. `conductor_area_mm2`
9. calculate winding geometry / mean turn length externally or from the next geometry module
10. `winding_length`
11. existing `elec.resistance_from_resistivity`
12. `resistance_at_temperature`
13. existing `elec.power_i2r`
14. core-loss operation using manufacturer material data
15. `window_fill_pct`
16. equivalent-circuit regulation and impedance checks

A design optimizer should reject candidates that violate material, thermal,
window-fill, insulation, or standard constraints; this pack only supplies the
first numerical layer.

## Next implementation slice

The next v1.3 transformer slice should be native Rust rather than more formula
aliases and should add data-dependent calculations that UDO formulas cannot
model cleanly:

- interpolation of B-H curves
- interpolation of manufacturer core-loss tables
- lamination stacking / step-lap geometry
- winding radial and axial build
- skin/proximity-effect AC resistance
- leakage-reactance estimation from winding geometry
- stray-loss model
- thermal network and hot-spot model
- insulation/clearance tables as explicit standard-versioned constraints
- candidate-level pass/fail report and design objective scoring

Those capabilities require arrays, interpolation, validation, and structured
results and therefore belong in a real `xfmr` Rust module after the formula pack
has supplied stable equation and naming contracts.
