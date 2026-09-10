use serde_json::{Value, json};

const MU0: f64 = 1.256_637_061_435_917_3e-6;
const MAX_COMPONENTS: usize = 4096;

/// Native winding-field calculations for preliminary transformer design.
///
/// These models are deliberately explicit about their geometry assumptions.
/// They are not a replacement for FEM, factory short-circuit tests, or a
/// manufacturer-calibrated stray-loss model.
pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if !op.starts_with("xfmr.") {
        return None;
    }
    Some(run(op, args))
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "xfmr.rogowski_factor" => rogowski_factor_op(args),
        "xfmr.effective_leakage_height" => effective_leakage_height(args),
        "xfmr.leakage_inductance_concentric" => leakage_inductance_concentric(args),
        "xfmr.leakage_reactance_concentric" => leakage_reactance_concentric(args),
        "xfmr.dowell_foil_ac_factor" => dowell_foil_ac_factor_op(args),
        "xfmr.harmonic_copper_loss" => harmonic_copper_loss(args),
        _ => Err("OP"),
    }
}

/// Rogowski fringing factor for two equal-height concentric windings.
///
/// K = 1 - (1 - exp(-pi*h/w)) / (pi*h/w)
/// where h is physical winding height and w is total radial width occupied by
/// inner winding + main duct + outer winding.
fn rogowski_factor_op(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let h = positive(num(&args[0])?)?;
    let w = positive(num(&args[1])?)?;
    finite(rogowski_factor(h, w)?)
}

fn effective_leakage_height(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let h = positive(num(&args[0])?)?;
    let w = positive(num(&args[1])?)?;
    let k = rogowski_factor(h, w)?;
    finite(h / k)
}

/// Preliminary short-circuit leakage inductance for a pair of equal-height,
/// cylindrical, concentric windings, referred to the actual integer turn count
/// of the reference winding.
///
/// Arguments:
/// turns_reference,
/// winding_height_m,
/// inner_radial_build_m, inner_mean_diameter_m,
/// main_duct_radial_m, main_duct_mean_diameter_m,
/// outer_radial_build_m, outer_mean_diameter_m.
///
/// L_sigma = mu0*pi*N^2/H_eq *
///           (T1*D1/3 + Tg*Dg + T2*D2/3)
/// H_eq = H/K_R.
fn leakage_inductance_concentric(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 8)?;
    let turns = positive_integer(&args[0])? as f64;
    let h = positive(num(&args[1])?)?;
    let t1 = positive(num(&args[2])?)?;
    let d1 = positive(num(&args[3])?)?;
    let tg = nonneg(num(&args[4])?)?;
    let dg = positive(num(&args[5])?)?;
    let t2 = positive(num(&args[6])?)?;
    let d2 = positive(num(&args[7])?)?;

    let total_radial = t1 + tg + t2;
    let k = rogowski_factor(h, total_radial)?;
    let h_eq = h / k;
    let atd = t1 * d1 / 3.0 + tg * dg + t2 * d2 / 3.0;
    let inductance_h = MU0 * std::f64::consts::PI * turns * turns * atd / h_eq;
    finite(inductance_h)
}

fn leakage_reactance_concentric(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 9)?;
    let frequency_hz = positive(num(&args[0])?)?;
    let l = leakage_inductance_concentric(&args[1..])?;
    let inductance_h = l.as_f64().ok_or("TYPE")?;
    finite(std::f64::consts::TAU * frequency_hz * inductance_h)
}

/// Dowell one-dimensional full-winding AC/DC resistance factor for rectangular
/// foil/layer windings.
///
/// Arguments: conductor_thickness_m, skin_depth_m, layer_count.
/// `layer_count` is the number of complete effective layers in the MMF section.
/// This model assumes the classic 1D Dowell geometry; round-wire, CTC, partial
/// layers, interleaving, axial ducts, transposition and 2D fringing need a more
/// specialized model.
fn dowell_foil_ac_factor_op(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let thickness = positive(num(&args[0])?)?;
    let skin_depth = positive(num(&args[1])?)?;
    let layers = positive_integer(&args[2])?;
    finite(dowell_foil_ac_factor(thickness / skin_depth, layers)?)
}

