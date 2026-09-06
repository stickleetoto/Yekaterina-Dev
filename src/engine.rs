use serde_json::{Value, json};

use crate::{
    advanced_matrix, advanced_numerical, advanced_probability, advanced_signal, advanced_stats,
    algebra, complex_math, engineering, extra_math, geometry, matrix, numerical, physics,
    practical, precision, probability, radix, registry, signal, stats, vector, discrete, chemistry, networking, color, information, astronomy, time_ops, geodesy, thermodynamics, mechanics, fluids, electrical, optics, waves, data_ops, verification, frame, curve, predicate, deep_linalg, special_functions, optimization, ode, series, inference,
};

pub fn execute(opcode: &str, args: &[Value]) -> Result<Value, &'static str> {
    let spec = registry::resolve(opcode).ok_or("OP")?;

    if let Some(result) = precision::execute(spec.opcode, args) {
        return result;
    }
    if let Some(result) = dispatch_module(spec.opcode, args) {
        return result;
    }

    match spec.opcode {
        "math.add" => binary(args, |a, b| a + b),
        "math.sub" => binary(args, |a, b| a - b),
        "math.mul" => binary(args, |a, b| a * b),
        "math.div" => {
            let (a, b) = two(args)?;
            if b == 0.0 { return Err("DIV0"); }
            finite(a / b)
        }
        "math.mod" => {
            let (a, b) = two(args)?;
            if b == 0.0 { return Err("DIV0"); }
            finite(a % b)
        }
        "math.pow" => binary(args, |a, b| a.powf(b)),
        "math.sqrt" => {
            let x = one(args)?;
            if x < 0.0 { return Err("DOMAIN"); }
            finite(x.sqrt())
        }
        "math.abs" => finite(one(args)?.abs()),
        "math.round" => {
            if args.is_empty() || args.len() > 2 { return Err("ARG"); }
            let x = number(&args[0])?;
            let digits = if args.len() == 2 {
                args[1].as_i64().ok_or("TYPE")?.clamp(-12, 12) as i32
            } else { 0 };
            let factor = 10_f64.powi(digits);
            finite((x * factor).round() / factor)
        }
        "math.floor" => finite(one(args)?.floor()),
        "math.ceil" => finite(one(args)?.ceil()),
        "math.min" => binary(args, f64::min),
        "math.max" => binary(args, f64::max),
        "math.clamp" => {
            if args.len() != 3 { return Err("ARG"); }
            let x = number(&args[0])?;
            let lo = number(&args[1])?;
            let hi = number(&args[2])?;
            if lo > hi { return Err("DOMAIN"); }
            finite(x.clamp(lo, hi))
        }
        "math.exp" => finite(one(args)?.exp()),
        "math.ln" => {
            let x = one(args)?;
            if x <= 0.0 { return Err("DOMAIN"); }
            finite(x.ln())
        }
        "math.log10" => {
            let x = one(args)?;
            if x <= 0.0 { return Err("DOMAIN"); }
            finite(x.log10())
        }
        "math.sin" => finite(one(args)?.sin()),
        "math.cos" => finite(one(args)?.cos()),
        "math.tan" => finite(one(args)?.tan()),
        "stat.sum" => {
            let xs = array(args)?;
            finite(xs.iter().sum())
        }
        "stat.mean" => {
            let xs = array(args)?;
            if xs.is_empty() { return Err("EMPTY"); }
            finite(xs.iter().sum::<f64>() / xs.len() as f64)
        }
        "stat.min" => {
            let xs = array(args)?;
            xs.into_iter().reduce(f64::min).map_or(Err("EMPTY"), finite)
        }
        "stat.max" => {
            let xs = array(args)?;
            xs.into_iter().reduce(f64::max).map_or(Err("EMPTY"), finite)
        }
        "stat.count" => {
            if args.len() != 1 { return Err("ARG"); }
            let xs = args[0].as_array().ok_or("TYPE")?;
            Ok(json!(xs.len()))
        }
        "stat.variance" => variance(args),
        "stat.std" => {
            let v = variance_value(args)?;
            finite(v.sqrt())
        }
        "stat.median" => {
            let mut xs = array(args)?;
            if xs.is_empty() { return Err("EMPTY"); }
            xs.sort_by(f64::total_cmp);
            let n = xs.len();
            if n % 2 == 1 { finite(xs[n / 2]) }
            else { finite((xs[n / 2 - 1] + xs[n / 2]) / 2.0) }
        }
        "udo.formula" | "udo.remove" => Err("CONTROL"),
        _ => Err("NYI"),
    }
}

