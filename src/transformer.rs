use serde_json::{Value, json};

const MU0: f64 = 1.256_637_061_435_917_3e-6;
const MAX_AXIS: usize = 4096;

/// Native transformer-design calculations that require arrays, interpolation,
/// integer winding geometry, or structured results. This module is deliberately
/// not registered in the frozen v1.2 built-in registry yet; v1.3 promotes the
/// contract only after these calculations are independently verified.
pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if !op.starts_with("xfmr.") {
        return None;
    }
    Some(run(op, args))
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "xfmr.bh_field_strength" => bh_field_strength_op(args),
        "xfmr.magnetizing_current_bh" => magnetizing_current_bh(args),
        "xfmr.core_loss_specific_interp" => core_loss_specific_interp(args),
        "xfmr.core_loss_from_grid" => core_loss_from_grid(args),
        "xfmr.winding_geometry" => winding_geometry(args),
        "xfmr.skin_depth" => skin_depth(args),
        "xfmr.thermal_two_node" => thermal_two_node(args),
        _ => Err("OP"),
    }
}

fn bh_field_strength_op(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let curve = pairs(&args[0])?;
    let b = num(&args[1])?;
    finite(linear_interp_pairs(&curve, b)?)
}

fn magnetizing_current_bh(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 4)?;
    let curve = pairs(&args[0])?;
    let b = num(&args[1])?;
    let path_m = positive(num(&args[2])?)?;
    let turns = positive(num(&args[3])?)?;
    let h = linear_interp_pairs(&curve, b)?;
    finite(h * path_m / turns)
}

fn core_loss_specific_interp(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 5)?;
    let freq = axis(&args[0])?;
    let flux = axis(&args[1])?;
    let grid = matrix(&args[2], freq.len(), flux.len())?;
    let f = positive(num(&args[3])?)?;
    let b = nonneg(num(&args[4])?)?;
    finite(bilinear(&freq, &flux, &grid, f, b)?)
}

fn core_loss_from_grid(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 6)?;
    let freq = axis(&args[0])?;
    let flux = axis(&args[1])?;
    let grid = matrix(&args[2], freq.len(), flux.len())?;
    let f = positive(num(&args[3])?)?;
    let b = nonneg(num(&args[4])?)?;
    let mass_kg = nonneg(num(&args[5])?)?;
    let specific_w_kg = bilinear(&freq, &flux, &grid, f, b)?;
    finite(specific_w_kg * mass_kg)
}

/// Arguments:
/// turns, max_turns_per_layer, insulated_radial_mm, axial_pitch_mm,
/// interlayer_insulation_mm, axial_margin_each_end_mm.
///
/// Returns integer layer occupancy plus preliminary radial/axial winding build.
fn winding_geometry(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 6)?;
    let turns = positive_integer(&args[0])?;
    let capacity = positive_integer(&args[1])?;
    let radial_mm = positive(num(&args[2])?)?;
    let axial_pitch_mm = positive(num(&args[3])?)?;
    let interlayer_mm = nonneg(num(&args[4])?)?;
    let end_margin_mm = nonneg(num(&args[5])?)?;

    let layers = turns.div_ceil(capacity);
    let last_layer_turns = turns - (layers - 1) * capacity;
    let radial_build_mm = layers as f64 * radial_mm
        + layers.saturating_sub(1) as f64 * interlayer_mm;
    let axial_height_mm = capacity.min(turns) as f64 * axial_pitch_mm
        + 2.0 * end_margin_mm;

    if !radial_build_mm.is_finite() || !axial_height_mm.is_finite() {
        return Err("NONFINITE");
    }
    Ok(json!({
        "layers": layers,
        "last_layer_turns": last_layer_turns,
        "radial_build_mm": radial_build_mm,
        "axial_height_mm": axial_height_mm
    }))
}

/// Classical skin depth delta = sqrt(rho / (pi f mu)).
/// `relative_permeability` should normally be approximately 1 for copper or
/// aluminium conductors; it is explicit so the equation never hides material
/// assumptions.
fn skin_depth(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let frequency_hz = positive(num(&args[0])?)?;
    let resistivity_ohm_m = positive(num(&args[1])?)?;
    let relative_permeability = positive(num(&args[2])?)?;
    finite((resistivity_ohm_m / (std::f64::consts::PI * frequency_hz * MU0 * relative_permeability)).sqrt())
}

