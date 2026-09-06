//! Statistical inference: p-values, complete hypothesis tests, confidence
//! intervals and sample sizing.
//!
//! Before this module the engine could compute a t statistic but never a
//! p-value, which meant it could describe a sample and not decide anything
//! about it. The distributions in `advanced_probability` are the missing piece;
//! this is what they were for.
//!
//! Tail probabilities are taken from survival functions rather than `1 - cdf`
//! throughout. A p-value is a tail probability, and that is exactly where the
//! subtraction throws away the digits that matter.

use serde_json::{json, Value};

use crate::advanced_probability::{chi2_sf, f_sf, normal_ppf_std, normal_sf_std, t_sf};

const MAX_N: usize = 100_000;
const MAX_GROUPS: usize = 1_000;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "test.p_t" | "test.p_z" | "test.p_chi2" | "test.p_f" |
        "test.t_paired" | "test.t_one_sample_test" | "test.t_two_equal_test" |
        "test.t_welch_test" | "test.t_paired_test" |
        "test.anova_one_way" | "test.mann_whitney_u" | "test.wilcoxon_signed_rank" |
        "test.levene" | "test.bartlett" | "test.ks_normal" | "test.correlation_p" |
        "test.chi_square_yates" | "test.fisher_exact" | "test.binomial_test" | "test.mcnemar" |
        "test.ci_mean" | "test.ci_mean_diff" | "test.ci_proportion" | "test.ci_variance" |
        "test.sample_size_mean" | "test.sample_size_proportion" | "test.power_t" |
        "reg.multiple" |
        "reg.adjusted_r2" |
        "reg.aic" |
        "reg.bic" |
        "reg.slope_se" |
        "reg.intercept_se" |
        "reg.slope_t" |
        "reg.slope_p" |
        "reg.slope_ci" |
        "reg.f_test" |
        "reg.durbin_watson" |
        "reg.polynomial" |
        "reg.kendall_tau" |
        "reg.exponential" |
        "reg.power" |
        "reg.logarithmic" |
        "reg.theil_sen"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "test.p_t" => {
            need(args, 3)?;
            let t = num(&args[0])?;
            let df = positive(&args[1])?;
            finite(tail(t_sf(t.abs(), df), t, tails(&args[2])?))
        }
        "test.p_z" => {
            need(args, 2)?;
            let z = num(&args[0])?;
            finite(tail(normal_sf_std(z.abs()), z, tails(&args[1])?))
        }
        // Chi-square and F tests are one-sided by construction: a larger
        // statistic is the only kind of departure they can express.
        "test.p_chi2" => { need(args, 2)?; finite(chi2_sf(nonneg(&args[0])?, positive(&args[1])?)) }
        "test.p_f" => {
            need(args, 3)?;
            finite(f_sf(nonneg(&args[0])?, positive(&args[1])?, positive(&args[2])?))
        }

        "test.t_paired" => { let (t, _df) = paired_t(args)?; finite(t) }
        "test.t_one_sample_test" => {
            need(args, 2)?;
            let x = sample(&args[0])?;
            let mu0 = num(&args[1])?;
            let n = x.len() as f64;
            let sd = sd(&x)?;
            if sd == 0.0 { return Err("DEGENERATE"); }
            let t = (mean(&x) - mu0) / (sd / n.sqrt());
            report_t(t, n - 1.0)
        }
        "test.t_two_equal_test" => {
            let (a, b) = two_samples(args)?;
            let (na, nb) = (a.len() as f64, b.len() as f64);
            let (va, vb) = (var(&a)?, var(&b)?);
            let df = na + nb - 2.0;
            let pooled = ((na - 1.0) * va + (nb - 1.0) * vb) / df;
            if pooled == 0.0 { return Err("DEGENERATE"); }
            let t = (mean(&a) - mean(&b)) / (pooled * (1.0 / na + 1.0 / nb)).sqrt();
            report_t(t, df)
        }
        "test.t_welch_test" => {
            let (a, b) = two_samples(args)?;
            let (na, nb) = (a.len() as f64, b.len() as f64);
            let (va, vb) = (var(&a)?, var(&b)?);
            let se2 = va / na + vb / nb;
            if se2 == 0.0 { return Err("DEGENERATE"); }
            // Welch-Satterthwaite degrees of freedom.
            let df = se2 * se2 / ((va / na).powi(2) / (na - 1.0) + (vb / nb).powi(2) / (nb - 1.0));
            report_t((mean(&a) - mean(&b)) / se2.sqrt(), df)
        }
        "test.t_paired_test" => { let (t, df) = paired_t(args)?; report_t(t, df) }

        "test.anova_one_way" => anova(args),
        "test.mann_whitney_u" => mann_whitney(args),
        "test.wilcoxon_signed_rank" => wilcoxon(args),
        "test.levene" => levene(args),
        "test.bartlett" => bartlett(args),
        "test.ks_normal" => ks_normal(args),
        "test.correlation_p" => {
            need(args, 2)?;
            let r = num(&args[0])?;
            let n = count(&args[1])?;
            if !(-1.0..=1.0).contains(&r) { return Err("DOMAIN"); }
            if n < 3 { return Err("SHAPE"); }
            if r.abs() == 1.0 { return Ok(json!(0.0)); }
            let df = n as f64 - 2.0;
            let t = r * (df / (1.0 - r * r)).sqrt();
            finite(2.0 * t_sf(t.abs(), df))
        }
        "test.chi_square_yates" => yates(args),
        "test.fisher_exact" => fisher(args),
        "test.binomial_test" => binomial_test(args),
        "test.mcnemar" => mcnemar(args),

        "test.ci_mean" => ci_mean(args),
        "test.ci_mean_diff" => ci_mean_diff(args),
        "test.ci_proportion" => ci_proportion(args),
        "test.ci_variance" => ci_variance(args),
        "test.sample_size_mean" => sample_size_mean(args),
        "test.sample_size_proportion" => sample_size_proportion(args),
        "test.power_t" => power_t(args),
        _ if op.starts_with("reg.") => reg_run(op, args),
        _ => Err("OP"),
    }
}

// ------------------------------------------------------------------- helpers

