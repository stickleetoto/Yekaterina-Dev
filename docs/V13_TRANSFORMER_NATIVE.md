# Yekaterina v1.3 Native Transformer Core

This document defines the second transformer-design slice: calculations that are
not a good fit for scalar UDO formulas because they require arrays, interpolation,
integer winding layout, or structured results.

## Status

`src/transformer.rs` is a native Rust candidate module. It is compiled and tested
through `tests/transformer_native.rs`, but is intentionally not yet added to the
frozen v1.2 built-in registry. This lets the numerical/data contract stabilize
without breaking the v1.2 1,410-op release gates.

The existing `pack.xfmr.*` formula pack remains the immediately importable MCP
surface for scalar preliminary-design equations.

## Native operations

### `xfmr.bh_field_strength`

Arguments: `[bh_curve, flux_density_t]`

`bh_curve` is a strictly increasing array of `[B_tesla, H_ampere_per_meter]`
pairs. The operation performs linear interpolation only. Extrapolation is
rejected with `DOMAIN` because extrapolating electrical-steel B-H data near
saturation can produce dangerously misleading magnetizing-current estimates.

### `xfmr.magnetizing_current_bh`

Arguments: `[bh_curve, flux_density_t, magnetic_path_length_m, turns]`

Interpolates H from the material curve and evaluates `I_m = H*l/N`.

### `xfmr.core_loss_specific_interp`

Arguments:
`[frequency_axis_hz, flux_axis_t, specific_loss_grid_w_per_kg, frequency_hz, flux_density_t]`

The grid is rectangular: rows correspond to frequency and columns correspond to
flux density. Bilinear interpolation is used inside the measured table. The
engine rejects extrapolation instead of inventing material behavior outside the
manufacturer data.

### `xfmr.core_loss_from_grid`

Arguments:
`[frequency_axis_hz, flux_axis_t, specific_loss_grid_w_per_kg, frequency_hz, flux_density_t, core_mass_kg]`

Uses the same bilinear interpolation contract and multiplies the resulting
specific loss by core mass.

### `xfmr.winding_geometry`

Arguments:
`[turns, max_turns_per_layer, insulated_radial_mm, axial_pitch_mm, interlayer_insulation_mm, axial_margin_each_end_mm]`

Returns:

- `layers`
- `last_layer_turns`
- `radial_build_mm`
- `axial_height_mm`

This is a preliminary layered-winding geometry model. `insulated_radial_mm` and
`axial_pitch_mm` are explicit inputs so enamel/paper/covered-conductor dimensions
come from the real selected conductor rather than a hidden default.

### `xfmr.skin_depth`

Arguments: `[frequency_hz, resistivity_ohm_m, relative_permeability]`

Evaluates `delta = sqrt(rho/(pi*f*mu0*mur))`. This is a diagnostic used to decide
whether simple DC resistance is likely insufficient; it is not itself a full
proximity-effect or Dowell winding-loss model.

### `xfmr.thermal_two_node`

Arguments:
`[winding_loss_w, core_loss_w, winding_to_oil_k_per_w, core_to_oil_k_per_w, oil_to_ambient_k_per_w, ambient_c]`

Returns oil, winding, and core temperatures and rises for a steady-state lumped
thermal network. All thermal resistances are supplied by the caller from design,
test, CFD, or a validated empirical model. The engine deliberately contains no
universal transformer cooling coefficient.

## Data policy

For manufacturing work, material-dependent values must be sourced from the exact
selected material/conductor/insulation data set. Yekaterina should calculate and
interpolate; it should not silently invent electrical-steel, copper, aluminium,
insulation, or cooling properties.

The current native module therefore has two important fail-closed behaviors:

1. B-H and loss-table extrapolation is rejected.
2. Axes must be strictly increasing and grids must have exact dimensions.

## Promotion path

The native candidate should be promoted into the MCP surface only after:

1. native unit/integration tests pass;
2. clippy is clean under the pinned toolchain;
3. representative manufacturer B-H/loss tables are tested;
4. a v1.3 registry/audit policy replaces rather than mutates the frozen v1.2
   release invariants;
5. full-audit fixtures are added for every promoted `xfmr.*` opcode.

The next native slice should focus on leakage reactance and AC winding loss, but
only after their geometry/sign/unit contracts are pinned against trusted
references or measured transformer data.