fn dispatch_module(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    let family = op.split_once('.').map(|(p, _)| p).unwrap_or(op);
    match family {
        "math" => extra_math::execute(op, args),
        // inference is asked first because advanced_stats claims the whole
        // "test." and "reg." prefixes and would answer OP for anything it does
        // not itself implement, which would swallow every operation added to
        // those families afterwards. inference matches an explicit list, so
        // going first cannot shadow an existing operation.
        "stat" | "reg" | "test" => inference::execute(op, args)
            .or_else(|| stats::execute(op, args))
            .or_else(|| advanced_stats::execute(op, args)),
        "vec" => vector::execute(op, args),
        "mat" => matrix::execute(op, args).or_else(|| advanced_matrix::execute(op, args)),
        "geo" => geometry::execute(op, args),
        "pct" | "fin" | "unit" => practical::execute(op, args),
        "signal" => signal::execute(op, args).or_else(|| advanced_signal::execute(op, args)),
        "prob" => probability::execute(op, args).or_else(|| advanced_probability::execute(op, args)),
        "num" => numerical::execute(op, args).or_else(|| advanced_numerical::execute(op, args)),
        "bit" | "base" => radix::execute(op, args),
        "alg" => algebra::execute(op, args),
        "cplx" => complex_math::execute(op, args),
        "phys" => physics::execute(op, args),
        "eng" => engineering::execute(op, args),
        "disc" => discrete::execute(op, args),
        "chem" => chemistry::execute(op, args),
        "net" => networking::execute(op, args),
        "color" => color::execute(op, args),
        "info" => information::execute(op, args),
        "astro" => astronomy::execute(op, args),
        "time" => time_ops::execute(op, args),
        "geod" => geodesy::execute(op, args),
        "thermo" => thermodynamics::execute(op, args),
        "mech" => mechanics::execute(op, args),
        "fluid" => fluids::execute(op, args),
        "elec" => electrical::execute(op, args),
        "optics" => optics::execute(op, args),
        "wave" => waves::execute(op, args),
        "data" => data_ops::execute(op, args),
        "verify" => verification::execute(op, args),
        "frame" => frame::execute(op, args),
        "curve" => curve::execute(op, args),
        "predicate" => predicate::execute(op, args),
        "linalg" => deep_linalg::execute(op, args),
        "special" => special_functions::execute(op, args),
        "optimize" => optimization::execute(op, args),
        "ode" => ode::execute(op, args),
        "series" => series::execute(op, args),
        _ => None,
    }
}

fn number(v: &Value) -> Result<f64, &'static str> { v.as_f64().ok_or("TYPE") }

fn one(args: &[Value]) -> Result<f64, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    number(&args[0])
}

fn two(args: &[Value]) -> Result<(f64, f64), &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    Ok((number(&args[0])?, number(&args[1])?))
}

fn binary<F>(args: &[Value], f: F) -> Result<Value, &'static str>
where
    F: FnOnce(f64, f64) -> f64,
{
    let (a, b) = two(args)?;
    finite(f(a, b))
}

fn array(args: &[Value]) -> Result<Vec<f64>, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let values = args[0].as_array().ok_or("TYPE")?;
    if values.len() > 100_000 { return Err("LIMIT"); }
    values.iter().map(number).collect()
}

fn variance(args: &[Value]) -> Result<Value, &'static str> { finite(variance_value(args)?) }

fn variance_value(args: &[Value]) -> Result<f64, &'static str> {
    let xs = array(args)?;
    if xs.is_empty() { return Err("EMPTY"); }
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|x| {
        let d = *x - mean;
        d * d
    }).sum::<f64>() / xs.len() as f64;
    if !var.is_finite() { return Err("NONFINITE"); }
    Ok(var)
}

fn finite(x: f64) -> Result<Value, &'static str> {
    if !x.is_finite() { return Err("NONFINITE"); }
    Ok(json!(x))
}
