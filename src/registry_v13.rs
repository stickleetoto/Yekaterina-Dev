//! v1.3 aggregate operation registry.
//!
//! The promoted v1.2 registry remains byte-for-byte in `registry.rs` as the
//! historical 1,410-operation baseline. This module layers the explicitly
//! staged transformer-native surface on top without rewriting that baseline.

use crate::registry_v12 as legacy;

pub use legacy::{OperationSource, OperationSpec};

pub const V12_BUILTIN_COUNT: usize = 1410;
pub const V13_TRANSFORMER_COUNT: usize = 15;
pub const BUILTIN_COUNT: usize = V12_BUILTIN_COUNT + V13_TRANSFORMER_COUNT;

pub const TRANSFORMER_OPERATIONS: &[OperationSpec] = &[
    op("xfmr.bh_field_strength", &["bh_curve", "b_t"], "number", "interpolate magnetic field strength H from a supplied B-H curve"),
    op("xfmr.magnetizing_current_bh", &["bh_curve", "b_t", "path_m", "turns"], "number", "magnetizing current from B-H curve field strength, magnetic path, and turns"),
    op("xfmr.core_loss_specific_interp", &["frequency_axis_hz", "flux_axis_t", "loss_grid_w_per_kg", "frequency_hz", "flux_t"], "number", "bilinear interpolation of manufacturer specific core-loss data"),
    op("xfmr.core_loss_from_grid", &["frequency_axis_hz", "flux_axis_t", "loss_grid_w_per_kg", "frequency_hz", "flux_t", "mass_kg"], "number", "total core loss from manufacturer specific-loss grid and core mass"),
    op("xfmr.winding_geometry", &["turns", "max_turns_per_layer", "insulated_radial_mm", "axial_pitch_mm", "interlayer_insulation_mm", "axial_margin_each_end_mm"], "object", "integer winding-layer occupancy and preliminary radial/axial build"),
    op("xfmr.skin_depth", &["frequency_hz", "resistivity_ohm_m", "relative_permeability"], "number", "classical conductor skin depth with explicit material permeability"),
    op("xfmr.thermal_two_node", &["winding_loss_w", "core_loss_w", "winding_to_oil_k_per_w", "core_to_oil_k_per_w", "oil_to_ambient_k_per_w", "ambient_c"], "object", "steady-state winding/core/oil thermal network with caller-supplied resistances"),
    op("xfmr.rogowski_factor", &["winding_height_m", "total_radial_width_m"], "number", "Rogowski fringing factor for equal-height concentric windings"),
    op("xfmr.effective_leakage_height", &["winding_height_m", "total_radial_width_m"], "number", "effective leakage height using the Rogowski fringing correction"),
    op("xfmr.leakage_inductance_concentric", &["turns_reference", "winding_height_m", "inner_radial_build_m", "inner_mean_diameter_m", "main_duct_radial_m", "main_duct_mean_diameter_m", "outer_radial_build_m", "outer_mean_diameter_m"], "number", "preliminary leakage inductance for equal-height cylindrical concentric windings"),
    op("xfmr.leakage_reactance_concentric", &["frequency_hz", "turns_reference", "winding_height_m", "inner_radial_build_m", "inner_mean_diameter_m", "main_duct_radial_m", "main_duct_mean_diameter_m", "outer_radial_build_m", "outer_mean_diameter_m"], "number", "preliminary leakage reactance for equal-height cylindrical concentric windings"),
    op("xfmr.dowell_foil_ac_factor", &["conductor_thickness_m", "skin_depth_m", "layer_count"], "number", "Dowell one-dimensional foil/layer AC-to-DC resistance factor"),
    op("xfmr.harmonic_copper_loss", &["hot_dc_resistance_ohm", "currents_rms_a", "ac_factors"], "number", "harmonic copper-loss aggregation with per-frequency AC resistance factors"),
    op("xfmr.candidate_evaluate", &["metrics", "constraints", "objectives"], "object", "evaluate transformer candidate constraints and normalized multi-objective desirability"),
    op("xfmr.candidate_rank", &["candidates", "constraints", "objectives"], "object", "deterministically rank transformer candidates with feasible designs first"),
];

