#[path = "../src/deep_linalg.rs"] mod deep_linalg;
#[path = "../src/special_functions.rs"] mod special_functions;
#[path = "../src/inference.rs"] mod inference;
#[path = "../src/optimization.rs"] mod optimization;
#[path = "../src/ode.rs"] mod ode;
#[path = "../src/series.rs"] mod series;
#[path = "../src/verification.rs"] mod verification;
#[path = "../src/frame.rs"] mod frame;
#[path = "../src/curve.rs"] mod curve;
#[path = "../src/predicate.rs"] mod predicate;
#[path = "../src/time_ops.rs"] mod time_ops;
#[path = "../src/geodesy.rs"] mod geodesy;
#[path = "../src/thermodynamics.rs"] mod thermodynamics;
#[path = "../src/mechanics.rs"] mod mechanics;
#[path = "../src/fluids.rs"] mod fluids;
#[path = "../src/electrical.rs"] mod electrical;
#[path = "../src/optics.rs"] mod optics;
#[path = "../src/waves.rs"] mod waves;
#[path = "../src/data_ops.rs"] mod data_ops;
#[path = "../src/discrete.rs"] mod discrete;
#[path = "../src/chemistry.rs"] mod chemistry;
#[path = "../src/networking.rs"] mod networking;
#[path = "../src/color.rs"] mod color;
#[path = "../src/information.rs"] mod information;
#[path = "../src/astronomy.rs"] mod astronomy;
#[path = "../src/algebra.rs"] mod algebra;
#[path = "../src/complex_math.rs"] mod complex_math;
#[path = "../src/advanced_matrix.rs"] mod advanced_matrix;
#[path = "../src/advanced_stats.rs"] mod advanced_stats;
#[path = "../src/advanced_probability.rs"] mod advanced_probability;
#[path = "../src/advanced_signal.rs"] mod advanced_signal;
#[path = "../src/advanced_numerical.rs"] mod advanced_numerical;
#[path = "../src/physics.rs"] mod physics;
#[path = "../src/engineering.rs"] mod engineering;
#[path = "../src/registry.rs"]
mod registry;
#[path = "../src/precision.rs"]
mod precision;
#[path = "../src/extra_math.rs"]
mod extra_math;
#[path = "../src/stats.rs"]
mod stats;
#[path = "../src/vector.rs"]
mod vector;
#[path = "../src/geometry.rs"]
mod geometry;
#[path = "../src/practical.rs"]
mod practical;
#[path = "../src/signal.rs"]
mod signal;
#[path = "../src/engine.rs"]
mod engine;

#[path = "../src/matrix.rs"]
mod matrix;
#[path = "../src/probability.rs"]
mod probability;
#[path = "../src/radix.rs"]
mod radix;
#[path = "../src/numerical.rs"]
mod numerical;
#[path = "../src/formula.rs"]
mod formula;

use serde_json::json;

#[test]
fn arithmetic_works() {
    let result = engine::execute("math.mul", &[json!(137), json!(829)]).unwrap();
    assert_eq!(result, json!(113573.0));
}

#[test]
fn stats_work() {
    let result = engine::execute("avg", &[json!([1,2,3,4])]).unwrap();
    assert_eq!(result, json!(2.5));
}

#[test]
fn divide_by_zero_is_compact_error() {
    assert_eq!(engine::execute("math.div", &[json!(1), json!(0)]), Err("DIV0"));
}

#[test]
fn bigint_is_exact() {
    let result = engine::execute(
        "int.mul",
        &[json!("12345678901234567890"), json!("10")],
    ).unwrap();
    assert_eq!(result, json!("123456789012345678900"));
}

#[test]
fn decimal_add_is_exact() {
    let result = engine::execute("dec.add", &[json!("0.1"), json!("0.2")]).unwrap();
    assert_eq!(result, json!("0.3"));
}

