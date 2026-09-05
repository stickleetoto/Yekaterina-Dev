use serde_json::{json, Value};
use std::f64::consts::{E, PI};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if op.starts_with("special.") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "special.gamma" => unary(args, gamma),
        "special.log_gamma" => unary(args, log_gamma),
        "special.beta" => beta(args, false),
        "special.log_beta" => beta(args, true),
        "special.erf" => unary(args, libm::erf),
        "special.erfc" => unary(args, libm::erfc),
        "special.digamma" => unary(args, digamma),
        "special.trigamma" => unary(args, trigamma),
        "special.bessel_j0" => unary(args, |x| libm::j0(x)),
        "special.bessel_j1" => unary(args, |x| libm::j1(x)),
        "special.bessel_jn" => jn(args),
        "special.bessel_i0" => unary(args, |x| bessel_in(0, x)),
        "special.bessel_i1" => unary(args, |x| bessel_in(1, x)),
        "special.zeta" => unary(args, zeta),
        "special.eta" => unary(args, eta),
        "special.lambert_w0" => unary(args, |x| lambert_w(x, 0)),
        "special.lambert_wm1" => unary(args, |x| lambert_w(x, -1)),
        "special.sinc" => unary(args, |x| if x.abs() < 1e-15 { 1.0 } else { x.sin() / x }),
        _ => Err("OP"),
    }
}

fn num(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if x.is_finite() { Ok(x) } else { Err("NONFINITE") }
}
fn unary<F: FnOnce(f64) -> f64>(a: &[Value], f: F) -> Result<Value, &'static str> {
    if a.len() != 1 { return Err("ARG"); }
    finite(f(num(&a[0])?))
}
fn finite(x: f64) -> Result<Value, &'static str> { if x.is_finite() { Ok(json!(x)) } else { Err("DOMAIN") } }

fn gamma(x: f64) -> f64 { libm::tgamma(x) }
fn log_gamma(x: f64) -> f64 { if x <= 0.0 { f64::NAN } else { libm::lgamma(x) } }

fn beta(args: &[Value], log: bool) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let a = num(&args[0])?;
    let b = num(&args[1])?;
    if a <= 0.0 || b <= 0.0 { return Err("DOMAIN"); }
    let lb = libm::lgamma(a) + libm::lgamma(b) - libm::lgamma(a + b);
    finite(if log { lb } else { lb.exp() })
}

fn digamma(mut x: f64) -> f64 {
    if x <= 0.0 { return f64::NAN; }
    let mut r = 0.0;
    while x < 8.0 { r -= 1.0 / x; x += 1.0; }
    let inv = 1.0 / x; let inv2 = inv * inv;
    r + x.ln() - 0.5 * inv - inv2 * (1.0 / 12.0 - inv2 * (1.0 / 120.0 - inv2 / 252.0))
}
fn trigamma(mut x: f64) -> f64 {
    if x <= 0.0 { return f64::NAN; }
    let mut r = 0.0;
    while x < 8.0 { r += 1.0 / (x * x); x += 1.0; }
    let inv = 1.0 / x; let inv2 = inv * inv;
    r + inv + 0.5 * inv2 + inv2 * inv / 6.0 - inv2 * inv2 * inv / 30.0 + inv2 * inv2 * inv2 * inv / 42.0
}

fn fact(n: u32) -> f64 { (1..=n).fold(1.0, |a, b| a * b as f64) }
fn bessel_in(n: u32, x: f64) -> f64 {
    let h = x / 2.0;
    let mut sum = 0.0;
    let mut term = h.powi(n as i32) / fact(n);
    for k in 0..512 {
        if k > 0 { term *= h * h / ((k as f64) * (n as f64 + k as f64)); }
        sum += term;
        if !sum.is_finite() || !term.is_finite() { return f64::NAN; }
        if term.abs() <= 2.0 * f64::EPSILON * sum.abs().max(f64::MIN_POSITIVE) { break; }
    }
    sum
}
fn jn(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let n = args[0].as_u64().ok_or("TYPE")?;
    if n > 64 { return Err("LIMIT"); }
    finite(libm::jn(n as i32, num(&args[1])?))
}

// Euler transform of the alternating Dirichlet series.  A small, fixed
// difference table converges rapidly across s>0 and avoids the old 200k-term
// slow/low-accuracy path for small s.
fn eta(s: f64) -> f64 {
    if s <= 0.0 { return f64::NAN; }
    const N: usize = 128;
    let mut a = (0..N).map(|n| ((n + 1) as f64).powf(-s)).collect::<Vec<_>>();
    let mut sum = 0.0;
    let mut factor = 0.5;
    for len in (1..=N).rev() {
        sum += factor * a[0];
        if len == 1 { break; }
        for i in 0..len - 1 { a[i] -= a[i + 1]; }
        factor *= 0.5;
    }
    sum
}
fn zeta(s: f64) -> f64 {
    if s <= 0.0 || (s - 1.0).abs() <= 8.0 * f64::EPSILON { return f64::NAN; }
    let den = -((1.0 - s) * std::f64::consts::LN_2).exp_m1();
    eta(s) / den
}

fn lambert_w(x: f64, branch: i8) -> f64 {
    let min = -1.0 / E;
    if x < min || (branch == -1 && !(x < 0.0 && x >= min)) { return f64::NAN; }
    if (x - min).abs() <= 8.0 * f64::EPSILON { return -1.0; }
    if x == 0.0 { return if branch == 0 { 0.0 } else { f64::NAN }; }

    let q2 = 2.0 * (E * x + 1.0);
    let mut w = if q2 >= 0.0 && q2 < 0.04 {
        let q = q2.sqrt();
        if branch == 0 { -1.0 + q - q*q/3.0 + 11.0*q*q*q/72.0 }
        else { -1.0 - q - q*q/3.0 - 11.0*q*q*q/72.0 }
    } else if branch == 0 {
        // The asymptotic log(x)-log(log(x)) seed is undefined at x=1
        // and poor for moderate positive x.  log1p(x) is finite across this
        // transition and gives Halley a robust principal-branch seed.
        if x < 3.0 { x.ln_1p() } else { let l = x.ln(); l - l.ln() }
    } else {
        let l1 = (-x).ln(); l1 - (-l1).ln()
    };

    for _ in 0..100 {
        let ew = w.exp();
        let f = w * ew - x;
        let wp = w + 1.0;
        if wp.abs() < 1e-14 { break; }
        let den = ew * wp - (wp + 1.0) * f / (2.0 * wp);
        if !den.is_finite() || den == 0.0 { break; }
        let nw = w - f / den;
        if !nw.is_finite() { return f64::NAN; }
        if (nw - w).abs() < 1e-14 * (1.0 + nw.abs()) { return nw; }
        w = nw;
    }
    let residual = w * w.exp() - x;
    if residual.abs() <= 1e-12 * (1.0 + x.abs()) { w } else { f64::NAN }
}
