#[path = "../src/deep_linalg.rs"] mod deep_linalg;
#[path = "../src/special_functions.rs"] mod special_functions;
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
#[path = "../src/registry.rs"] mod registry;
#[path = "../src/precision.rs"] mod precision;
#[path = "../src/extra_math.rs"] mod extra_math;
#[path = "../src/stats.rs"] mod stats;
#[path = "../src/vector.rs"] mod vector;
#[path = "../src/matrix.rs"] mod matrix;
#[path = "../src/geometry.rs"] mod geometry;
#[path = "../src/practical.rs"] mod practical;
#[path = "../src/signal.rs"] mod signal;
#[path = "../src/probability.rs"] mod probability;
#[path = "../src/numerical.rs"] mod numerical;
#[path = "../src/radix.rs"] mod radix;
#[path = "../src/formula.rs"] mod formula;
#[path = "../src/engine.rs"] mod engine;

use serde_json::{Value, json};

fn scalar(op: &str, args: &[Value]) -> f64 {
    engine::execute(op, args).unwrap().as_f64().unwrap()
}
fn close(got: f64, expected: f64, tol: f64) {
    assert!((got-expected).abs() <= tol.max(expected.abs()*tol), "got={got} expected={expected}");
}

#[test]
fn golden_matrix() {
    assert_eq!(engine::execute("mat.mul", &[json!([[1,2],[3,4]]),json!([[5,6],[7,8]])]).unwrap(), json!([[19.0,22.0],[43.0,50.0]]));
    close(scalar("mat.det", &[json!([[6,1,1],[4,-2,5],[2,8,7]])]), -306.0, 1e-12);
    let inv=engine::execute("mat.inverse", &[json!([[4,7],[2,6]])]).unwrap();
    let a=inv.as_array().unwrap();
    close(a[0][0].as_f64().unwrap(),0.6,1e-12); close(a[1][1].as_f64().unwrap(),0.4,1e-12);
    assert_eq!(engine::execute("mat.inverse", &[json!([[1,2],[2,4]])]), Err("SINGULAR"));
}

#[test]
fn golden_probability() {
    close(scalar("prob.binomial_pmf", &[json!(4),json!(2),json!(0.5)]),0.375,1e-12);
    close(scalar("prob.normal_cdf", &[json!(0),json!(0),json!(1)]),0.5,2e-7);
    close(scalar("prob.binary_entropy", &[json!(0.5)]),1.0,1e-12);
    let s=engine::execute("prob.softmax", &[json!([0,0])]).unwrap();
    assert_eq!(s,json!([0.5,0.5]));
}

#[test]
fn golden_statistics() {
    close(scalar("stat.percentile", &[json!([0,10,20,30]),json!(25)]),7.5,1e-12);
    close(scalar("stat.sample_variance", &[json!([1,2,3])]),1.0,1e-12);
    close(scalar("stat.correlation", &[json!([1,2,3]),json!([2,4,6])]),1.0,1e-12);
    assert_eq!(engine::execute("stat.cumsum", &[json!([1,2,3])]).unwrap(),json!([1.0,3.0,6.0]));
}

#[test]
fn golden_numerical() {
    close(scalar("num.simpson_uniform", &[json!([0,1,4]),json!(1)]),8.0/3.0,1e-12);
    close(scalar("num.bisect", &[json!({"e":"x^2-2"}),json!(1),json!(2)]),2.0_f64.sqrt(),1e-8);
    close(scalar("num.integrate", &[json!({"e":"x^2"}),json!(0),json!(1),json!(10000)]),1.0/3.0,1e-7);
    close(scalar("num.newton", &[json!({"e":"x^2-2"}),json!(1)]),2.0_f64.sqrt(),1e-8);
}

#[test]
fn golden_signal() {
    assert_eq!(engine::execute("signal.convolve", &[json!([1,2]),json!([1,1])]).unwrap(),json!([1.0,3.0,2.0]));
    assert_eq!(engine::execute("signal.correlate", &[json!([1,2]),json!([3,4])]).unwrap(),json!([4.0,11.0,6.0]));
    close(scalar("signal.energy", &[json!([1,2,3])]),14.0,1e-12);
    assert_eq!(engine::execute("signal.normalize_peak", &[json!([0,0])]),Err("DOMAIN"));
}