#[test]
fn variance_and_median_work() {
    assert_eq!(engine::execute("stat.variance", &[json!([1,2,3])]).unwrap(), json!(2.0 / 3.0));
    assert_eq!(engine::execute("stat.median", &[json!([9,1,4,2])]).unwrap(), json!(3.0));
}

#[test]
fn huge_pow_is_rejected_before_giant_result() {
    let result = engine::execute("int.pow", &[json!("9999999999"), json!(1_000_000)]);
    assert_eq!(result, Err("OUT_LIMIT"));
}

#[test]
fn alpha5_math_ops_work() {
    assert_eq!(engine::execute("math.cbrt", &[json!(27)]).unwrap(), json!(3.0));
    assert_eq!(engine::execute("math.approx_eq", &[json!(1.0), json!(1.001), json!(0.01)]).unwrap(), json!(true));
}

#[test]
fn alpha5_stats_and_vectors_work() {
    assert_eq!(engine::execute("stat.range", &[json!([1,5,3])]).unwrap(), json!(4.0));
    assert_eq!(engine::execute("vec.dot", &[json!([1,2,3]), json!([4,5,6])]).unwrap(), json!(32.0));
    assert_eq!(engine::execute("vec.cross3", &[json!([1,0,0]), json!([0,1,0])]).unwrap(), json!([0.0,0.0,1.0]));
}

#[test]
fn alpha5_practical_and_signal_work() {
    assert_eq!(engine::execute("pct.of", &[json!(200), json!(15)]).unwrap(), json!(30.0));
    assert_eq!(engine::execute("unit.length", &[json!(1), json!("km"), json!("m")]).unwrap(), json!(1000.0));
    assert_eq!(engine::execute("signal.diff", &[json!([1,4,9])]).unwrap(), json!([3.0,5.0]));
}

#[test]
fn alpha6_matrix_ops_work() {
    assert_eq!(engine::execute("mat.mul", &[json!([[1,2],[3,4]]), json!([[5,6],[7,8]])]).unwrap(), json!([[19.0,22.0],[43.0,50.0]]));
    assert_eq!(engine::execute("mat.det", &[json!([[1,2],[3,4]])]).unwrap(), json!(-2.0));
    assert_eq!(engine::execute("mat.vecmul", &[json!([[1,2],[3,4]]), json!([5,6])]).unwrap(), json!([17.0,39.0]));
}

#[test]
fn alpha6_probability_ops_work() {
    assert_eq!(engine::execute("prob.combination", &[json!(5), json!(2)]).unwrap(), json!(10.0));
    assert!((engine::execute("prob.normal_cdf", &[json!(0), json!(0), json!(1)]).unwrap().as_f64().unwrap() - 0.5).abs() < 1e-7);
    let soft = engine::execute("prob.softmax", &[json!([1,2,3])]).unwrap();
    let sum: f64 = soft.as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).sum();
    assert!((sum - 1.0).abs() < 1e-12);
}

#[test]
fn alpha6_numerical_and_radix_work() {
    assert_eq!(engine::execute("base.convert", &[json!("ff"), json!(16), json!(10)]).unwrap(), json!("255"));
    assert_eq!(engine::execute("bit.popcount", &[json!(15)]).unwrap(), json!(4));
    let root = engine::execute("num.bisect", &[json!({"e":"x^2-2"}), json!(1.0), json!(2.0)]).unwrap().as_f64().unwrap();
    assert!((root - 2.0_f64.sqrt()).abs() < 1e-8);
}

#[test]
fn alpha6_practical_signal_stats_work() {
    assert_eq!(engine::execute("unit.speed", &[json!(36), json!("kmh"), json!("mps")]).unwrap(), json!(10.0));
    assert_eq!(engine::execute("signal.energy", &[json!([1,2,3])]).unwrap(), json!(14.0));
    assert_eq!(engine::execute("stat.weighted_mean", &[json!([10,20]), json!([1,3])]).unwrap(), json!(17.5));
}

