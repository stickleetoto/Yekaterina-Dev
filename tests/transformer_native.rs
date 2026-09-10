#[path = "../src/transformer.rs"]
mod transformer;

use serde_json::{Value, json};

fn run(op: &str, args: Vec<Value>) -> Value {
    transformer::execute(op, &args)
        .expect("xfmr operation must be claimed")
        .expect("xfmr operation must succeed")
}

fn number(v: Value) -> f64 {
    v.as_f64().expect("numeric result")
}

fn close(actual: f64, expected: f64, tol: f64) {
    assert!((actual - expected).abs() <= tol, "{actual} != {expected}");
}

#[test]
fn bh_interpolation_is_linear_and_exact_on_nodes() {
    let curve = json!([[1.20, 42.0], [1.50, 80.0], [1.70, 160.0]]);
    close(number(run("xfmr.bh_field_strength", vec![curve.clone(), json!(1.50)])), 80.0, 1e-12);
    close(number(run("xfmr.bh_field_strength", vec![curve, json!(1.60)])), 120.0, 1e-12);
}

#[test]
fn bh_interpolation_rejects_extrapolation_and_unsorted_data() {
    let curve = json!([[1.20, 42.0], [1.50, 80.0], [1.70, 160.0]]);
    assert_eq!(
        transformer::execute("xfmr.bh_field_strength", &[curve, json!(1.80)]),
        Some(Err("DOMAIN"))
    );
    let unsorted = json!([[1.20, 42.0], [1.10, 80.0]]);
    assert_eq!(
        transformer::execute("xfmr.bh_field_strength", &[unsorted, json!(1.15)]),
        Some(Err("DOMAIN"))
    );
}

#[test]
fn magnetizing_current_uses_interpolated_h() {
    let curve = json!([[1.20, 42.0], [1.50, 80.0], [1.70, 160.0]]);
    let out = number(run(
        "xfmr.magnetizing_current_bh",
        vec![curve, json!(1.60), json!(1.25), json!(1680.0)],
    ));
    close(out, 120.0 * 1.25 / 1680.0, 1e-12);
}

#[test]
fn manufacturer_core_loss_grid_is_bilinear() {
    let frequencies = json!([50.0, 60.0]);
    let flux = json!([1.40, 1.60]);
    let losses = json!([
        [0.80, 1.20],
        [1.00, 1.50]
    ]);

    let specific = number(run(
        "xfmr.core_loss_specific_interp",
        vec![frequencies.clone(), flux.clone(), losses.clone(), json!(55.0), json!(1.50)],
    ));
    close(specific, 1.125, 1e-12);

    let total = number(run(
        "xfmr.core_loss_from_grid",
        vec![frequencies, flux, losses, json!(55.0), json!(1.50), json!(320.0)],
    ));
    close(total, 360.0, 1e-10);
}

#[test]
fn core_loss_grid_rejects_out_of_range_queries() {
    let frequencies = json!([50.0, 60.0]);
    let flux = json!([1.40, 1.60]);
    let losses = json!([[0.80, 1.20], [1.00, 1.50]]);
    assert_eq!(
        transformer::execute(
            "xfmr.core_loss_specific_interp",
            &[frequencies, flux, losses, json!(70.0), json!(1.50)],
        ),
        Some(Err("DOMAIN"))
    );
}

#[test]
fn winding_geometry_rounds_layers_up_and_reports_partial_last_layer() {
    let out = run(
        "xfmr.winding_geometry",
        vec![json!(1682), json!(210), json!(2.1), json!(3.0), json!(0.25), json!(8.0)],
    );
    assert_eq!(out["layers"], json!(9));
    assert_eq!(out["last_layer_turns"], json!(2));
    close(out["radial_build_mm"].as_f64().unwrap(), 20.9, 1e-12);
    close(out["axial_height_mm"].as_f64().unwrap(), 646.0, 1e-12);
}

#[test]
fn copper_skin_depth_at_60_hz_is_about_8_5_mm() {
    let out = number(run(
        "xfmr.skin_depth",
        vec![json!(60.0), json!(1.724e-8), json!(1.0)],
    ));
    close(out, 0.008529, 2e-6);
}

#[test]
fn thermal_two_node_conserves_common_oil_rise() {
    let out = run(
        "xfmr.thermal_two_node",
        vec![json!(1800.0), json!(400.0), json!(0.012), json!(0.018), json!(0.010), json!(25.0)],
    );
    close(out["oil_rise_k"].as_f64().unwrap(), 22.0, 1e-12);
    close(out["oil_c"].as_f64().unwrap(), 47.0, 1e-12);
    close(out["winding_c"].as_f64().unwrap(), 68.6, 1e-12);
    close(out["core_c"].as_f64().unwrap(), 54.2, 1e-12);
}

#[test]
fn non_xfmr_opcode_is_not_claimed() {
    assert_eq!(transformer::execute("elec.voltage", &[]), None);
}