#[test]
fn golden_finance_and_units() {
    close(scalar("fin.present_value", &[json!(110),json!(10),json!(1)]),100.0,1e-12);
    close(scalar("fin.cagr", &[json!(100),json!(121),json!(2)]),10.0,1e-12);
    close(scalar("unit.speed", &[json!(36),json!("kmh"),json!("mps")]),10.0,1e-12);
    close(scalar("unit.energy", &[json!(1),json!("kWh"),json!("J")]),3_600_000.0,1e-12);
    assert_eq!(engine::execute("unit.temp", &[json!(-1),json!("K"),json!("C")]),Err("DOMAIN"));
}

#[test]
fn golden_geometry_and_vectors() {
    close(scalar("geo.triangle_area", &[json!(3),json!(4),json!(5)]),6.0,1e-12);
    close(scalar("vec.dot", &[json!([1,2,3]),json!([4,5,6])]),32.0,1e-12);
    close(scalar("vec.cosine", &[json!([1,0]),json!([1,1])]),1.0/2.0_f64.sqrt(),1e-12);
    assert_eq!(engine::execute("vec.cross3", &[json!([1,0,0]),json!([0,1,0])]).unwrap(),json!([0.0,0.0,1.0]));
}

#[test]
fn golden_bit_base_and_exact() {
    assert_eq!(engine::execute("bit.popcount", &[json!(45)]).unwrap(),json!(4));
    assert_eq!(engine::execute("base.convert", &[json!("ff"),json!(16),json!(10)]).unwrap(),json!("255"));
    assert_eq!(engine::execute("int.mul", &[json!("12345678901234567890"),json!("10")]).unwrap(),json!("123456789012345678900"));
    assert_eq!(engine::execute("dec.add", &[json!("0.1"),json!("0.2")]).unwrap(),json!("0.3"));
}

#[test]
fn golden_math() {
    close(scalar("math.hypot", &[json!(3),json!(4)]),5.0,1e-12);
    close(scalar("math.deg2rad", &[json!(180)]),std::f64::consts::PI,1e-12);
    close(scalar("math.log2", &[json!(8)]),3.0,1e-12);
    assert_eq!(engine::execute("math.sqrt", &[json!(-1)]),Err("DOMAIN"));
}

#[test]
fn golden_alpha8_algebra_and_complex() {
    assert_eq!(engine::execute("alg.factorial", &[json!(10)]).unwrap(), json!("3628800"));
    assert_eq!(engine::execute("alg.totient", &[json!(36)]).unwrap(), json!(12));
    close(scalar("cplx.abs", &[json!([3,4])]), 5.0, 1e-12);
    let z = engine::execute("cplx.mul", &[json!([1,2]), json!([3,4])]).unwrap();
    assert_eq!(z, json!([-5.0,10.0]));
}

#[test]
fn golden_alpha8_advanced_matrix() {
    assert_eq!(engine::execute("mat.rank", &[json!([[1,2],[2,4]])]).unwrap(), json!(1));
    assert_eq!(engine::execute("mat.solve", &[json!([[2,1],[1,-1]]), json!([5,1])]).unwrap(), json!([2.0,1.0]));
    assert_eq!(engine::execute("mat.power", &[json!([[1,1],[1,0]]), json!(5)]).unwrap(), json!([[8.0,5.0],[5.0,3.0]]));
}

#[test]
fn golden_alpha8_regression_and_tests() {
    let r = engine::execute("reg.linear", &[json!([1,2,3]),json!([2,4,6])]).unwrap();
    close(r["slope"].as_f64().unwrap(), 2.0, 1e-12);
    close(r["r2"].as_f64().unwrap(), 1.0, 1e-12);
    close(scalar("test.t_one_sample", &[json!([1,2,3]),json!(2)]), 0.0, 1e-12);
    close(scalar("stat.spearman", &[json!([1,2,3]),json!([3,2,1])]), -1.0, 1e-12);
}

#[test]
fn golden_alpha8_probability() {
    close(scalar("prob.geometric_pmf", &[json!(3),json!(0.5)]), 0.125, 1e-12);
    close(scalar("prob.hypergeometric_pmf", &[json!(10),json!(5),json!(4),json!(2)]), 100.0/210.0, 1e-12);
    close(scalar("prob.expected_value", &[json!([1,2,3]),json!([0.2,0.3,0.5])]), 2.3, 1e-12);
}