#[test]
fn alpha10_domain_packs_work() {
    assert_eq!(
        engine::execute("time.days_in_month", &[json!(2024), json!(2)]).unwrap(),
        json!(29)
    );
    assert_eq!(
        engine::execute("geod.normalize_lon", &[json!(190)]).unwrap(),
        json!(-170.0)
    );
    assert_eq!(
        engine::execute("thermo.celsius_to_kelvin", &[json!(0)]).unwrap(),
        json!(273.15)
    );
    assert_eq!(
        engine::execute("mech.kinetic_energy", &[json!(2), json!(3)]).unwrap(),
        json!(9.0)
    );
    assert_eq!(
        engine::execute("fluid.flow_rate", &[json!(2), json!(3)]).unwrap(),
        json!(6.0)
    );
    assert_eq!(
        engine::execute("elec.voltage", &[json!(2), json!(5)]).unwrap(),
        json!(10.0)
    );
    assert_eq!(
        engine::execute("optics.lens_power_diopter", &[json!(0.5)]).unwrap(),
        json!(2.0)
    );
    assert_eq!(
        engine::execute("wave.beat_frequency", &[json!(440), json!(442)]).unwrap(),
        json!(2.0)
    );
    assert_eq!(
        engine::execute("data.ceil_div", &[json!(10), json!(3)]).unwrap(),
        json!(4)
    );
}


#[test]
fn alpha11_verification_convergence_works() {
    let values = json!([1.25, 1.0625, 1.015625, 1.00390625]);
    let resolutions = json!([10, 20, 40, 80]);

    let strict = engine::execute(
        "verify.grid_convergence",
        &[values.clone(), resolutions.clone(), json!(0.01), json!(0.01)],
    )
    .unwrap();
    assert_eq!(strict["ok"], json!(false));
    assert_eq!(strict["p"], json!(2.0));

    let relaxed = engine::execute(
        "verify.grid_convergence",
        &[values, resolutions, json!(0.02), json!(0.02)],
    )
    .unwrap();
    assert_eq!(relaxed["ok"], json!(true));
    assert_eq!(relaxed["p"], json!(2.0));
}

#[test]
fn alpha11_frame_mismatch_is_rejected() {
    let a = engine::execute("frame.vec3", &[json!([1, 0, 0]), json!("leg")]).unwrap();
    let b = engine::execute("frame.vec3", &[json!([0, 1, 0]), json!("foot")]).unwrap();
    assert_eq!(engine::execute("frame.add_vec", &[a, b]), Err("FRAME"));
}

#[test]
fn alpha11_rotation_basis_is_explicit() {
    let t = engine::execute("frame.rot_x", &[json!(std::f64::consts::FRAC_PI_2), json!("a"), json!("b")]).unwrap();
    let b = engine::execute("frame.basis", &[t]).unwrap();
    let z = b["z"].as_array().unwrap();
    assert!(z[0].as_f64().unwrap().abs() < 1e-12);
    assert!((z[1].as_f64().unwrap() + 1.0).abs() < 1e-12);
    assert!(z[2].as_f64().unwrap().abs() < 1e-12);
}

#[test]
fn alpha11_curve_topology_audit_catches_crossing() {
    let bow = json!([[0,0],[2,2],[0,2],[2,0],[0,0]]);
    let audit = engine::execute("curve.audit", &[bow]).unwrap();
    assert_eq!(audit["c"], json!(true));
    assert_eq!(audit["s"], json!(false));
    assert_eq!(audit["x"], json!(1));
}

#[test]
fn alpha11_backtrack_count_counts_each_immediate_reversal() {
    let retrace = json!([[0,0],[2,0],[0,0],[0,2],[0,0]]);
    assert_eq!(
        engine::execute("curve.backtrack_count", &[retrace]).unwrap(),
        json!(2)
    );
}