fn num(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if x.is_finite() { Ok(x) } else { Err("NONFINITE") }
}
fn positive(v: &Value) -> Result<f64, &'static str> {
    let x = num(v)?;
    if x > 0.0 { Ok(x) } else { Err("DOMAIN") }
}
fn nonneg(v: &Value) -> Result<f64, &'static str> {
    let x = num(v)?;
    if x >= 0.0 { Ok(x) } else { Err("DOMAIN") }
}
fn count(v: &Value) -> Result<usize, &'static str> {
    let x = num(v)?;
    if x < 0.0 || x.fract() != 0.0 || x > MAX_N as f64 { return Err("DOMAIN"); }
    Ok(x as usize)
}
fn need(args: &[Value], n: usize) -> Result<(), &'static str> {
    if args.len() == n { Ok(()) } else { Err("ARG") }
}
fn finite(x: f64) -> Result<Value, &'static str> {
    if x.is_finite() { Ok(json!(x)) } else { Err("NONFINITE") }
}
fn probability(v: &Value) -> Result<f64, &'static str> {
    let p = num(v)?;
    if (0.0..=1.0).contains(&p) { Ok(p) } else { Err("DOMAIN") }
}
/// A confidence level, strictly inside (0, 1): 0 and 1 have no finite interval.
fn confidence(v: &Value) -> Result<f64, &'static str> {
    let c = num(v)?;
    if c > 0.0 && c < 1.0 { Ok(c) } else { Err("DOMAIN") }
}
fn tails(v: &Value) -> Result<u8, &'static str> {
    match v.as_u64() {
        Some(1) => Ok(1),
        Some(2) => Ok(2),
        _ => Err("DOMAIN"),
    }
}
/// One-tailed p follows the sign of the statistic; two-tailed doubles the
/// smaller tail. Splitting this out keeps every test consistent about it.
fn tail(upper_abs: f64, stat: f64, tails: u8) -> f64 {
    if tails == 2 { 2.0 * upper_abs } else if stat >= 0.0 { upper_abs } else { 1.0 - upper_abs }
}

fn values(v: &Value) -> Result<Vec<f64>, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.len() > MAX_N { return Err("LIMIT"); }
    a.iter().map(num).collect()
}
fn sample(v: &Value) -> Result<Vec<f64>, &'static str> {
    let x = values(v)?;
    if x.len() < 2 { return Err("SHAPE"); }
    Ok(x)
}
fn two_samples(args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), &'static str> {
    need(args, 2)?;
    Ok((sample(&args[0])?, sample(&args[1])?))
}
fn group_list(v: &Value) -> Result<Vec<Vec<f64>>, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.len() < 2 { return Err("SHAPE"); }
    if a.len() > MAX_GROUPS { return Err("LIMIT"); }
    let groups: Vec<Vec<f64>> = a.iter().map(values).collect::<Result<_, _>>()?;
    if groups.iter().any(|g| g.len() < 2) { return Err("SHAPE"); }
    Ok(groups)
}
fn mean(x: &[f64]) -> f64 { x.iter().sum::<f64>() / x.len() as f64 }
fn var(x: &[f64]) -> Result<f64, &'static str> {
    if x.len() < 2 { return Err("SHAPE"); }
    let m = mean(x);
    Ok(x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (x.len() - 1) as f64)
}
fn sd(x: &[f64]) -> Result<f64, &'static str> { Ok(var(x)?.sqrt()) }
fn median(x: &[f64]) -> f64 {
    let mut s = x.to_vec();
    s.sort_by(f64::total_cmp);
    let n = s.len();
    if n % 2 == 1 { s[n / 2] } else { (s[n / 2 - 1] + s[n / 2]) / 2.0 }
}

/// Midranks: tied values share the average of the ranks they span. Returns the
/// ranks and the tie-correction sum used by the rank tests.
fn midranks(x: &[f64]) -> (Vec<f64>, f64) {
    let n = x.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| x[i].total_cmp(&x[j]));
    let mut ranks = vec![0.0; n];
    let mut tie_sum = 0.0;
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && x[order[j + 1]] == x[order[i]] { j += 1; }
        let width = (j - i + 1) as f64;
        let avg = (i + j + 2) as f64 / 2.0;   // ranks are 1-based
        for &k in &order[i..=j] { ranks[k] = avg; }
        if width > 1.0 { tie_sum += width * width * width - width; }
        i = j + 1;
    }
    (ranks, tie_sum)
}

fn report_t(t: f64, df: f64) -> Result<Value, &'static str> {
    if !t.is_finite() || !df.is_finite() || df <= 0.0 { return Err("NONFINITE"); }
    Ok(json!({"t": t, "df": df, "p": 2.0 * t_sf(t.abs(), df)}))
}

fn paired_t(args: &[Value]) -> Result<(f64, f64), &'static str> {
    let (a, b) = two_samples(args)?;
    if a.len() != b.len() { return Err("SHAPE"); }
    let d: Vec<f64> = a.iter().zip(&b).map(|(x, y)| x - y).collect();
    let n = d.len() as f64;
    let s = sd(&d)?;
    if s == 0.0 { return Err("DEGENERATE"); }
    Ok((mean(&d) / (s / n.sqrt()), n - 1.0))
}

// --------------------------------------------------------------------- tests

fn anova(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?;
    let groups = group_list(&args[0])?;
    let n: usize = groups.iter().map(Vec::len).sum();
    let k = groups.len();
    if n <= k { return Err("SHAPE"); }
    let grand = groups.iter().flatten().sum::<f64>() / n as f64;
    let ssb: f64 = groups.iter().map(|g| g.len() as f64 * (mean(g) - grand).powi(2)).sum();
    let ssw: f64 = groups.iter().flat_map(|g| {
        let m = mean(g);
        g.iter().map(move |v| (v - m) * (v - m))
    }).sum();
    if ssw == 0.0 { return Err("DEGENERATE"); }
    let (df1, df2) = ((k - 1) as f64, (n - k) as f64);
    let f = (ssb / df1) / (ssw / df2);
    Ok(json!({"f": f, "df1": df1, "df2": df2, "p": f_sf(f, df1, df2)}))
}

/// Mann-Whitney U with the normal approximation, tie-corrected and with a
/// continuity correction. The exact permutation p-value is not offered: it is
/// only tractable for small samples, and an operation whose cost explodes with
/// input size does not belong in a batch engine.
fn mann_whitney(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two_samples(args)?;
    let (n1, n2) = (a.len() as f64, b.len() as f64);
    let mut all = a.clone();
    all.extend_from_slice(&b);
    let (ranks, tie_sum) = midranks(&all);
    let r1: f64 = ranks[..a.len()].iter().sum();
    let u1 = r1 - n1 * (n1 + 1.0) / 2.0;
    let u2 = n1 * n2 - u1;
    let u = u1.min(u2);
    let mu = n1 * n2 / 2.0;
    let n = n1 + n2;
    let sigma2 = n1 * n2 / 12.0 * ((n + 1.0) - tie_sum / (n * (n - 1.0)));
    if sigma2 <= 0.0 { return Err("DEGENERATE"); }
    let z = (u - mu + 0.5) / sigma2.sqrt();
    Ok(json!({"u": u, "u1": u1, "z": z, "p": 2.0 * normal_sf_std(z.abs())}))
}

fn wilcoxon(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two_samples(args)?;
    if a.len() != b.len() { return Err("SHAPE"); }
    // Zero differences carry no information about direction and are dropped,
    // which is the standard (Wilcoxon) handling.
    let d: Vec<f64> = a.iter().zip(&b).map(|(x, y)| x - y).filter(|v| *v != 0.0).collect();
    if d.len() < 2 { return Err("SHAPE"); }
    let n = d.len() as f64;
    let abs: Vec<f64> = d.iter().map(|v| v.abs()).collect();
    let (ranks, tie_sum) = midranks(&abs);
    let w_plus: f64 = d.iter().zip(&ranks).filter(|(v, _)| **v > 0.0).map(|(_, r)| *r).sum();
    let w_minus = n * (n + 1.0) / 2.0 - w_plus;
    let w = w_plus.min(w_minus);
    let mu = n * (n + 1.0) / 4.0;
    let sigma2 = n * (n + 1.0) * (2.0 * n + 1.0) / 24.0 - tie_sum / 48.0;
    if sigma2 <= 0.0 { return Err("DEGENERATE"); }
    let z = (w - mu + 0.5) / sigma2.sqrt();
    Ok(json!({"w": w, "z": z, "p": 2.0 * normal_sf_std(z.abs())}))
}