/// Two-source, one-coolant-node steady-state thermal network.
///
/// Arguments:
/// winding_loss_w, core_loss_w, winding_to_oil_k_per_w,
/// core_to_oil_k_per_w, oil_to_ambient_k_per_w, ambient_c.
///
/// The caller supplies thermal resistances from geometry/test/CFD. The function
/// does not pretend one universal transformer thermal resistance exists.
fn thermal_two_node(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 6)?;
    let winding_loss = nonneg(num(&args[0])?)?;
    let core_loss = nonneg(num(&args[1])?)?;
    let r_wo = nonneg(num(&args[2])?)?;
    let r_co = nonneg(num(&args[3])?)?;
    let r_oa = nonneg(num(&args[4])?)?;
    let ambient = num(&args[5])?;

    let oil_rise = (winding_loss + core_loss) * r_oa;
    let oil_c = ambient + oil_rise;
    let winding_c = oil_c + winding_loss * r_wo;
    let core_c = oil_c + core_loss * r_co;
    if !oil_c.is_finite() || !winding_c.is_finite() || !core_c.is_finite() {
        return Err("NONFINITE");
    }
    Ok(json!({
        "oil_c": oil_c,
        "winding_c": winding_c,
        "core_c": core_c,
        "oil_rise_k": oil_rise,
        "winding_rise_k": winding_c - ambient,
        "core_rise_k": core_c - ambient
    }))
}

fn bilinear(
    x_axis: &[f64],
    y_axis: &[f64],
    grid: &[Vec<f64>],
    x: f64,
    y: f64,
) -> Result<f64, &'static str> {
    let (xi0, xi1, tx) = bracket(x_axis, x)?;
    let (yi0, yi1, ty) = bracket(y_axis, y)?;

    let q00 = grid[xi0][yi0];
    let q10 = grid[xi1][yi0];
    let q01 = grid[xi0][yi1];
    let q11 = grid[xi1][yi1];

    let low = q00 + (q10 - q00) * tx;
    let high = q01 + (q11 - q01) * tx;
    let out = low + (high - low) * ty;
    if out.is_finite() { Ok(out) } else { Err("NONFINITE") }
}

fn linear_interp_pairs(points: &[(f64, f64)], x: f64) -> Result<f64, &'static str> {
    if x < points[0].0 || x > points[points.len() - 1].0 {
        return Err("DOMAIN");
    }
    match points.binary_search_by(|p| p.0.total_cmp(&x)) {
        Ok(i) => Ok(points[i].1),
        Err(i) => {
            if i == 0 || i >= points.len() { return Err("DOMAIN"); }
            let (x0, y0) = points[i - 1];
            let (x1, y1) = points[i];
            let t = (x - x0) / (x1 - x0);
            let out = y0 + (y1 - y0) * t;
            if out.is_finite() { Ok(out) } else { Err("NONFINITE") }
        }
    }
}

fn bracket(axis: &[f64], x: f64) -> Result<(usize, usize, f64), &'static str> {
    if x < axis[0] || x > axis[axis.len() - 1] {
        return Err("DOMAIN");
    }
    match axis.binary_search_by(|v| v.total_cmp(&x)) {
        Ok(i) => Ok((i, i, 0.0)),
        Err(i) => {
            if i == 0 || i >= axis.len() { return Err("DOMAIN"); }
            let lo = i - 1;
            let hi = i;
            Ok((lo, hi, (x - axis[lo]) / (axis[hi] - axis[lo])))
        }
    }
}

fn pairs(v: &Value) -> Result<Vec<(f64, f64)>, &'static str> {
    let rows = v.as_array().ok_or("TYPE")?;
    if rows.len() < 2 { return Err("ARG"); }
    if rows.len() > MAX_AXIS { return Err("LIMIT"); }
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let pair = row.as_array().ok_or("TYPE")?;
        if pair.len() != 2 { return Err("ARG"); }
        let x = num(&pair[0])?;
        let y = num(&pair[1])?;
        if !x.is_finite() || !y.is_finite() { return Err("NONFINITE"); }
        out.push((x, y));
    }
    if out.windows(2).any(|w| w[0].0 >= w[1].0) { return Err("DOMAIN"); }
    Ok(out)
}

fn axis(v: &Value) -> Result<Vec<f64>, &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.len() < 2 { return Err("ARG"); }
    if xs.len() > MAX_AXIS { return Err("LIMIT"); }
    let out: Vec<f64> = xs.iter().map(num).collect::<Result<_, _>>()?;
    if out.windows(2).any(|w| w[0] >= w[1]) { return Err("DOMAIN"); }
    Ok(out)
}

fn matrix(v: &Value, rows: usize, cols: usize) -> Result<Vec<Vec<f64>>, &'static str> {
    let outer = v.as_array().ok_or("TYPE")?;
    if outer.len() != rows { return Err("ARG"); }
    let mut out = Vec::with_capacity(rows);
    for row in outer {
        let xs = row.as_array().ok_or("TYPE")?;
        if xs.len() != cols { return Err("ARG"); }
        let values: Vec<f64> = xs.iter().map(num).collect::<Result<_, _>>()?;
        if values.iter().any(|x| *x < 0.0) { return Err("DOMAIN"); }
        out.push(values);
    }
    Ok(out)
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