#[test]
fn alpha11_circle_containment_reports_clearance() {
    let square = json!([[0,0],[10,0],[10,10],[0,10],[0,0]]);
    assert_eq!(engine::execute("predicate.circle_in_polygon", &[json!([5,5]), json!(4), square.clone()]).unwrap(), json!(true));
    assert_eq!(engine::execute("predicate.circle_in_polygon", &[json!([5,5]), json!(6), square.clone()]).unwrap(), json!(false));
    assert_eq!(engine::execute("predicate.circle_polygon_clearance", &[json!([5,5]), json!(4), square]).unwrap(), json!(1.0));
}

#[test]
fn alpha12_svd_and_pinv_properties_hold() {
    let a = json!([[3.0, 0.0], [0.0, 1.0]]);
    let svd = engine::execute("linalg.svd", &[a.clone()]).unwrap();
    assert!(svd["residual"].as_f64().unwrap() < 1e-10);
    let s = svd["s"].as_array().unwrap();
    assert!((s[0].as_f64().unwrap() - 3.0).abs() < 1e-9);
    assert!((s[1].as_f64().unwrap() - 1.0).abs() < 1e-9);
    let mp = engine::execute("linalg.moore_penrose_error", &[a]).unwrap();
    assert!(mp["a_pa"].as_f64().unwrap() < 1e-9);
    assert!(mp["p_ap"].as_f64().unwrap() < 1e-9);
}

#[test]
fn alpha12_special_function_identities_hold() {
    let g5 = engine::execute("special.gamma", &[json!(5.0)]).unwrap().as_f64().unwrap();
    let g6 = engine::execute("special.gamma", &[json!(6.0)]).unwrap().as_f64().unwrap();
    assert!((g6 - 5.0 * g5).abs() < 1e-8);
    let e = engine::execute("special.erf", &[json!(0.7)]).unwrap().as_f64().unwrap();
    let em = engine::execute("special.erf", &[json!(-0.7)]).unwrap().as_f64().unwrap();
    assert!((e + em).abs() < 1e-12);
    let w = engine::execute("special.lambert_w0", &[json!(1.0)]).unwrap().as_f64().unwrap();
    assert!((w * w.exp() - 1.0).abs() < 1e-10);
}

#[test]
fn alpha12_optimization_finds_known_minima() {
    let r = engine::execute(
        "optimize.brent",
        &[json!({"e":"(x-2)^2"}), json!(-5.0), json!(5.0), json!(1e-10), json!(1000)],
    ).unwrap();
    assert!((r["x"].as_f64().unwrap() - 2.0).abs() < 1e-6);
    assert!(r["fx"].as_f64().unwrap() < 1e-10);
    let b = engine::execute(
        "optimize.bfgs2d",
        &[json!({"e":"(x-1)^2+(y+2)^2"}), json!(5.0), json!(5.0)],
    ).unwrap();
    assert!(b["grad_norm"].as_f64().unwrap() < 1e-6);
}

#[test]
fn alpha12_rk45_tracks_exponential_growth() {
    let r = engine::execute(
        "ode.rk45",
        &[json!({"e":"y"}), json!(0.0), json!(1.0), json!(1.0), json!(1e-10), json!(1e-9)],
    ).unwrap();
    assert_eq!(r["converged"], json!(true));
    assert!((r["y"].as_f64().unwrap() - std::f64::consts::E).abs() < 1e-7);
}

#[test]
fn alpha12_series_acceleration_and_approximation_work() {
    let a = engine::execute("series.aitken", &[json!([1.0, 1.5, 1.75])]).unwrap().as_f64().unwrap();
    assert!((a - 2.0).abs() < 1e-12);
    let e = engine::execute("series.taylor_exp", &[json!(1.0), json!(18)]).unwrap().as_f64().unwrap();
    assert!((e - std::f64::consts::E).abs() < 1e-14);
    let c = engine::execute(
        "series.chebyshev_coefficients",
        &[json!({"e":"x^2"}), json!(-1.0), json!(1.0), json!(8)],
    ).unwrap();
    let v = engine::execute("series.chebyshev_eval", &[c, json!(0.3)]).unwrap().as_f64().unwrap();
    assert!((v - 0.09).abs() < 1e-10);
}