/// Levene's test centred on the median, which is the Brown-Forsythe variant and
/// the one that stays honest on skewed data.
fn levene(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?;
    let groups = group_list(&args[0])?;
    let spread: Vec<Vec<f64>> = groups.iter()
        .map(|g| { let c = median(g); g.iter().map(|v| (v - c).abs()).collect() })
        .collect();
    anova(&[json!(spread)])
        .map(|v| json!({"f": v["f"], "df1": v["df1"], "df2": v["df2"], "p": v["p"]}))
}

fn bartlett(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?;
    let groups = group_list(&args[0])?;
    let k = groups.len() as f64;
    let n: f64 = groups.iter().map(|g| g.len() as f64).sum();
    let vars: Vec<f64> = groups.iter().map(|g| var(g)).collect::<Result<_, _>>()?;
    if vars.iter().any(|v| *v <= 0.0) { return Err("DEGENERATE"); }
    let pooled: f64 = groups.iter().zip(&vars)
        .map(|(g, v)| (g.len() as f64 - 1.0) * v).sum::<f64>() / (n - k);
    let num = (n - k) * pooled.ln()
        - groups.iter().zip(&vars).map(|(g, v)| (g.len() as f64 - 1.0) * v.ln()).sum::<f64>();
    let den = 1.0 + (groups.iter().map(|g| 1.0 / (g.len() as f64 - 1.0)).sum::<f64>()
        - 1.0 / (n - k)) / (3.0 * (k - 1.0));
    let chi2 = num / den;
    let df = k - 1.0;
    Ok(json!({"chi2": chi2, "df": df, "p": chi2_sf(chi2, df)}))
}

/// One-sample Kolmogorov-Smirnov against a *specified* normal. The parameters
/// must be given rather than estimated from the sample: estimating them turns
/// this into Lilliefors' test, whose null distribution is different, and
/// quietly reporting a KS p-value for it would overstate the fit.
fn ks_normal(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let mut x = sample(&args[0])?;
    let mu = num(&args[1])?;
    let sigma = positive(&args[2])?;
    x.sort_by(f64::total_cmp);
    let n = x.len() as f64;
    let mut d: f64 = 0.0;
    for (i, v) in x.iter().enumerate() {
        let cdf = 1.0 - normal_sf_std((v - mu) / sigma);
        d = d.max(((i + 1) as f64 / n - cdf).abs()).max((cdf - i as f64 / n).abs());
    }
    // Asymptotic Kolmogorov distribution. Labelled as asymptotic in the
    // operation summary because it is not the exact small-sample p-value.
    let lambda = n.sqrt() * d;
    let mut q = 0.0;
    for k in 1..=100 {
        let k = k as f64;
        q += (if (k as i64) % 2 == 1 { 1.0 } else { -1.0 }) * (-2.0 * k * k * lambda * lambda).exp();
    }
    let p = (2.0 * q).clamp(0.0, 1.0);
    Ok(json!({"d": d, "p": p}))
}

fn table2x2(args: &[Value]) -> Result<(f64, f64, f64, f64), &'static str> {
    need(args, 4)?;
    Ok((nonneg(&args[0])?, nonneg(&args[1])?, nonneg(&args[2])?, nonneg(&args[3])?))
}

fn yates(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b, c, d) = table2x2(args)?;
    let n = a + b + c + d;
    let (r1, r2, c1, c2) = (a + b, c + d, a + c, b + d);
    if r1 == 0.0 || r2 == 0.0 || c1 == 0.0 || c2 == 0.0 { return Err("DEGENERATE"); }
    let num = ((a * d - b * c).abs() - n / 2.0).max(0.0);
    let chi2 = n * num * num / (r1 * r2 * c1 * c2);
    Ok(json!({"chi2": chi2, "df": 1.0, "p": chi2_sf(chi2, 1.0)}))
}

fn ln_choose(n: f64, k: f64) -> f64 {
    libm::lgamma(n + 1.0) - libm::lgamma(k + 1.0) - libm::lgamma(n - k + 1.0)
}

/// Fisher's exact test, two-sided by the "sum every table at least as extreme"
/// convention: every hypergeometric probability no greater than the observed
/// one is added. The small relative slack matches the usual implementations and
/// keeps a table that ties the observed probability from being excluded by
/// floating-point noise.
fn fisher(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b, c, d) = table2x2(args)?;
    if [a, b, c, d].iter().any(|v| v.fract() != 0.0 || *v > 1e7) { return Err("DOMAIN"); }
    let n = a + b + c + d;
    let (r1, c1) = (a + b, a + c);
    if n == 0.0 { return Err("DEGENERATE"); }
    let ln_den = ln_choose(n, c1);
    let pmf = |k: f64| (ln_choose(r1, k) + ln_choose(n - r1, c1 - k) - ln_den).exp();
    let observed = pmf(a);
    let lo = 0.0_f64.max(c1 - (n - r1));
    let hi = r1.min(c1);
    let mut p = 0.0;
    let mut k = lo;
    while k <= hi {
        let v = pmf(k);
        if v <= observed * (1.0 + 1e-7) { p += v; }
        k += 1.0;
    }
    finite(p.min(1.0))
}

/// Exact two-sided binomial test, by the same "at least as extreme" rule.
fn binomial_test(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let k = count(&args[0])?;
    let n = count(&args[1])?;
    let p0 = probability(&args[2])?;
    if k > n || n == 0 { return Err("DOMAIN"); }
    if p0 == 0.0 { return Ok(json!(if k == 0 { 1.0 } else { 0.0 })); }
    if p0 == 1.0 { return Ok(json!(if k == n { 1.0 } else { 0.0 })); }
    let nf = n as f64;
    let pmf = |i: f64| (ln_choose(nf, i) + i * p0.ln() + (nf - i) * (1.0 - p0).ln()).exp();
    let observed = pmf(k as f64);
    let mut p = 0.0;
    for i in 0..=n {
        let v = pmf(i as f64);
        if v <= observed * (1.0 + 1e-7) { p += v; }
    }
    finite(p.min(1.0))
}

fn mcnemar(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let b = nonneg(&args[0])?;
    let c = nonneg(&args[1])?;
    if b + c == 0.0 { return Err("DEGENERATE"); }
    let num = ((b - c).abs() - 1.0).max(0.0);
    let chi2 = num * num / (b + c);
    Ok(json!({"chi2": chi2, "df": 1.0, "p": chi2_sf(chi2, 1.0)}))
}