#[test]
fn golden_alpha8_signal() {
    let fft = engine::execute("signal.fft", &[json!([1,0,0,0])]).unwrap();
    assert_eq!(fft, json!([[1.0,0.0],[1.0,0.0],[1.0,0.0],[1.0,0.0]]));
    assert_eq!(engine::execute("signal.resample_linear", &[json!([0,10]),json!(3)]).unwrap(), json!([0.0,5.0,10.0]));
}

#[test]
fn golden_alpha8_numerical() {
    close(scalar("num.secant", &[json!({"e":"x^2-2"}),json!(1),json!(2)]), 2.0_f64.sqrt(), 1e-9);
    close(scalar("num.integrate_simpson_expr", &[json!({"e":"x^2"}),json!(0),json!(1),json!(100)]), 1.0/3.0, 1e-10);
    assert_eq!(engine::execute("num.linspace", &[json!(0),json!(1),json!(5)]).unwrap(), json!([0.0,0.25,0.5,0.75,1.0]));
}

#[test]
fn golden_alpha8_physics_engineering() {
    close(scalar("phys.kinetic_energy", &[json!(2),json!(3)]), 9.0, 1e-12);
    close(scalar("phys.projectile_range", &[json!(10),json!(std::f64::consts::FRAC_PI_4),json!(10)]), 10.0, 1e-12);
    close(scalar("eng.ohm_voltage", &[json!(2),json!(5)]), 10.0, 1e-12);
    close(scalar("eng.voltage_divider", &[json!(12),json!(1000),json!(1000)]), 6.0, 1e-12);
}

#[test]
fn golden_alpha9_discrete() {
    assert_eq!(engine::execute("disc.ncr", &[json!(10),json!(3)]).unwrap(), json!("120"));
    assert_eq!(engine::execute("disc.derangements", &[json!(6)]).unwrap(), json!("265"));
    assert_eq!(engine::execute("disc.stirling2", &[json!(5),json!(2)]).unwrap(), json!("15"));
    assert_eq!(engine::execute("disc.bell", &[json!(5)]).unwrap(), json!("52"));
    assert_eq!(engine::execute("disc.tribonacci", &[json!(10)]).unwrap(), json!("81"));
    assert_eq!(engine::execute("disc.digital_root", &[json!("123456")]).unwrap(), json!(3));
}

#[test]
fn golden_alpha9_chemistry() {
    close(scalar("chem.moles_from_mass", &[json!(18.01528),json!(18.01528)]),1.0,1e-12);
    close(scalar("chem.molarity", &[json!(2),json!(0.5)]),4.0,1e-12);
    close(scalar("chem.ph_from_h", &[json!(1e-7)]),7.0,1e-12);
    close(scalar("chem.henderson_hasselbalch", &[json!(4.76),json!(1),json!(1)]),4.76,1e-12);
    close(scalar("chem.freezing_point_depression", &[json!(2),json!(1.86),json!(0.5)]),1.86,1e-12);
    assert_eq!(engine::execute("chem.molarity", &[json!(1),json!(0)]),Err("DOMAIN"));
}

#[test]
fn golden_alpha9_networking() {
    assert_eq!(engine::execute("net.ipv4_to_u32", &[json!("192.168.1.1")]).unwrap(), json!(3232235777u64));
    assert_eq!(engine::execute("net.mask_from_prefix", &[json!(24)]).unwrap(), json!("255.255.255.0"));
    assert_eq!(engine::execute("net.network", &[json!("192.168.1.123"),json!(24)]).unwrap(), json!("192.168.1.0"));
    assert_eq!(engine::execute("net.broadcast", &[json!("192.168.1.123"),json!(24)]).unwrap(), json!("192.168.1.255"));
    assert_eq!(engine::execute("net.contains", &[json!("10.0.0.0/8"),json!("10.2.3.4")]).unwrap(), json!(true));
    close(scalar("net.serialization_delay", &[json!(1500),json!(1_000_000)]),0.012,1e-12);
}