/// Copper loss for a harmonic decomposition when each harmonic already has an
/// independently calculated R_ac/R_dc factor.
///
/// Arguments: hot_dc_resistance_ohm, currents_rms_a[], ac_factors[].
/// Result = sum(I_h^2 * R_dc_hot * F_h).
fn harmonic_copper_loss(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let rdc = nonneg(num(&args[0])?)?;
    let currents = nonnegative_array(&args[1])?;
    let factors = ac_factor_array(&args[2])?;
    if currents.len() != factors.len() {
        return Err("ARG");
    }

    let mut loss = 0.0;
    for (current, factor) in currents.into_iter().zip(factors) {
        loss += current * current * rdc * factor;
        if !loss.is_finite() {
            return Err("NONFINITE");
        }
    }
    finite(loss)
}

fn rogowski_factor(height_m: f64, total_radial_width_m: f64) -> Result<f64, &'static str> {
    let ratio = std::f64::consts::PI * height_m / total_radial_width_m;
    // exp_m1 keeps 1-exp(-ratio) accurate even for very small ratios.
    let k = 1.0 + (-ratio).exp_m1() / ratio;
    if k > 0.0 && k <= 1.0 && k.is_finite() {
        Ok(k)
    } else {
        Err("NONFINITE")
    }
}

fn dowell_foil_ac_factor(x: f64, layers: usize) -> Result<f64, &'static str> {
    if !x.is_finite() || x <= 0.0 || layers == 0 {
        return Err("DOMAIN");
    }
    let m = layers as f64;

    // Direct hyperbolic evaluation loses significance near zero. The first
    // non-zero correction of the Dowell expansion is O(x^4).
    let factor = if x < 1.0e-3 {
        1.0 + (5.0 * m * m - 1.0) * x.powi(4) / 45.0
    } else if x > 40.0 {
        // sinh/cosh overflow eventually, while both ratios have already reached
        // their asymptotic limit of one to floating-point precision.
        x * (2.0 * m * m + 1.0) / 3.0
    } else {
        let skin = x * ((2.0 * x).sinh() + (2.0 * x).sin())
            / ((2.0 * x).cosh() - (2.0 * x).cos());
        let proximity = (2.0 / 3.0) * (m * m - 1.0) * x
            * (x.sinh() - x.sin()) / (x.cosh() + x.cos());
        skin + proximity
    };

    if factor.is_finite() && factor >= 1.0 {
        Ok(factor)
    } else {
        Err("NONFINITE")
    }
}

fn nonnegative_array(v: &Value) -> Result<Vec<f64>, &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.is_empty() { return Err("EMPTY"); }
    if xs.len() > MAX_COMPONENTS { return Err("LIMIT"); }
    xs.iter()
        .map(|v| nonneg(num(v)?))
        .collect()
}

fn ac_factor_array(v: &Value) -> Result<Vec<f64>, &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.is_empty() { return Err("EMPTY"); }
    if xs.len() > MAX_COMPONENTS { return Err("LIMIT"); }
    xs.iter()
        .map(|v| {
            let x = num(v)?;
            if x >= 1.0 { Ok(x) } else { Err("DOMAIN") }
        })
        .collect()
}

fn need(args: &[Value], n: usize) -> Result<(), &'static str> {
    if args.len() == n { Ok(()) } else { Err("ARG") }
}

fn num(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if x.is_finite() { Ok(x) } else { Err("NONFINITE") }
}

fn positive(x: f64) -> Result<f64, &'static str> {
    if x > 0.0 { Ok(x) } else { Err("DOMAIN") }
}

fn nonneg(x: f64) -> Result<f64, &'static str> {
    if x >= 0.0 { Ok(x) } else { Err("DOMAIN") }
}

fn positive_integer(v: &Value) -> Result<usize, &'static str> {
    let n = v.as_u64().ok_or("TYPE")?;
    if n == 0 || n > usize::MAX as u64 { return Err("DOMAIN"); }
    Ok(n as usize)
}

fn finite(x: f64) -> Result<Value, &'static str> {
    if x.is_finite() { Ok(json!(x)) } else { Err("NONFINITE") }
}