// -------------------------------------------------------- confidence intervals

/// Two-sided critical value of t at the given confidence level.
fn t_crit(conf: f64, df: f64) -> f64 {
    // Solve the upper tail directly rather than going through a quantile of a
    // probability very close to one.
    let target = (1.0 - conf) / 2.0;
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..200 { if t_sf(hi, df) <= target { break; } hi *= 2.0; }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi { break; }
        if t_sf(mid, df) > target { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

fn ci_mean(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let x = sample(&args[0])?;
    let conf = confidence(&args[1])?;
    let n = x.len() as f64;
    let m = mean(&x);
    let half = t_crit(conf, n - 1.0) * sd(&x)? / n.sqrt();
    Ok(json!([m - half, m + half]))
}

fn ci_mean_diff(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let a = sample(&args[0])?;
    let b = sample(&args[1])?;
    let conf = confidence(&args[2])?;
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let (va, vb) = (var(&a)?, var(&b)?);
    let se2 = va / na + vb / nb;
    if se2 == 0.0 { return Err("DEGENERATE"); }
    let df = se2 * se2 / ((va / na).powi(2) / (na - 1.0) + (vb / nb).powi(2) / (nb - 1.0));
    let half = t_crit(conf, df) * se2.sqrt();
    let d = mean(&a) - mean(&b);
    Ok(json!([d - half, d + half]))
}

/// Wilson score interval, not the normal approximation: it stays inside [0, 1]
/// and does not collapse to zero width when no successes are observed.
fn ci_proportion(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let k = count(&args[0])? as f64;
    let n = count(&args[1])? as f64;
    let conf = confidence(&args[2])?;
    if n == 0.0 || k > n { return Err("DOMAIN"); }
    let z = normal_ppf_std(1.0 - (1.0 - conf) / 2.0);
    let p = k / n;
    let denom = 1.0 + z * z / n;
    let centre = (p + z * z / (2.0 * n)) / denom;
    let half = z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt() / denom;
    Ok(json!([(centre - half).max(0.0), (centre + half).min(1.0)]))
}

fn ci_variance(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let x = sample(&args[0])?;
    let conf = confidence(&args[1])?;
    let df = x.len() as f64 - 1.0;
    let s2 = var(&x)?;
    let alpha = 1.0 - conf;
    let upper_crit = chi2_crit(1.0 - alpha / 2.0, df);
    let lower_crit = chi2_crit(alpha / 2.0, df);
    if upper_crit <= 0.0 || lower_crit <= 0.0 { return Err("DEGENERATE"); }
    Ok(json!([df * s2 / upper_crit, df * s2 / lower_crit]))
}

fn chi2_crit(p: f64, df: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, df + 10.0 * (2.0 * df).sqrt() + 10.0);
    for _ in 0..200 { if 1.0 - chi2_sf(hi, df) >= p { break; } hi *= 2.0; }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi { break; }
        if 1.0 - chi2_sf(mid, df) < p { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

// ------------------------------------------------------------ sizing and power

fn alpha_power(args: &[Value], i: usize) -> Result<(f64, f64), &'static str> {
    let alpha = num(&args[i])?;
    let power = num(&args[i + 1])?;
    if alpha <= 0.0 || alpha >= 1.0 || power <= 0.0 || power >= 1.0 { return Err("DOMAIN"); }
    Ok((alpha, power))
}

/// Normal-approximation sample size for a two-sided one-sample mean test. It
/// ignores the t correction, so it is a starting point rather than an exact
/// answer, and the summary says so.
fn sample_size_mean(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 4)?;
    let effect = num(&args[0])?;
    let sd = positive(&args[1])?;
    let (alpha, power) = alpha_power(args, 2)?;
    if effect == 0.0 { return Err("DOMAIN"); }
    let z_a = normal_ppf_std(1.0 - alpha / 2.0);
    let z_b = normal_ppf_std(power);
    finite(((z_a + z_b) * sd / effect).powi(2).ceil())
}

fn sample_size_proportion(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 4)?;
    let p1 = probability(&args[0])?;
    let p2 = probability(&args[1])?;
    let (alpha, power) = alpha_power(args, 2)?;
    if p1 == p2 { return Err("DOMAIN"); }
    let z_a = normal_ppf_std(1.0 - alpha / 2.0);
    let z_b = normal_ppf_std(power);
    let pbar = (p1 + p2) / 2.0;
    let num = z_a * (2.0 * pbar * (1.0 - pbar)).sqrt()
        + z_b * (p1 * (1.0 - p1) + p2 * (1.0 - p2)).sqrt();
    finite((num / (p1 - p2)).powi(2).ceil())
}

/// Power of a two-sided one-sample t test, by the normal approximation to the
/// noncentral t. Adequate for planning; not a substitute for the exact
/// noncentral calculation.
fn power_t(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let n = count(&args[0])?;
    let effect = num(&args[1])?;
    let alpha = num(&args[2])?;
    if n < 2 { return Err("SHAPE"); }
    if alpha <= 0.0 || alpha >= 1.0 { return Err("DOMAIN"); }
    let ncp = effect * (n as f64).sqrt();
    let crit = t_crit(1.0 - alpha, (n - 1) as f64);
    finite(normal_sf_std(crit - ncp) + normal_sf_std(crit + ncp))
}

// ---------------------------------------------------------------------------
// Regression: multiple least squares, inference on the fitted coefficients, and
// the diagnostics that say whether the fit means anything.
//
// The engine could already fit a line through two vectors. It could not say
// whether the slope was distinguishable from zero, could not fit more than one
// predictor, and could not compare two models. Those are the things a
// regression is usually run to find out.

/// Predictors accepted by the multiple-regression path. Wide enough for real
/// designs, narrow enough that the O(k^3) solve stays bounded.
const MAX_PREDICTORS: usize = 200;
/// Polynomial degree cap. Beyond this the Vandermonde system is too
/// ill-conditioned for the answer to mean anything, so it is refused rather
/// than returned with quiet garbage in the high coefficients.
const MAX_DEGREE: usize = 12;

fn matrix(v: &Value) -> Result<Vec<Vec<f64>>, &'static str> {
    let rows = v.as_array().ok_or("TYPE")?;
    if rows.is_empty() { return Err("EMPTY"); }
    if rows.len() > MAX_N { return Err("LIMIT"); }
    let out: Vec<Vec<f64>> = rows.iter().map(values).collect::<Result<_, _>>()?;
    let width = out[0].len();
    if width == 0 || width > MAX_PREDICTORS { return Err("SHAPE"); }
    if out.iter().any(|r| r.len() != width) { return Err("SHAPE"); }
    Ok(out)
}