#[test]
fn golden_alpha9_color() {
    assert_eq!(engine::execute("color.rgb_to_hex", &[json!([255,165,0])]).unwrap(), json!("#FFA500"));
    assert_eq!(engine::execute("color.hex_to_rgb", &[json!("#00FF7F")]).unwrap(), json!([0,255,127]));
    assert_eq!(engine::execute("color.rgb_to_hsv", &[json!([255,0,0])]).unwrap(), json!([0.0,1.0,1.0]));
    assert_eq!(engine::execute("color.hsv_to_rgb", &[json!([120,1,1])]).unwrap(), json!([0.0,255.0,0.0]));
    close(scalar("color.contrast_ratio", &[json!([0,0,0]),json!([255,255,255])]),21.0,1e-12);
    assert_eq!(engine::execute("color.invert", &[json!([10,20,30])]).unwrap(), json!([245.0,235.0,225.0]));
}

#[test]
fn golden_alpha9_information() {
    close(scalar("info.shannon_entropy", &[json!([0.5,0.5])]),1.0,1e-12);
    close(scalar("info.kl_divergence", &[json!([0.5,0.5]),json!([0.5,0.5])]),0.0,1e-12);
    close(scalar("info.js_divergence", &[json!([1.0,0.0]),json!([0.0,1.0])]),1.0,1e-12);
    assert_eq!(engine::execute("info.hamming_string", &[json!("kitten"),json!("sitten")]).unwrap(),json!(1));
    assert_eq!(engine::execute("info.levenshtein", &[json!("kitten"),json!("sitting")]).unwrap(),json!(3));
    close(scalar("info.nyquist_bitrate", &[json!(3000),json!(4)]),12000.0,1e-12);
}

#[test]
fn golden_alpha9_astronomy() {
    close(scalar("astro.escape_velocity", &[json!(5.9722e24),json!(6.371e6)]),11186.165197346223,1e-12);
    close(scalar("astro.surface_gravity", &[json!(5.9722e24),json!(6.371e6)]),9.820302293385645,1e-12);
    close(scalar("astro.schwarzschild_radius", &[json!(1.98847e30)]),2953.3393820668784,1e-12);
    close(scalar("astro.distance_modulus", &[json!(10)]),0.0,1e-12);
    close(scalar("astro.light_travel_time", &[json!(299792458)]),1.0,1e-12);
    close(scalar("astro.rocket_delta_v", &[json!(3000),json!(1000),json!(500)]),2079.441541679836,1e-12);
}

#[test]
fn golden_alpha10_time_and_geodesy() {
    assert_eq!(engine::execute("time.days_in_month", &[json!(2024), json!(2)]).unwrap(), json!(29));
    close(scalar("time.hz_to_period", &[json!(2)]), 0.5, 1e-12);
    close(scalar("geod.sphere_circumference", &[json!(1)]), std::f64::consts::TAU, 1e-12);
    close(scalar("geod.normalize_lon", &[json!(190)]), -170.0, 1e-12);
}

#[test]
fn golden_alpha10_thermo_mechanics_fluid() {
    close(scalar("thermo.carnot_efficiency", &[json!(400), json!(300)]), 0.25, 1e-12);
    close(scalar("thermo.celsius_to_kelvin", &[json!(0)]), 273.15, 1e-12);
    close(scalar("mech.kinetic_energy", &[json!(2), json!(3)]), 9.0, 1e-12);
    close(scalar("fluid.flow_rate", &[json!(2), json!(3)]), 6.0, 1e-12);
}

#[test]
fn golden_alpha10_electrical_optics_wave_data() {
    close(scalar("elec.voltage_divider", &[json!(12), json!(1000), json!(1000)]), 6.0, 1e-12);
    close(scalar("optics.lens_power_diopter", &[json!(0.5)]), 2.0, 1e-12);
    close(scalar("wave.beat_frequency", &[json!(440), json!(442)]), 2.0, 1e-12);
    assert_eq!(engine::execute("data.ceil_div", &[json!(10), json!(3)]).unwrap(), json!(4));
    assert_eq!(engine::execute("data.xor_checksum", &[json!([1,2,3])]).unwrap(), json!(0));
}


