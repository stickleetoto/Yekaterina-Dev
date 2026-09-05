use serde_json::{Value, json};

const MAX_VALUES: usize = 100_000;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    Some(match op {
        "verify.finite" => finite_op(args),
        "verify.finite_all" => finite_all_op(args),
        "verify.near" => near(args),
        "verify.abs_change" => binary(args, |a, b| (b - a).abs()),
        "verify.rel_change" => rel_change_op(args),
        "verify.monotonic_inc" => monotonic_op(args, true),
        "verify.monotonic_dec" => monotonic_op(args, false),
        "verify.bounded" => bounded(args),
        "verify.residual_l2" => residual_l2(args),
        "verify.residual_rms" => residual_rms(args),
        "verify.residual_max" => residual_max(args),
        "verify.in_range" => in_range(args),
        "verify.positive" => positive_op(args),
        "verify.nonnegative" => nonnegative_op(args),
        "verify.nonzero" => nonzero(args),
        "verify.percent_error" => percent_error(args),
        "verify.stable_digits" => stable_digits(args),
        "verify.sign_consistent" => sign_consistent(args),
        "verify.shape_equal" => shape_equal(args),
        "verify.convergence" => convergence(args),
        "verify.grid_convergence" => grid_convergence(args),
        "verify.condition_ok" => condition_ok(args),
        "verify.sequence_audit" => sequence_audit(args),
        _ => return None,
    })
}


fn finite_op(args: &[Value]) -> Result<Value, &'static str> { Ok(json!(one(args)?.is_finite())) }
fn finite_all_op(args: &[Value]) -> Result<Value, &'static str> { Ok(json!(array(args)?.iter().all(|x| x.is_finite()))) }
fn rel_change_op(args: &[Value]) -> Result<Value, &'static str> { let (a,b)=two(args)?; Ok(json!(relative_change(a,b))) }
fn monotonic_op(args: &[Value], inc: bool) -> Result<Value, &'static str> { Ok(json!(monotonic(args, inc)?)) }
fn positive_op(args: &[Value]) -> Result<Value, &'static str> { Ok(json!(one(args)? > 0.0)) }
fn nonnegative_op(args: &[Value]) -> Result<Value, &'static str> { Ok(json!(one(args)? >= 0.0)) }

fn number(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if !x.is_finite() { return Err("NONFINITE"); }
    Ok(x)
}

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
    let out = f(a, b);
    if !out.is_finite() { return Err("NONFINITE"); }
    Ok(json!(out))
}

fn vec_from(v: &Value) -> Result<Vec<f64>, &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.len() > MAX_VALUES { return Err("LIMIT"); }
    xs.iter().map(number).collect()
}

fn array(args: &[Value]) -> Result<Vec<f64>, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    vec_from(&args[0])
}

fn tolerances(args: &[Value], start: usize) -> Result<(f64, f64), &'static str> {
    let abs_tol = if args.len() > start { number(&args[start])? } else { 1e-9 };
    let rel_tol = if args.len() > start + 1 { number(&args[start + 1])? } else { 1e-6 };
    if abs_tol < 0.0 || rel_tol < 0.0 { return Err("DOMAIN"); }
    Ok((abs_tol, rel_tol))
}

fn relative_change(a: f64, b: f64) -> f64 {
    let scale = a.abs().max(b.abs());
    if scale == 0.0 { 0.0 } else { (b - a).abs() / scale }
}

fn near(args: &[Value]) -> Result<Value, &'static str> {
    if !(2..=4).contains(&args.len()) { return Err("ARG"); }
    let a = number(&args[0])?;
    let b = number(&args[1])?;
    let (abs_tol, rel_tol) = tolerances(args, 2)?;
    let delta = (a - b).abs();
    let threshold = abs_tol.max(rel_tol * a.abs().max(b.abs()));
    Ok(json!(delta <= threshold))
}

fn monotonic(args: &[Value], inc: bool) -> Result<bool, &'static str> {
    let xs = array(args)?;
    Ok(xs.windows(2).all(|w| if inc { w[1] >= w[0] } else { w[1] <= w[0] }))
}

fn bounded(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 3 { return Err("ARG"); }
    let xs = vec_from(&args[0])?;
    let lo = number(&args[1])?;
    let hi = number(&args[2])?;
    if lo > hi { return Err("DOMAIN"); }
    Ok(json!(xs.iter().all(|x| *x >= lo && *x <= hi)))
}

fn residual_l2(args: &[Value]) -> Result<Value, &'static str> {
    let xs = array(args)?;
    let sum = xs.iter().map(|x| x * x).sum::<f64>();
    let out = sum.sqrt();
    if !out.is_finite() { return Err("NONFINITE"); }
    Ok(json!(out))
}

fn residual_rms(args: &[Value]) -> Result<Value, &'static str> {
    let xs = array(args)?;
    if xs.is_empty() { return Err("EMPTY"); }
    let out = (xs.iter().map(|x| x * x).sum::<f64>() / xs.len() as f64).sqrt();
    if !out.is_finite() { return Err("NONFINITE"); }
    Ok(json!(out))
}

fn residual_max(args: &[Value]) -> Result<Value, &'static str> {
    let xs = array(args)?;
    let Some(v) = xs.iter().map(|x| x.abs()).reduce(f64::max) else { return Err("EMPTY"); };
    Ok(json!(v))
}

fn in_range(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 3 { return Err("ARG"); }
    let x = number(&args[0])?;
    let lo = number(&args[1])?;
    let hi = number(&args[2])?;
    if lo > hi { return Err("DOMAIN"); }
    Ok(json!(x >= lo && x <= hi))
}