/// Solves a symmetric positive-definite normal-equation system by Gaussian
/// elimination with partial pivoting. Deterministic: the pivot is chosen by
/// magnitude with ties broken by index, so the same input always takes the same
/// path.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, &'static str> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if a[row][col].abs() > a[pivot][col].abs() { pivot = row; }
        }
        if a[pivot][col].abs() < 1e-300 { return Err("SINGULAR"); }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            if f == 0.0 { continue; }
            // Split so the pivot row and the row being reduced are separate
            // borrows; `row` is always greater than `col` here, so the pivot
            // lands in the first half.
            let (above, from_row) = a.split_at_mut(row);
            for (target, pivot) in from_row[0].iter_mut().zip(&above[col]).skip(col) {
                *target -= f * pivot;
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let mut acc = b[row];
        for k in row + 1..n { acc -= a[row][k] * x[k]; }
        x[row] = acc / a[row][row];
    }
    if x.iter().any(|v| !v.is_finite()) { return Err("NONFINITE"); }
    Ok(x)
}

/// Least squares for a design matrix that already contains its intercept
/// column. Returns the coefficients.
fn least_squares(design: &[Vec<f64>], y: &[f64]) -> Result<Vec<f64>, &'static str> {
    let n = design.len();
    let k = design[0].len();
    if n != y.len() { return Err("SHAPE"); }
    if n <= k { return Err("SHAPE"); }
    let mut ata = vec![vec![0.0; k]; k];
    let mut aty = vec![0.0; k];
    for row in 0..n {
        for i in 0..k {
            aty[i] += design[row][i] * y[row];
            for j in 0..k { ata[i][j] += design[row][i] * design[row][j]; }
        }
    }
    solve(ata, aty)
}

fn fitted(design: &[Vec<f64>], coef: &[f64]) -> Vec<f64> {
    design.iter().map(|r| r.iter().zip(coef).map(|(a, b)| a * b).sum()).collect()
}

fn sums(y: &[f64], yhat: &[f64]) -> (f64, f64) {
    let m = mean(y);
    let sst = y.iter().map(|v| (v - m) * (v - m)).sum::<f64>();
    let sse = y.iter().zip(yhat).map(|(a, b)| (a - b) * (a - b)).sum::<f64>();
    (sst, sse)
}

/// Simple regression pieces shared by the inference operations: slope,
/// intercept, residual standard error and the spread of x.
fn simple_fit(args: &[Value]) -> Result<(f64, f64, f64, f64, f64), &'static str> {
    let (x, y) = two_samples(args)?;
    if x.len() != y.len() { return Err("SHAPE"); }
    let n = x.len() as f64;
    if x.len() < 3 { return Err("SHAPE"); }
    let (mx, my) = (mean(&x), mean(&y));
    let sxx = x.iter().map(|v| (v - mx) * (v - mx)).sum::<f64>();
    if sxx == 0.0 { return Err("DEGENERATE"); }
    let sxy = x.iter().zip(&y).map(|(a, b)| (a - mx) * (b - my)).sum::<f64>();
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let sse = x.iter().zip(&y).map(|(a, b)| {
        let e = b - (intercept + slope * a);
        e * e
    }).sum::<f64>();
    let s2 = sse / (n - 2.0);
    Ok((slope, intercept, s2, sxx, n))
}