#[test]
fn golden_alpha11_verification() {
    assert_eq!(engine::execute("verify.near", &[json!(1.0),json!(1.0000001),json!(1e-6),json!(0)]).unwrap(), json!(true));
    assert_eq!(engine::execute("verify.monotonic_inc", &[json!([1,1,2,3])]).unwrap(), json!(true));
    close(scalar("verify.residual_rms", &[json!([3,4])]), (12.5_f64).sqrt(), 1e-12);
    let r=engine::execute("verify.convergence", &[json!([1.1,1.01,1.001]),json!(0.01),json!(0.01)]).unwrap();
    assert_eq!(r["ok"],json!(true));
}

#[test]
fn golden_alpha11_frames() {
    let v=engine::execute("frame.vec3", &[json!([1,2,3]),json!("A")]).unwrap();
    assert_eq!(v,json!({"f":"A","t":"v","v":[1.0,2.0,3.0]}));
    let t=engine::execute("frame.translate", &[json!([10,0,0]),json!("A"),json!("B")]).unwrap();
    let p=engine::execute("frame.point3", &[json!([1,2,3]),json!("A")]).unwrap();
    assert_eq!(engine::execute("frame.transform_point", &[t,p]).unwrap(),json!({"f":"B","t":"p","v":[11.0,2.0,3.0]}));
}

#[test]
fn golden_alpha11_curves() {
    let sq=json!([[0,0],[2,0],[2,2],[0,2],[0,0]]);
    close(scalar("curve.area", &[sq.clone()]),4.0,1e-12);
    assert_eq!(engine::execute("curve.is_simple_closed", &[sq.clone()]).unwrap(),json!(true));
    assert_eq!(engine::execute("curve.centroid", &[sq]).unwrap(),json!([1.0,1.0]));
}

#[test]
fn golden_alpha11_predicates() {
    let sq=json!([[0,0],[10,0],[10,10],[0,10],[0,0]]);
    assert_eq!(engine::execute("predicate.point_in_polygon", &[json!([5,5]),sq.clone()]).unwrap(),json!(true));
    assert_eq!(engine::execute("predicate.circle_in_polygon", &[json!([5,5]),json!(4),sq.clone()]).unwrap(),json!(true));
    close(scalar("predicate.point_polygon_clearance", &[json!([5,5]),sq]),5.0,1e-12);
}

#[test]
fn golden_alpha12_linalg() {
    assert_eq!(engine::execute("linalg.rank", &[json!([[1,2],[2,4]])]).unwrap(), json!(1));
    close(scalar("linalg.condition_number", &[json!([[3,0],[0,1]])]), 3.0, 1e-9);
    close(scalar("linalg.spectral_norm", &[json!([[3,0],[0,1]])]), 3.0, 1e-9);
}

#[test]
fn golden_alpha12_special() {
    close(scalar("special.gamma", &[json!(5)]), 24.0, 1e-9);
    close(scalar("special.zeta", &[json!(2)]), std::f64::consts::PI.powi(2)/6.0, 1e-8);
    close(scalar("special.sinc", &[json!(0)]), 1.0, 1e-12);
}

#[test]
fn golden_alpha12_optimize() {
    close(scalar("optimize.stationary_residual", &[json!({"e":"(x-2)^2"}), json!(2)]), 0.0, 1e-7);
    let r=engine::execute("optimize.audit_minimum", &[json!({"e":"(x-2)^2"}),json!(2)]).unwrap();
    assert_eq!(r["local_min"],json!(true));
}

#[test]
fn golden_alpha12_ode() {
    let r=engine::execute("ode.rk4", &[json!({"e":"0"}),json!(0),json!(3),json!(1),json!(10)]).unwrap();
    close(r["y"].as_f64().unwrap(),3.0,1e-12);
    close(scalar("ode.residual", &[json!({"e":"0"}),json!(0),json!(3),json!(3),json!(0.1)]),0.0,1e-12);
}

#[test]
fn golden_alpha12_series() {
    close(scalar("series.geometric_sum", &[json!(1),json!(0.5)]),2.0,1e-12);
    close(scalar("series.aitken", &[json!([1,1.5,1.75])]),2.0,1e-12);
    close(scalar("series.taylor_exp", &[json!(0),json!(10)]),1.0,1e-12);
}