#[test]
fn alpha12_erf_zero_is_exact() {
    assert_eq!(engine::execute("special.erf", &[json!(0)]).unwrap(), json!(0.0));
    assert_eq!(engine::execute("special.erfc", &[json!(0)]).unwrap(), json!(1.0));
}

#[test]
fn alpha12_hotfix4_numerical_edge_contracts_hold() {
    assert_eq!(engine::execute("linalg.condition_number", &[json!([[1.0,0.0],[0.0,0.0]])]).unwrap(), json!(null));
    let cond = engine::execute("linalg.condition_number", &[json!([[1e-20,0.0],[0.0,2e-20]])]).unwrap().as_f64().unwrap();
    assert!((cond - 2.0).abs() < 1e-9);
    assert_eq!(engine::execute("linalg.power_iteration", &[json!([[1.0,0.0],[0.0,-1.0]]), json!(1e-10), json!(1000)]), Err("NO_CONVERGE"));
    assert_eq!(engine::execute("special.gamma", &[json!(-1.0)]), Err("DOMAIN"));
    let j0 = engine::execute("special.bessel_j0", &[json!(40.0)]).unwrap().as_f64().unwrap();
    assert!((j0 - 0.0073668905842372896).abs() < 1e-12);
    assert_eq!(engine::execute("optimize.coordinate_descent2d", &[json!({"e":"x^2+y^2"}), json!(1.0), json!(1.0), json!(10.0), json!(0.0), json!(100)]), Err("DOMAIN"));
    let fc = engine::execute("series.fourier_coefficients", &[json!([1.0,-1.0,1.0,-1.0]), json!(2)]).unwrap();
    let fa = fc["a"].as_array().unwrap();
    assert_eq!(fa.len(), 3);
    assert!(fa[0].as_f64().unwrap().abs() < 1e-12);
    assert!(fa[1].as_f64().unwrap().abs() < 1e-12);
    assert!((fa[2].as_f64().unwrap() - 1.0).abs() < 1e-12);
}

#[test]
fn alpha12_hotfix4_formula_recursion_is_bounded() {
    use std::collections::HashMap;
    let vars = HashMap::new();
    assert_eq!(formula::eval(&format!("{}1", "-".repeat(80)), &vars), Err("LIMIT"));
    assert_eq!(formula::eval(&("1^".repeat(70) + "1"), &vars), Err("LIMIT"));
}

#[test]
fn alpha12_hotfix4_resource_guards_hold() {
    let huge = vec![json!(1.0); 4097];
    assert_eq!(engine::execute("series.euler_transform", &[json!(huge)]), Err("LIMIT"));
    assert_eq!(engine::execute("series.geometric_sum", &[json!(1.0), json!(0.5), json!(1_000_001u64)]), Err("LIMIT"));
    let ev = engine::execute("ode.event_target", &[json!({"e":"1"}), json!(0.0), json!(0.0), json!(1.0), json!(0.55), json!(0.2), json!(1e-9)]).unwrap();
    assert_eq!(ev["found"], json!(true));
    assert!((ev["x"].as_f64().unwrap() - 0.55).abs() < 1e-7);
    assert!((ev["y"].as_f64().unwrap() - 0.55).abs() < 1e-7);
}

#[test]
fn alpha12_hotfix6_case_distinct_data_rates_compute_correctly() {
    let bps = engine::execute("data.throughput_bps", &[json!(800.0), json!(2.0)]).unwrap();
    let bytes_per_sec = engine::execute("data.throughput_Bps", &[json!(800.0), json!(2.0)]).unwrap();
    assert_eq!(bps, json!(3200.0));
    assert_eq!(bytes_per_sec, json!(400.0));
}
