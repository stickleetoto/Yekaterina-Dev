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
        "special.gamma_p" => binary_dom(args, gamma_p),
        "special.gamma_q" => binary_dom(args, gamma_q),
        "special.beta_inc" => {
            if args.len() != 3 { return Err("ARG"); }
            let v = beta_inc(num(&args[0])?, num(&args[1])?, num(&args[2])?);
            if v.is_nan() { Err("DOMAIN") } else { finite(v) }
        }
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
fn binary_dom<F: FnOnce(f64, f64) -> f64>(a: &[Value], f: F) -> Result<Value, &'static str> {
    if a.len() != 2 { return Err("ARG"); }
    let v = f(num(&a[0])?, num(&a[1])?);
    if v.is_nan() { Err("DOMAIN") } else { finite(v) }
}

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

// ---------------------------------------------------------------------------
// Regularized incomplete gamma and beta.
//
// These are the foundation the t, chi-square and F distributions are built on,
// and without them the engine could produce a test statistic but never a
// p-value. Both use the standard series/continued-fraction pair, with a fixed
// iteration cap and a fixed tolerance: the loop count must not depend on the
// input in a way that could differ between runs or platforms, because this
// engine's contract is byte-identical output.

/// Iterations are capped rather than run to convergence. At this cap the
/// series and the continued fraction have both converged to f64 precision for
/// every argument the domain guards admit; the cap exists so the loop is
/// bounded, not because it is expected to be reached.
const INC_MAX_ITER: usize = 300;
const INC_EPS: f64 = 3.0e-16;
/// Guards the continued fractions against a zero denominator.
const INC_TINY: f64 = 1.0e-300;

/// Series expansion for P(a, x), used where it converges fastest: x < a + 1.
fn gamma_p_series(a: f64, x: f64) -> f64 {
    let mut ap = a;
    let mut del = 1.0 / a;
    let mut sum = del;
    for _ in 0..INC_MAX_ITER {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * INC_EPS { break; }
    }
    sum * (-x + a * x.ln() - libm::lgamma(a)).exp()
}

/// Continued fraction for Q(a, x) by modified Lentz, used where the series is
/// slow: x >= a + 1.
fn gamma_q_cf(a: f64, x: f64) -> f64 {
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / INC_TINY;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..=INC_MAX_ITER {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < INC_TINY { d = INC_TINY; }
        c = b + an / c;
        if c.abs() < INC_TINY { c = INC_TINY; }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < INC_EPS { break; }
    }
    (-x + a * x.ln() - libm::lgamma(a)).exp() * h
}

/// Regularized lower incomplete gamma P(a, x). NaN outside the domain, which
/// the caller turns into DOMAIN.
pub(crate) fn gamma_p(a: f64, x: f64) -> f64 {
    if a <= 0.0 || x < 0.0 { return f64::NAN; }
    if x == 0.0 { return 0.0; }
    if x < a + 1.0 { gamma_p_series(a, x) } else { 1.0 - gamma_q_cf(a, x) }
}

/// Regularized upper incomplete gamma Q(a, x) = 1 - P(a, x). Computed on its
/// own branch rather than as 1 - P so the far tail keeps its relative accuracy;
/// subtracting a number very close to 1 would throw away the digits that matter
/// for a small p-value.
pub(crate) fn gamma_q(a: f64, x: f64) -> f64 {
    if a <= 0.0 || x < 0.0 { return f64::NAN; }
    if x == 0.0 { return 1.0; }
    if x < a + 1.0 { 1.0 - gamma_p_series(a, x) } else { gamma_q_cf(a, x) }
}

/// Continued fraction for the incomplete beta, by modified Lentz.
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < INC_TINY { d = INC_TINY; }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=INC_MAX_ITER {
        let m = m as f64;
        let m2 = 2.0 * m;
        // Even step.
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < INC_TINY { d = INC_TINY; }
        c = 1.0 + aa / c;
        if c.abs() < INC_TINY { c = INC_TINY; }
        d = 1.0 / d;
        h *= d * c;
        // Odd step.
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < INC_TINY { d = INC_TINY; }
        c = 1.0 + aa / c;
        if c.abs() < INC_TINY { c = INC_TINY; }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < INC_EPS { break; }
    }
    h
}

/// Regularized incomplete beta I_x(a, b).
pub(crate) fn beta_inc(a: f64, b: f64, x: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 || !(0.0..=1.0).contains(&x) { return f64::NAN; }
    if x == 0.0 { return 0.0; }
    if x == 1.0 { return 1.0; }
    let front = (libm::lgamma(a + b) - libm::lgamma(a) - libm::lgamma(b)
        + a * x.ln() + b * (1.0 - x).ln()).exp();
    // The fraction converges quickly only on one side of this point; the
    // symmetry I_x(a,b) = 1 - I_(1-x)(b,a) covers the other.
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_cf(a, b, x) / a
    } else {
        1.0 - front * beta_cf(b, a, 1.0 - x) / b
    }
}
