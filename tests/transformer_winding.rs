#[path = "../src/transformer_winding.rs"]
mod transformer_winding;

use serde_json::{Value, json};

fn run(op: &str, args: Vec<Value>) -> Value {
    transformer_winding::execute(op, &args)
        .expect("xfmr winding operation must be claimed")
        .expect("xfmr winding operation must succeed")
}

fn number(v: Value) -> f64 {
    v.as_f64().expect("numeric result")
}

fn close(actual: f64, expected: f64, tol: f64) {
    assert!((actual - expected).abs() <= tol, "{actual} != {expected}");
}

#[test]
fn rogowski_factor_matches_reference_expression() {
    let h = 0.5_f64;
    let w = 0.055_f64;
    let ratio = std::f64::consts::PI * h / w;
    let expected = 1.0 - (1.0 - (-ratio).exp()) / ratio;
    close(number(run("xfmr.rogowski_factor", vec![json!(h), json!(w)])), expected, 1e-14);

    let heq = number(run("xfmr.effective_leakage_height", vec![json!(h), json!(w)]));
    close(heq, h / expected, 1e-14);
    assert!(heq > h);
}

#[test]
fn concentric_leakage_inductance_matches_reference_formula() {
    let turns = 1000_u64;
    let turns_f = turns as f64;
    let h = 0.5_f64;
    let t1 = 0.020_f64;
    let d1 = 0.300_f64;
    let tg = 0.010_f64;
    let dg = 0.340_f64;
    let t2 = 0.025_f64;
    let d2 = 0.380_f64;

    let width = t1 + tg + t2;
    let ratio = std::f64::consts::PI * h / width;
    let k = 1.0 - (1.0 - (-ratio).exp()) / ratio;
    let h_eq = h / k;
    let atd = t1 * d1 / 3.0 + tg * dg + t2 * d2 / 3.0;
    let expected_l = 4.0 * std::f64::consts::PI * 1e-7
        * std::f64::consts::PI * turns_f * turns_f * atd / h_eq;

    let l = number(run(
        "xfmr.leakage_inductance_concentric",
        vec![
            json!(turns), json!(h), json!(t1), json!(d1),
            json!(tg), json!(dg), json!(t2), json!(d2),
        ],
    ));
    close(l, expected_l, 1e-14);
    close(l, 0.06527134684701377, 1e-14);
}

#[test]
fn leakage_reactance_is_omega_l() {
    let args = vec![
        json!(60.0), json!(1000), json!(0.5), json!(0.020), json!(0.300),
        json!(0.010), json!(0.340), json!(0.025), json!(0.380),
    ];
    let x = number(run("xfmr.leakage_reactance_concentric", args));
    close(x, 24.60671804933877, 1e-12);
}

#[test]
fn leakage_model_rejects_fractional_turn_count() {
    assert_eq!(
        transformer_winding::execute(
            "xfmr.leakage_inductance_concentric",
            &[
                json!(1000.5), json!(0.5), json!(0.020), json!(0.300),
                json!(0.010), json!(0.340), json!(0.025), json!(0.380),
            ],
        ),
        Some(Err("TYPE"))
    );
}

#[test]
fn dowell_factor_is_near_one_for_thin_single_layer() {
    let factor = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(0.0008531259202666352), json!(0.008531259202666352), json!(1)],
    ));
    close(factor, 1.0000088888550256, 1e-12);
}

#[test]
fn dowell_factor_captures_multilayer_proximity_penalty() {
    let skin_depth = 1.0;
    let one = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(1.0), json!(skin_depth), json!(1)],
    ));
    let two = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(1.0), json!(skin_depth), json!(2)],
    ));
    let five = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(1.0), json!(skin_depth), json!(5)],
    ));
    close(one, 1.0856357047503278, 1e-12);
    close(two, 1.4060090766532731, 1e-12);
    close(five, 3.6486226799738914, 1e-12);
    assert!(one < two && two < five);
}

#[test]
fn dowell_factor_stays_stable_at_extreme_ratios() {
    let thin = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(1e-8), json!(1.0), json!(8)],
    ));
    close(thin, 1.0, 1e-14);

    let thick = number(run(
        "xfmr.dowell_foil_ac_factor",
        vec![json!(100.0), json!(1.0), json!(3)],
    ));
    close(thick, 100.0 * 19.0 / 3.0, 1e-12);
}

#[test]
fn harmonic_copper_loss_sums_frequency_dependent_resistance() {
    let out = number(run(
        "xfmr.harmonic_copper_loss",
        vec![
            json!(0.08),
            json!([100.0, 20.0, 10.0]),
            json!([1.0, 1.25, 1.8]),
        ],
    ));
    let expected = 0.08 * (100.0_f64.powi(2) * 1.0 + 20.0_f64.powi(2) * 1.25 + 10.0_f64.powi(2) * 1.8);
    close(out, expected, 1e-12);
}

#[test]
fn harmonic_loss_rejects_bad_factor_or_length_mismatch() {
    assert_eq!(
        transformer_winding::execute(
            "xfmr.harmonic_copper_loss",
            &[json!(0.1), json!([10.0, 2.0]), json!([1.0])],
        ),
        Some(Err("ARG"))
    );
    assert_eq!(
        transformer_winding::execute(
            "xfmr.harmonic_copper_loss",
            &[json!(0.1), json!([10.0]), json!([0.9])],
        ),
        Some(Err("DOMAIN"))
    );
}

#[test]
fn non_xfmr_opcode_is_not_claimed() {
    assert_eq!(transformer_winding::execute("math.add", &[]), None);
}
