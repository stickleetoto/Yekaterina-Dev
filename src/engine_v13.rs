//! v1.3 execution shim.
//!
//! Existing operations continue through the frozen v1.2 engine. The new
//! transformer-native family is dispatched only after canonical resolution
//! through the aggregate v1.3 registry.

use serde_json::Value;

use crate::{
    engine_v12, registry, transformer, transformer_candidate, transformer_winding,
};

pub fn execute(opcode: &str, args: &[Value]) -> Result<Value, &'static str> {
    let spec = registry::resolve(opcode).ok_or("OP")?;
    match spec.opcode {
        "xfmr.bh_field_strength"
        | "xfmr.magnetizing_current_bh"
        | "xfmr.core_loss_specific_interp"
        | "xfmr.core_loss_from_grid"
        | "xfmr.winding_geometry"
        | "xfmr.skin_depth"
        | "xfmr.thermal_two_node" => transformer::execute(spec.opcode, args)
            .unwrap_or(Err("NYI")),

        "xfmr.rogowski_factor"
        | "xfmr.effective_leakage_height"
        | "xfmr.leakage_inductance_concentric"
        | "xfmr.leakage_reactance_concentric"
        | "xfmr.dowell_foil_ac_factor"
        | "xfmr.harmonic_copper_loss" => transformer_winding::execute(spec.opcode, args)
            .unwrap_or(Err("NYI")),

        "xfmr.candidate_evaluate" | "xfmr.candidate_rank" => {
            transformer_candidate::execute(spec.opcode, args).unwrap_or(Err("NYI"))
        }

        _ => engine_v12::execute(spec.opcode, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_dispatch_still_works() {
        assert_eq!(execute("math.add", &[json!(20), json!(22)]).unwrap(), json!(42.0));
    }

    #[test]
    fn transformer_dispatch_is_live() {
        let out = execute(
            "xfmr.skin_depth",
            &[json!(60.0), json!(1.724e-8), json!(1.0)],
        )
        .unwrap();
        let delta = out.as_f64().unwrap();
        assert!(delta > 0.008 && delta < 0.009);
    }

    #[test]
    fn candidate_dispatch_is_live() {
        let out = execute(
            "xfmr.candidate_evaluate",
            &[
                json!({"loss":1000.0}),
                json!([{"metric":"loss","max":1200.0}]),
                json!([{"metric":"loss","kind":"minimize","good":900.0,"bad":1500.0}]),
            ],
        )
        .unwrap();
        assert_eq!(out["pass"], json!(true));
    }
}
