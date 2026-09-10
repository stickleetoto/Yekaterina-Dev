# Yekaterina v1.3 Transformer Design Engine

This development line builds a manufacturing-oriented transformer calculation
backend for preliminary 50/60 Hz power/distribution transformer design.

It deliberately separates three layers while the v1.2 built-in registry remains
frozen:

1. an importable scalar formula pack for stable basic manufacturing equations;
2. a native Rust material/geometry/thermal core for array and structured data;
3. a native Rust winding-field core for leakage and frequency-dependent copper loss.

This is an engineering calculation kernel, not a certification engine. It does
not assert IEC/IEEE/DOE compliance, dielectric safety, mechanical short-circuit
withstand, or a guaranteed factory temperature-rise result.

## Layer 1: importable manufacturing formula pack

`packs/xfmr_design_v1.json` exposes 20 operations under `pack.xfmr.*`:

- effective core area and sinusoidal volts/turn;
- turns and back-calculated flux density;
- saturation margin;
- magnetic reluctance and scalar-permeability magnetizing inductance;
- magnetizing current from a supplied B-H field-strength sample;
- conductor sizing from target current density;
- winding length and hot resistance correction;
- Steinmetz and manufacturer-specific core-loss calculations;
- core mass and window fill;
- three-phase rated current;
- lagging/leading voltage regulation;
- short-circuit impedance percentage from equivalent R/X.

### Deliberate non-duplication

The pack does not reimplement existing Yekaterina operations when the existing
operation already has the required semantics. Existing built-ins remain
canonical for:

- `elec.transformer_voltage`;
- `elec.transformer_current`;
- `elec.transformer_impedance`;
- `elec.resistance_from_resistivity`;
- `elec.current_density`;
- `elec.power_i2r` once winding resistance is known;
- `eng.efficiency` once input and output power are known.

## Layer 2: native material, geometry and thermal core

`src/transformer.rs` handles calculations that scalar UDO formulas cannot model
cleanly:

- monotonic B-H curve interpolation with no extrapolation;
- magnetizing current directly from a B-H curve;
- bilinear interpolation over manufacturer frequency/B core-loss tables;
- total core loss from a W/kg grid and core mass;
- integer winding-layer occupancy and radial/axial build;
- conductor skin depth;
- a steady-state two-source winding/core/oil thermal network with caller-supplied
  thermal resistances.

Material interpolation fails closed outside the supplied manufacturer data. The
engine must not silently extrapolate a B-H or loss curve into saturation or an
uncharacterized operating region.

## Layer 3: native winding-field core

`src/transformer_winding.rs` adds preliminary field/loss calculations driven by
actual winding geometry:

- `xfmr.rogowski_factor`;
- `xfmr.effective_leakage_height`;
- `xfmr.leakage_inductance_concentric`;
- `xfmr.leakage_reactance_concentric`;
- `xfmr.dowell_foil_ac_factor`;
- `xfmr.harmonic_copper_loss`.

The concentric leakage model uses the classical equal-height cylindrical
winding approximation:

`L_sigma = mu0*pi*N^2/H_eq * (T1*D1/3 + Tg*Dg + T2*D2/3)`

with:

`H_eq = H/K_R`

and:

`K_R = 1 - (1-exp(-pi*H/W))/(pi*H/W)`

where `W = T1 + Tg + T2`.

The model is useful for preliminary geometry optimization and impedance target
search. It is not a substitute for FEM or factory short-circuit impedance
measurement when end effects, unequal winding heights, screens, tertiary
windings, structural steel, complex ducts or non-axisymmetric geometry are
important.

`xfmr.dowell_foil_ac_factor` implements the classical one-dimensional Dowell
full-winding AC/DC resistance factor for rectangular foil/layer geometry. The
API deliberately requires conductor thickness, skin depth and layer count so the
assumptions remain visible. Round wire, CTC, partial layers, interleaving,
transposition, axial ducts and 2D fringing require dedicated models rather than
hidden correction constants.

`xfmr.harmonic_copper_loss` accepts an RMS current decomposition and one
frequency-specific AC/DC resistance factor for each component. This makes
non-sinusoidal load-loss calculations possible without pretending one single
Rac correction applies to every harmonic.

## Recommended candidate calculation chain

A preliminary optimizer can now execute the following chain for each candidate:

1. determine effective core area;
2. calculate volts/turn and integer winding turns;
3. back-check actual Bmax and saturation margin;
4. interpolate B-H data and estimate excitation current;
5. interpolate manufacturer core-loss data at actual frequency/B;
6. size conductor area from current-density policy;
7. construct integer winding geometry and radial/axial build;
8. calculate winding length and hot DC resistance;
9. calculate skin depth and, where its assumptions apply, Dowell Rac/Rdc;
10. calculate harmonic copper loss;
11. calculate Rogowski factor and preliminary leakage L/X from winding geometry;
12. evaluate voltage regulation / impedance using the resulting equivalent model;
13. run the winding/core/oil thermal network;
14. reject candidates that violate material, window, thermal, insulation or
    standard-versioned constraints.

This is enough to make geometry changes participate in both electrical loss and
impedance optimization rather than treating `%Z` and load loss as fixed inputs.

## Units and modeling rules

Unless an opcode states otherwise, SI units are used.

The 4.44 EMF relationships are for sinusoidal excitation using RMS voltage and
peak flux density. They must not be reused unchanged for PWM/non-sinusoidal SMPS
magnetics.

B-H permeability is nonlinear. Prefer manufacturer B-H data near the intended
operating point rather than treating one scalar relative permeability as a
material identity.

Steinmetz `k`, `alpha` and `beta` are material/model/unit dependent. They are
caller data, never universal Yekaterina constants.

Thermal resistances are caller-supplied from geometry, measurement, validated
correlations or CFD. The native thermal network only solves the network; it does
not invent one universal transformer thermal resistance.

## Verification

- `scripts/verify_transformer_pack.py` independently verifies all 20 scalar
  formula-pack operations.
- `tests/transformer_native.rs` verifies B-H/loss interpolation, winding geometry,
  skin depth and the thermal network.
- `tests/transformer_winding.rs` independently checks the Rogowski expression,
  concentric leakage inductance/reactance, Dowell factor behavior and harmonic
  copper-loss aggregation.
- dedicated GitHub Actions workflows gate all three transformer layers.
- the unchanged v1.2 full CI remains the regression gate for the existing engine.

## Next implementation slice

The next useful manufacturing slice is not more basic algebra. Priority items are:

- unequal-height / multi-winding leakage models;
- CTC and strand-eddy/circulating-current loss models for large power windings;
- validated stray structural loss estimates;
- winding mean-diameter / duct / transposition geometry builders;
- thermal hot-spot models calibrated against test data;
- mechanical short-circuit force/stress calculations;
- versioned insulation and clearance constraint tables;
- candidate-level structured pass/fail reports and multi-objective scoring;
- explicit v1.3 registry/audit policy before promoting native `xfmr.*` operations
  onto the MCP built-in surface.