fn nonzero(args: &[Value]) -> Result<Value, &'static str> {
    if !(1..=2).contains(&args.len()) { return Err("ARG"); }
    let x = number(&args[0])?;
    let tol = if args.len() == 2 { number(&args[1])? } else { 0.0 };
    if tol < 0.0 { return Err("DOMAIN"); }
    Ok(json!(x.abs() > tol))
}

fn percent_error(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let x = number(&args[0])?;
    let reference = number(&args[1])?;
    if reference == 0.0 { return Err("DIV0"); }
    Ok(json!(((x - reference).abs() / reference.abs()) * 100.0))
}

fn stable_digits(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two(args)?;
    let rel = relative_change(a, b);
    let digits = if rel == 0.0 { 15 } else { (-rel.log10()).floor().clamp(0.0, 15.0) as u64 };
    Ok(json!(digits))
}

fn sign_consistent(args: &[Value]) -> Result<Value, &'static str> {
    let xs = array(args)?;
    let mut sign = 0i8;
    for x in xs {
        if x == 0.0 { continue; }
        let s = if x > 0.0 { 1 } else { -1 };
        if sign == 0 { sign = s; }
        else if sign != s { return Ok(json!(false)); }
    }
    Ok(json!(true))
}

fn shape(v: &Value) -> Result<Vec<usize>, &'static str> {
    fn inner(v: &Value, out: &mut Vec<usize>, depth: usize) -> Result<(), &'static str> {
        if depth > 8 { return Err("LIMIT"); }
        match v {
            Value::Array(xs) => {
                out.push(xs.len());
                if let Some(first) = xs.first() {
                    let mut child = Vec::new();
                    inner(first, &mut child, depth + 1)?;
                    for x in xs.iter().skip(1) {
                        let mut s = Vec::new();
                        inner(x, &mut s, depth + 1)?;
                        if s != child { return Err("SHAPE"); }
                    }
                    out.extend(child);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    let mut out = Vec::new();
    inner(v, &mut out, 0)?;
    Ok(out)
}

fn shape_equal(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    Ok(json!(shape(&args[0])? == shape(&args[1])?))
}

fn convergence(args: &[Value]) -> Result<Value, &'static str> {
    if args.is_empty() || args.len() > 3 { return Err("ARG"); }
    let xs = vec_from(&args[0])?;
    if xs.len() < 2 { return Err("ARG"); }
    let (abs_tol, rel_tol) = tolerances(args, 1)?;
    let n = xs.len();
    let da = (xs[n - 1] - xs[n - 2]).abs();
    let dr = relative_change(xs[n - 2], xs[n - 1]);
    let ok = da <= abs_tol.max(rel_tol * xs[n - 2].abs().max(xs[n - 1].abs()));
    let p = if n >= 3 {
        let d0 = (xs[n - 2] - xs[n - 3]).abs();
        if d0 > 0.0 && da > 0.0 { Some((d0 / da).log2()) } else { None }
    } else { None };
    Ok(json!({"ok":ok,"v":xs[n-1],"da":da,"dr":dr,"p":p}))
}

fn grid_convergence(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() < 2 || args.len() > 4 { return Err("ARG"); }
    let xs = vec_from(&args[0])?;
    let ns = vec_from(&args[1])?;
    if xs.len() != ns.len() || xs.len() < 3 { return Err("SHAPE"); }
    if ns.iter().any(|n| *n <= 0.0) || !ns.windows(2).all(|w| w[1] > w[0]) { return Err("DOMAIN"); }
    let (abs_tol, rel_tol) = tolerances(args, 2)?;
    let n = xs.len();
    let d0 = (xs[n - 2] - xs[n - 3]).abs();
    let d1 = (xs[n - 1] - xs[n - 2]).abs();
    let r0 = ns[n - 2] / ns[n - 3];
    let r1 = ns[n - 1] / ns[n - 2];
    let ratio_consistent = ((r1 - r0).abs() / r0.abs().max(r1.abs())) <= 0.05;
    let p = if ratio_consistent && d0 > 0.0 && d1 > 0.0 && r1 > 1.0 {
        Some((d0 / d1).ln() / r1.ln())
    } else { None };
    let dr = relative_change(xs[n - 2], xs[n - 1]);
    let ok = d1 <= abs_tol.max(rel_tol * xs[n - 2].abs().max(xs[n - 1].abs()));
    Ok(json!({"ok":ok,"v":xs[n-1],"da":d1,"dr":dr,"p":p,"r":r1}))
}

fn condition_ok(args: &[Value]) -> Result<Value, &'static str> {
    if !(1..=2).contains(&args.len()) { return Err("ARG"); }
    let kappa = number(&args[0])?;
    let threshold = if args.len() == 2 { number(&args[1])? } else { 1e8 };
    if kappa < 0.0 || threshold <= 0.0 { return Err("DOMAIN"); }
    Ok(json!(kappa <= threshold))
}

fn sequence_audit(args: &[Value]) -> Result<Value, &'static str> {
    if args.is_empty() || args.len() > 3 { return Err("ARG"); }
    let xs = vec_from(&args[0])?;
    if xs.len() < 2 { return Err("ARG"); }
    let (abs_tol, rel_tol) = tolerances(args, 1)?;
    let finite = xs.iter().all(|x| x.is_finite());
    let inc = xs.windows(2).all(|w| w[1] >= w[0]);
    let dec = xs.windows(2).all(|w| w[1] <= w[0]);
    let n = xs.len();
    let da = (xs[n-1] - xs[n-2]).abs();
    let dr = relative_change(xs[n-2], xs[n-1]);
    let converged = da <= abs_tol.max(rel_tol * xs[n-2].abs().max(xs[n-1].abs()));
    Ok(json!({"f":finite,"m":inc||dec,"c":converged,"da":da,"dr":dr}))
}