fn reg_run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "reg.multiple" => {
            need(args, 2)?;
            let x = matrix(&args[0])?;
            let y = values(&args[1])?;
            if x.len() != y.len() { return Err("SHAPE"); }
            // The intercept column is added here rather than asked for, so the
            // caller cannot accidentally fit through the origin.
            let design: Vec<Vec<f64>> = x.iter()
                .map(|r| { let mut d = vec![1.0]; d.extend_from_slice(r); d }).collect();
            let coef = least_squares(&design, &y)?;
            let yhat = fitted(&design, &coef);
            let (sst, sse) = sums(&y, &yhat);
            let k = x[0].len();
            let n = y.len();
            let r2 = if sst == 0.0 { f64::NAN } else { 1.0 - sse / sst };
            if !r2.is_finite() { return Err("DEGENERATE"); }
            Ok(json!({
                "intercept": coef[0],
                "coefficients": coef[1..].to_vec(),
                "r2": r2,
                "adjusted_r2": 1.0 - (1.0 - r2) * (n as f64 - 1.0) / (n - k - 1) as f64,
                "sse": sse,
                "n": n as f64,
                "predictors": k as f64,
            }))
        }
        "reg.adjusted_r2" => {
            need(args, 3)?;
            let y = sample(&args[0])?;
            let yhat = values(&args[1])?;
            let k = count(&args[2])?;
            if y.len() != yhat.len() { return Err("SHAPE"); }
            if y.len() <= k + 1 { return Err("SHAPE"); }
            let (sst, sse) = sums(&y, &yhat);
            if sst == 0.0 { return Err("DEGENERATE"); }
            let r2 = 1.0 - sse / sst;
            finite(1.0 - (1.0 - r2) * (y.len() as f64 - 1.0) / (y.len() - k - 1) as f64)
        }
        "reg.aic" | "reg.bic" => {
            need(args, 3)?;
            let y = sample(&args[0])?;
            let yhat = values(&args[1])?;
            let k = count(&args[2])? as f64;
            if y.len() != yhat.len() { return Err("SHAPE"); }
            let n = y.len() as f64;
            let (_, sse) = sums(&y, &yhat);
            if sse <= 0.0 { return Err("DEGENERATE"); }
            // Gaussian log-likelihood form, with the variance counted as a
            // fitted parameter the way the standard definitions do.
            let ll = -0.5 * n * ((2.0 * std::f64::consts::PI * sse / n).ln() + 1.0);
            let params = k + 1.0;
            finite(if op == "reg.aic" { 2.0 * params - 2.0 * ll }
                   else { params * n.ln() - 2.0 * ll })
        }
        "reg.slope_se" => { let (_, _, s2, sxx, _) = simple_fit(args)?; finite((s2 / sxx).sqrt()) }
        "reg.intercept_se" => {
            let (_, _, s2, sxx, n) = simple_fit(args)?;
            let (x, _) = two_samples(args)?;
            let mx = mean(&x);
            finite((s2 * (1.0 / n + mx * mx / sxx)).sqrt())
        }
        "reg.slope_t" => {
            let (slope, _, s2, sxx, _) = simple_fit(args)?;
            let se = (s2 / sxx).sqrt();
            if se == 0.0 { return Err("DEGENERATE"); }
            finite(slope / se)
        }
        "reg.slope_p" => {
            let (slope, _, s2, sxx, n) = simple_fit(args)?;
            let se = (s2 / sxx).sqrt();
            if se == 0.0 { return Err("DEGENERATE"); }
            finite(2.0 * t_sf((slope / se).abs(), n - 2.0))
        }
        "reg.slope_ci" => {
            need(args, 3)?;
            let (slope, _, s2, sxx, n) = simple_fit(&args[..2])?;
            let conf = confidence(&args[2])?;
            let half = t_crit(conf, n - 2.0) * (s2 / sxx).sqrt();
            Ok(json!([slope - half, slope + half]))
        }
        "reg.f_test" => {
            let (slope, _, s2, sxx, n) = simple_fit(args)?;
            let se2 = s2 / sxx;
            if se2 == 0.0 { return Err("DEGENERATE"); }
            // For one predictor the F statistic is the square of the slope t.
            let f = slope * slope / se2;
            let df2 = n - 2.0;
            Ok(json!({"f": f, "df1": 1.0, "df2": df2, "p": f_sf(f, 1.0, df2)}))
        }
        "reg.durbin_watson" => {
            need(args, 1)?;
            let e = sample(&args[0])?;
            let den: f64 = e.iter().map(|v| v * v).sum();
            if den == 0.0 { return Err("DEGENERATE"); }
            let num: f64 = e.windows(2).map(|w| (w[1] - w[0]) * (w[1] - w[0])).sum();
            finite(num / den)
        }
        "reg.polynomial" => {
            need(args, 3)?;
            let x = sample(&args[0])?;
            let y = values(&args[1])?;
            let degree = count(&args[2])?;
            if x.len() != y.len() { return Err("SHAPE"); }
            if degree > MAX_DEGREE { return Err("LIMIT"); }
            if x.len() <= degree + 1 { return Err("SHAPE"); }
            let design: Vec<Vec<f64>> = x.iter()
                .map(|v| (0..=degree).map(|p| v.powi(p as i32)).collect()).collect();
            let coef = least_squares(&design, &y)?;
            // Highest degree first, matching alg.poly_eval's Horner convention.
            Ok(json!(coef.into_iter().rev().collect::<Vec<_>>()))
        }
        "reg.kendall_tau" => {
            let (x, y) = two_samples(args)?;
            if x.len() != y.len() { return Err("SHAPE"); }
            let n = x.len();
            let (mut concordant, mut discordant, mut tx, mut ty) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
            for i in 0..n {
                for j in i + 1..n {
                    let (dx, dy) = (x[i] - x[j], y[i] - y[j]);
                    let s = dx * dy;
                    if s > 0.0 { concordant += 1.0; }
                    else if s < 0.0 { discordant += 1.0; }
                    else if dx == 0.0 && dy == 0.0 { tx += 1.0; ty += 1.0; }
                    else if dx == 0.0 { tx += 1.0; }
                    else { ty += 1.0; }
                }
            }
            // tau-b, which corrects for ties in either variable.
            let d = ((concordant + discordant + tx) * (concordant + discordant + ty)).sqrt();
            if d == 0.0 { return Err("DEGENERATE"); }
            finite((concordant - discordant) / d)
        }
        // Linearisable fits. Each transforms, fits a line, and transforms back,
        // so the residuals are minimised in the transformed space -- which is
        // what "log-linear fit" means and is not the same as a nonlinear fit.
        "reg.exponential" | "reg.power" | "reg.logarithmic" => {
            let (x, y) = two_samples(args)?;
            if x.len() != y.len() { return Err("SHAPE"); }
            let (tx, ty): (Vec<f64>, Vec<f64>) = match op {
                "reg.exponential" => {
                    if y.iter().any(|v| *v <= 0.0) { return Err("DOMAIN"); }
                    (x.clone(), y.iter().map(|v| v.ln()).collect())
                }
                "reg.power" => {
                    if x.iter().any(|v| *v <= 0.0) || y.iter().any(|v| *v <= 0.0) { return Err("DOMAIN"); }
                    (x.iter().map(|v| v.ln()).collect(), y.iter().map(|v| v.ln()).collect())
                }
                _ => {
                    if x.iter().any(|v| *v <= 0.0) { return Err("DOMAIN"); }
                    (x.iter().map(|v| v.ln()).collect(), y.clone())
                }
            };
            let (slope, intercept, _, _, _) = simple_fit(&[json!(tx), json!(ty)])?;
            match op {
                "reg.exponential" => Ok(json!({"a": intercept.exp(), "b": slope})),
                "reg.power" => Ok(json!({"a": intercept.exp(), "b": slope})),
                _ => Ok(json!({"a": intercept, "b": slope})),
            }
        }
        // Theil-Sen: the median of all pairwise slopes. Insensitive to
        // outliers in a way least squares is not, at O(n^2) cost, which the
        // sample cap below keeps bounded.
        "reg.theil_sen" => {
            let (x, y) = two_samples(args)?;
            if x.len() != y.len() { return Err("SHAPE"); }
            if x.len() > 2_000 { return Err("LIMIT"); }
            let mut slopes = Vec::new();
            for i in 0..x.len() {
                for j in i + 1..x.len() {
                    if x[j] != x[i] { slopes.push((y[j] - y[i]) / (x[j] - x[i])); }
                }
            }
            if slopes.is_empty() { return Err("DEGENERATE"); }
            let slope = median(&slopes);
            // Intercept by the "separate" convention, median(y) - slope *
            // median(x). The alternative, median(y - slope * x), is also in use
            // and gives a different answer on the same data; this one matches
            // the common implementations, so a result can be checked against
            // them without the difference being mistaken for an error.
            let intercept = median(&y) - slope * median(&x);
            Ok(json!({"slope": slope, "intercept": intercept}))
        }
        _ => Err("OP"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(op: &str, args: &[Value]) -> Result<Value, &'static str> {
        execute(op, args).expect("op not routed")
    }
    fn n(op: &str, args: &[Value]) -> f64 {
        call(op, args).expect("op errored").as_f64().unwrap()
    }
    fn field(op: &str, args: &[Value], key: &str) -> f64 {
        call(op, args).expect("op errored")[key].as_f64().unwrap()
    }
    fn pair_of(op: &str, args: &[Value]) -> (f64, f64) {
        let v = call(op, args).expect("op errored");
        (v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
    }
    fn err(op: &str, args: &[Value]) -> &'static str { call(op, args).unwrap_err() }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} != {b}");
    }

    const A: [f64; 8] = [5.1, 4.9, 6.2, 5.8, 5.0, 6.6, 5.3, 4.4];
    const B: [f64; 8] = [4.1, 4.4, 3.9, 4.6, 5.2, 4.0, 4.8, 5.9];

    fn a() -> Value { json!(A.to_vec()) }
    fn b() -> Value { json!(B.to_vec()) }

    #[test]
    fn two_tailed_p_is_twice_the_one_tailed_p_for_a_positive_statistic() {
        for (t, df) in [(2.5, 10.0), (0.3, 4.0), (7.0, 100.0)] {
            close(n("test.p_t", &[json!(t), json!(df), json!(2)]),
                  2.0 * n("test.p_t", &[json!(t), json!(df), json!(1)]));
        }
        // And the two-tailed value must not depend on the sign.
        close(n("test.p_t", &[json!(-2.5), json!(10.0), json!(2)]),
              n("test.p_t", &[json!(2.5), json!(10.0), json!(2)]));
        close(n("test.p_z", &[json!(-1.96), json!(2)]), n("test.p_z", &[json!(1.96), json!(2)]));
        // One-tailed must follow the sign: a large negative z is not surprising
        // in the upper tail.
        assert!(n("test.p_z", &[json!(-3.0), json!(1)]) > 0.99);
        assert!(n("test.p_z", &[json!(3.0), json!(1)]) < 0.01);
    }

    /// With one predictor the F statistic is the square of the t statistic and
    /// the two p-values must be identical. Anything else means one of the two
    /// distributions is wrong.
    #[test]
    fn f_and_t_agree_on_one_degree_of_freedom() {
        let t = field("test.t_two_equal_test", &[a(), b()], "t");
        let df = field("test.t_two_equal_test", &[a(), b()], "df");
        let p_t = field("test.t_two_equal_test", &[a(), b()], "p");
        let groups = json!([A.to_vec(), B.to_vec()]);
        let f = field("test.anova_one_way", std::slice::from_ref(&groups), "f");
        let p_f = field("test.anova_one_way", &[groups], "p");
        close(f, t * t);
        close(p_f, p_t);
        close(n("test.p_f", &[json!(f), json!(1.0), json!(df)]), p_t);
    }

    #[test]
    fn a_paired_test_is_a_one_sample_test_on_the_differences() {
        let d: Vec<f64> = A.iter().zip(B.iter()).map(|(x, y)| x - y).collect();
        close(field("test.t_paired_test", &[a(), b()], "t"),
              field("test.t_one_sample_test", &[json!(d), json!(0.0)], "t"));
    }

    #[test]
    fn welch_matches_the_pooled_test_when_the_samples_match() {
        // Equal sizes and equal variances are exactly the case where the two
        // tests coincide; if they disagree there, one of them is wrong.
        let x = json!([1.0, 2.0, 3.0, 4.0]);
        let y = json!([3.0, 4.0, 5.0, 6.0]);
        close(field("test.t_welch_test", &[x.clone(), y.clone()], "t"),
              field("test.t_two_equal_test", &[x.clone(), y.clone()], "t"));
        close(field("test.t_welch_test", &[x.clone(), y.clone()], "df"),
              field("test.t_two_equal_test", &[x, y], "df"));
    }

    #[test]
    fn mann_whitney_u_statistics_sum_to_the_product_of_the_sizes() {
        let v = call("test.mann_whitney_u", &[a(), b()]).unwrap();
        let (u, u1) = (v["u"].as_f64().unwrap(), v["u1"].as_f64().unwrap());
        let u2 = 64.0 - u1;                     // 8 * 8
        assert!((u - u1.min(u2)).abs() < 1e-12, "u should be the smaller of the two");
        assert!((0.0..=64.0).contains(&u1));
        // Reversing the samples swaps U1 and U2 but must leave p unchanged.
        let rev = call("test.mann_whitney_u", &[b(), a()]).unwrap();
        close(rev["u1"].as_f64().unwrap(), u2);
        close(rev["p"].as_f64().unwrap(), v["p"].as_f64().unwrap());
    }

    #[test]
    fn wilcoxon_statistic_stays_within_its_range() {
        let v = call("test.wilcoxon_signed_rank", &[a(), b()]).unwrap();
        let w = v["w"].as_f64().unwrap();
        let p = v["p"].as_f64().unwrap();
        assert!((0.0..=8.0 * 9.0 / 2.0).contains(&w), "w = {w}");
        assert!((0.0..=1.0).contains(&p), "p = {p}");
    }

    #[test]
    fn variance_tests_see_no_difference_between_identical_groups() {
        let same = json!([[1.0, 2.0, 3.0, 4.0], [1.0, 2.0, 3.0, 4.0], [1.0, 2.0, 3.0, 4.0]]);
        assert!(field("test.levene", std::slice::from_ref(&same), "p") > 0.99);
        assert!(field("test.bartlett", &[same], "p") > 0.99);
        // And they do see one when the spreads are far apart.
        let different = json!([[1.0, 1.1, 0.9, 1.0], [1.0, 50.0, -50.0, 3.0], [1.0, 1.0, 1.2, 0.8]]);
        assert!(field("test.bartlett", &[different], "p") < 0.05);
    }

    #[test]
    fn exact_categorical_tests_hit_their_boundaries() {
        // A table with no association at all cannot be evidence against
        // independence.
        close(n("test.fisher_exact", &[json!(5.0), json!(5.0), json!(5.0), json!(5.0)]), 1.0);
        // The most likely outcome of a fair coin is the least surprising one.
        close(n("test.binomial_test", &[json!(5), json!(10), json!(0.5)]), 1.0);
        // Every outcome one way is the most extreme, and its p is the tail mass.
        close(n("test.binomial_test", &[json!(10), json!(10), json!(0.5)]), 2.0 / 1024.0);
        let p = n("test.fisher_exact", &[json!(12.0), json!(5.0), json!(8.0), json!(15.0)]);
        assert!((0.0..=1.0).contains(&p), "{p}");
        // Fisher and the corrected chi-square should agree about the direction.
        let chi_p = field("test.chi_square_yates", &[json!(12.0), json!(5.0), json!(8.0), json!(15.0)], "p");
        assert!((p < 0.1) == (chi_p < 0.1), "fisher {p} vs yates {chi_p}");
    }

    #[test]
    fn confidence_intervals_contain_the_estimate_and_widen_with_confidence() {
        let m = A.iter().sum::<f64>() / 8.0;
        let (lo, hi) = pair_of("test.ci_mean", &[a(), json!(0.95)]);
        assert!(lo < m && m < hi, "{lo} < {m} < {hi}");
        let (lo99, hi99) = pair_of("test.ci_mean", &[a(), json!(0.99)]);
        assert!(lo99 < lo && hi < hi99, "99% interval must contain the 95% one");

        let (plo, phi) = pair_of("test.ci_proportion", &[json!(7), json!(20), json!(0.95)]);
        assert!((0.0..=1.0).contains(&plo) && (0.0..=1.0).contains(&phi) && plo < 0.35 && 0.35 < phi);
        // The Wilson interval stays inside [0,1] and keeps a width at the edges,
        // which the normal approximation does not.
        let (zlo, zhi) = pair_of("test.ci_proportion", &[json!(0), json!(20), json!(0.95)]);
        assert!(zlo == 0.0 && zhi > 0.0, "degenerate interval at zero successes: {zlo}..{zhi}");

        let (vlo, vhi) = pair_of("test.ci_variance", &[a(), json!(0.95)]);
        let var = { let mm = m; A.iter().map(|v| (v - mm) * (v - mm)).sum::<f64>() / 7.0 };
        assert!(vlo < var && var < vhi, "{vlo} < {var} < {vhi}");
    }

    #[test]
    fn sample_size_grows_as_the_effect_shrinks_and_power_rises() {
        let small = n("test.sample_size_mean", &[json!(0.2), json!(1.0), json!(0.05), json!(0.8)]);
        let large = n("test.sample_size_mean", &[json!(0.8), json!(1.0), json!(0.05), json!(0.8)]);
        assert!(small > large, "a smaller effect needs a larger sample");
        let more_power = n("test.sample_size_mean", &[json!(0.5), json!(1.0), json!(0.05), json!(0.95)]);
        let less_power = n("test.sample_size_mean", &[json!(0.5), json!(1.0), json!(0.05), json!(0.8)]);
        assert!(more_power > less_power);
        // Power rises with n and with the effect, and stays a probability.
        let p1 = n("test.power_t", &[json!(10), json!(0.5), json!(0.05)]);
        let p2 = n("test.power_t", &[json!(100), json!(0.5), json!(0.05)]);
        assert!(p1 < p2 && (0.0..=1.0).contains(&p1) && (0.0..=1.0).contains(&p2));
    }

    // ------------------------------------------------------------- regression

    #[test]
    fn degree_one_polynomial_is_the_simple_regression_line() {
        let x = json!([1.0, 2.0, 3.0, 4.0, 5.0]);
        let y = json!([2.1, 3.9, 6.2, 7.8, 10.1]);
        let coef = call("reg.polynomial", &[x.clone(), y.clone(), json!(1)]).unwrap();
        let slope = coef[0].as_f64().unwrap();
        let intercept = coef[1].as_f64().unwrap();
        // Recover the same line from the closed form the slope tests use.
        let t = n("reg.slope_t", &[x.clone(), y.clone()]);
        let se = n("reg.slope_se", &[x, y]);
        close(slope, t * se);
        assert!(intercept.is_finite());
    }

    #[test]
    fn multiple_regression_recovers_an_exact_linear_relationship() {
        // y = 3 + 2*x1 - x2 exactly, so the fit must return those numbers and
        // an R squared of one.
        let x = json!([[1.0, 1.0], [2.0, 1.0], [1.0, 2.0], [3.0, 5.0], [4.0, 2.0], [0.0, 3.0]]);
        let y = json!([4.0, 6.0, 3.0, 4.0, 9.0, 0.0]);
        let v = call("reg.multiple", &[x, y]).unwrap();
        close(v["intercept"].as_f64().unwrap(), 3.0);
        close(v["coefficients"][0].as_f64().unwrap(), 2.0);
        close(v["coefficients"][1].as_f64().unwrap(), -1.0);
        close(v["r2"].as_f64().unwrap(), 1.0);
    }

    #[test]
    fn robust_and_least_squares_slopes_agree_on_clean_data_and_part_on_an_outlier() {
        let x = json!([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        let clean = json!([2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0]);
        let ts = field("reg.theil_sen", &[x.clone(), clean.clone()], "slope");
        close(ts, 2.0);
        // One wild point moves least squares and should barely move the median
        // of pairwise slopes.
        let dirty = json!([2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 900.0]);
        let robust = field("reg.theil_sen", &[x.clone(), dirty.clone()], "slope");
        let ls = n("reg.slope_t", &[x.clone(), dirty.clone()]) * n("reg.slope_se", &[x, dirty]);
        assert!((robust - 2.0).abs() < 1.0, "robust slope moved to {robust}");
        assert!(ls > 50.0, "least squares should be dragged by the outlier, got {ls}");
    }

    #[test]
    fn rank_correlation_is_one_on_a_monotone_relationship() {
        let x = json!([1.0, 2.0, 3.0, 4.0, 5.0]);
        close(n("reg.kendall_tau", &[x.clone(), json!([1.0, 4.0, 9.0, 16.0, 25.0])]), 1.0);
        close(n("reg.kendall_tau", &[x.clone(), json!([25.0, 16.0, 9.0, 4.0, 1.0])]), -1.0);
    }

    #[test]
    fn durbin_watson_reports_the_expected_extremes() {
        // Perfectly alternating residuals are maximally negatively correlated.
        // The statistic approaches 4 only asymptotically: for six points the
        // exact value is 20/6, because there are five differences of 2 over six
        // squares of 1.
        let alt = json!([1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);
        close(n("reg.durbin_watson", &[alt]), 20.0 / 6.0);
        // Longer runs get closer to the bound.
        let long_alt: Vec<f64> = (0..200).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        assert!(n("reg.durbin_watson", &[json!(long_alt)]) > 3.9);
        // A constant run has no successive differences at all.
        let flat = json!([2.0, 2.0, 2.0, 2.0]);
        close(n("reg.durbin_watson", &[flat]), 0.0);
    }

    #[test]
    fn information_criteria_prefer_the_model_that_fits() {
        let y = json!([2.1, 3.9, 6.2, 7.8, 10.1, 12.2]);
        let good = json!([2.0, 4.0, 6.0, 8.0, 10.0, 12.0]);
        let poor = json!([5.0, 5.0, 5.0, 5.0, 5.0, 5.0]);
        assert!(n("reg.aic", &[y.clone(), good.clone(), json!(1)])
              < n("reg.aic", &[y.clone(), poor.clone(), json!(1)]));
        // BIC penalises parameters by ln(n) where AIC uses 2, so BIC is the
        // harsher of the two only once ln(n) > 2, that is beyond about seven
        // points. Both directions are pinned so the crossover cannot drift.
        let a6 = n("reg.aic", &[y.clone(), good.clone(), json!(1)]);
        let b6 = n("reg.bic", &[y, good, json!(1)]);
        assert!(b6 < a6, "at n = 6, ln(n) < 2, so bic {b6} should be below aic {a6}");

        let y20: Vec<f64> = (0..20).map(|i| 2.0 * i as f64 + 0.1).collect();
        let fit20: Vec<f64> = (0..20).map(|i| 2.0 * i as f64).collect();
        let a20 = n("reg.aic", &[json!(y20.clone()), json!(fit20.clone()), json!(1)]);
        let b20 = n("reg.bic", &[json!(y20), json!(fit20), json!(1)]);
        assert!(b20 > a20, "at n = 20, bic {b20} should exceed aic {a20}");
    }

    #[test]
    fn inference_guards_reject_impossible_input() {
        assert_eq!(err("test.p_t", &[json!(1.0), json!(0.0), json!(2)]), "DOMAIN");
        assert_eq!(err("test.p_t", &[json!(1.0), json!(5.0), json!(3)]), "DOMAIN");
        assert_eq!(err("test.ci_mean", &[a(), json!(1.0)]), "DOMAIN");
        assert_eq!(err("test.ci_mean", &[a(), json!(0.0)]), "DOMAIN");
        assert_eq!(err("test.binomial_test", &[json!(11), json!(10), json!(0.5)]), "DOMAIN");
        assert_eq!(err("test.anova_one_way", &[json!([[1.0, 2.0]])]), "SHAPE");
        assert_eq!(err("test.t_paired", &[json!([1.0, 2.0, 3.0]), json!([1.0, 2.0])]), "SHAPE");
        // A sample with no spread has no t statistic to report.
        assert_eq!(err("test.t_one_sample_test", &[json!([2.0, 2.0, 2.0]), json!(1.0)]), "DEGENERATE");
        assert_eq!(err("reg.slope_se", &[json!([1.0, 1.0, 1.0]), json!([1.0, 2.0, 3.0])]), "DEGENERATE");
        assert_eq!(err("reg.polynomial", &[json!([1.0, 2.0, 3.0]), json!([1.0, 2.0, 3.0]), json!(99)]), "LIMIT");
        assert_eq!(err("reg.exponential", &[json!([1.0, 2.0, 3.0]), json!([1.0, -2.0, 3.0])]), "DOMAIN");
    }
}