const fn op(
    opcode: &'static str,
    args: &'static [&'static str],
    returns: &'static str,
    summary: &'static str,
) -> OperationSpec {
    OperationSpec {
        opcode,
        aliases: &[],
        args,
        returns,
        summary,
        source: OperationSource::BuiltIn,
    }
}

fn transformer_resolve(name: &str) -> Option<&'static OperationSpec> {
    let raw = name.trim();
    TRANSFORMER_OPERATIONS
        .iter()
        .find(|spec| spec.opcode == raw)
        .or_else(|| {
            TRANSFORMER_OPERATIONS
                .iter()
                .find(|spec| spec.opcode.eq_ignore_ascii_case(raw))
        })
}

pub fn resolve(name: &str) -> Option<&'static OperationSpec> {
    transformer_resolve(name).or_else(|| legacy::resolve(name))
}

pub fn search(query: &str, limit: usize) -> Vec<&'static OperationSpec> {
    if limit == 0 {
        return Vec::new();
    }
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut transformer: Vec<(u8, &'static OperationSpec)> = TRANSFORMER_OPERATIONS
        .iter()
        .filter_map(|spec| {
            let opcode = spec.opcode.to_ascii_lowercase();
            let summary = spec.summary.to_ascii_lowercase();
            let score = if opcode == q {
                0
            } else if opcode.starts_with(&q) {
                1
            } else if opcode.contains(&q) || summary.contains(&q) {
                2
            } else {
                return None;
            };
            Some((score, spec))
        })
        .collect();
    transformer.sort_by_key(|(score, spec)| (*score, spec.opcode));

    // Family-qualified transformer discovery should not be displaced by legacy
    // fuzzy matches. For every other query, preserve the v1.2 registry's exact
    // alias ownership/order and use transformer matches to fill remaining slots.
    if q == "xfmr" || q.starts_with("xfmr.") {
        let mut out: Vec<&'static OperationSpec> = transformer
            .into_iter()
            .take(limit)
            .map(|(_, spec)| spec)
            .collect();
        if out.len() < limit {
            for spec in legacy::search(query, limit - out.len()) {
                if !out.iter().any(|s| s.opcode == spec.opcode) {
                    out.push(spec);
                }
            }
        }
        return out;
    }

    let mut out = legacy::search(query, limit);
    if out.len() < limit {
        for (_, spec) in transformer {
            if out.len() >= limit {
                break;
            }
            if !out.iter().any(|s| s.opcode == spec.opcode) {
                out.push(spec);
            }
        }
    }
    out
}

pub fn capability_code(opcode: &str) -> &'static str {
    if opcode.starts_with("xfmr.") {
        "d"
    } else {
        legacy::capability_code(opcode)
    }
}

pub fn cost_code(opcode: &str) -> &'static str {
    if matches!(
        opcode,
        "xfmr.bh_field_strength"
            | "xfmr.magnetizing_current_bh"
            | "xfmr.core_loss_specific_interp"
            | "xfmr.core_loss_from_grid"
            | "xfmr.harmonic_copper_loss"
            | "xfmr.candidate_evaluate"
            | "xfmr.candidate_rank"
    ) {
        "n"
    } else if opcode.starts_with("xfmr.") {
        "1"
    } else {
        legacy::cost_code(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_count_is_explicit() {
        assert_eq!(V12_BUILTIN_COUNT, legacy::OPERATIONS.len());
        assert_eq!(TRANSFORMER_OPERATIONS.len(), V13_TRANSFORMER_COUNT);
        assert_eq!(BUILTIN_COUNT, 1425);
    }

    #[test]
    fn legacy_resolution_is_unchanged() {
        assert_eq!(resolve("math.add").unwrap().opcode, "math.add");
        assert_eq!(resolve("add").unwrap().opcode, "math.add");
    }

    #[test]
    fn transformer_resolution_and_search_are_live() {
        assert_eq!(resolve("xfmr.skin_depth").unwrap().opcode, "xfmr.skin_depth");
        let hits = search("xfmr", 20);
        assert_eq!(hits.len(), 15);
        assert!(hits.iter().any(|spec| spec.opcode == "xfmr.candidate_rank"));
    }
}
