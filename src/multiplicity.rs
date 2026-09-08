//! Multiplicity control, post-hoc comparisons and effect sizes.
//!
//! The engine gained 27 hypothesis tests in v1.2 and no way to correct for
//! running more than one of them. That is a sharper gap here than in a
//! single-call library: `yk.compute` takes a batch, so submitting twenty tests
//! in one request is the ordinary way to use it, and every one of those twenty
//! p-values was uncorrected. `test.anova_one_way` had the same shape from the
//! other side -- an omnibus p with nothing to say about which groups differ.
//!
//! Effect sizes are here for the same reason. `test.cohen_d` was the only one,
//! so every other test reported significance without magnitude.
//!
//! # Adjusted p-values, not reject/accept flags
//!
//! Each correction returns adjusted p-values in the caller's original order,
//! comparable against any alpha. Returning booleans would bake the threshold
//! into the result and lose the information needed to compare corrections.
//!
//! # Tukey HSD is deliberately absent
//!
//! It needs the studentized range distribution, which is a numerical project of
//! its own: the outer integral converges slowly in the far tail, and a Tukey
//! p-value is *only* read in that tail. Rather than ship a distribution an order
//! of magnitude less accurate than everything else in `special.*`, the post-hoc
//! path here is pairwise t against the pooled within-group variance, corrected
//! by one of the step-down or FDR procedures below. That combination is
//! standard, and every piece of it is exact.

use serde_json::{json, Value};

use crate::advanced_probability::{normal_ppf_std, t_sf};

const MAX_N: usize = 100_000;
const MAX_GROUPS: usize = 1_000;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "test.p_adjust_bonferroni" | "test.p_adjust_sidak" | "test.p_adjust_holm" |
        "test.p_adjust_hochberg" | "test.p_adjust_bh" | "test.p_adjust_by" |
        "test.fdr_threshold" |
        "test.pairwise_t" | "test.pairwise_welch" |
        "test.hedges_g" | "test.glass_delta" |
        "test.eta_squared" | "test.omega_squared" | "test.cohen_f" |
        "test.cramers_v" | "test.phi_coefficient" | "test.cohen_w" | "test.cohen_h" |
        "test.rank_biserial" | "test.cliffs_delta" |
        "test.odds_ratio_ci" | "test.risk_ratio" | "test.risk_ratio_ci"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "test.p_adjust_bonferroni" => adjust(args, Method::Bonferroni),
        "test.p_adjust_sidak" => adjust(args, Method::Sidak),
        "test.p_adjust_holm" => adjust(args, Method::Holm),
        "test.p_adjust_hochberg" => adjust(args, Method::Hochberg),
        "test.p_adjust_bh" => adjust(args, Method::Bh),
        "test.p_adjust_by" => adjust(args, Method::By),
        "test.fdr_threshold" => fdr_threshold(args),

        "test.pairwise_t" => pairwise(args, Pairwise::Pooled),
        "test.pairwise_welch" => pairwise(args, Pairwise::Welch),

        "test.hedges_g" => hedges_g(args),
        "test.glass_delta" => glass_delta(args),
        "test.eta_squared" => variance_explained(args, Explained::Eta),
        "test.omega_squared" => variance_explained(args, Explained::Omega),
        "test.cohen_f" => variance_explained(args, Explained::CohenF),

        "test.cramers_v" => cramers_v(args),
        "test.phi_coefficient" => phi_coefficient(args),
        "test.cohen_w" => cohen_w(args),
        "test.cohen_h" => cohen_h(args),

        "test.rank_biserial" => rank_biserial(args),
        "test.cliffs_delta" => cliffs_delta(args),

        "test.odds_ratio_ci" => ratio_ci(args, Ratio::Odds),
        "test.risk_ratio" => risk_ratio(args),
        "test.risk_ratio_ci" => ratio_ci(args, Ratio::Risk),

        _ => Err("OP"),
    }
}

// ------------------------------------------------------------------ helpers

fn num(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if x.is_finite() { Ok(x) } else { Err("NONFINITE") }
}

fn nonneg(v: &Value) -> Result<f64, &'static str> {
    let x = num(v)?;
    if x < 0.0 { Err("DOMAIN") } else { Ok(x) }
}

fn probability(v: &Value) -> Result<f64, &'static str> {
    let x = num(v)?;
    if !(0.0..=1.0).contains(&x) { Err("DOMAIN") } else { Ok(x) }
}

fn confidence(v: &Value) -> Result<f64, &'static str> {
    let x = num(v)?;
    if x <= 0.0 || x >= 1.0 { Err("DOMAIN") } else { Ok(x) }
}

fn need(args: &[Value], n: usize) -> Result<(), &'static str> {
    if args.len() == n { Ok(()) } else { Err("ARG") }
}

fn finite(x: f64) -> Result<Value, &'static str> {
    if x.is_finite() { Ok(json!(x)) } else { Err("NONFINITE") }
}

fn values(v: &Value) -> Result<Vec<f64>, &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.len() > MAX_N { return Err("LIMIT"); }
    xs.iter().map(num).collect()
}

/// A sample large enough for a variance to exist.
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
    let groups = v.as_array().ok_or("TYPE")?;
    if groups.len() < 2 { return Err("SHAPE"); }
    if groups.len() > MAX_GROUPS { return Err("LIMIT"); }
    groups.iter().map(sample).collect()
}

fn mean(x: &[f64]) -> f64 { x.iter().sum::<f64>() / x.len() as f64 }

/// Sample variance, matching the `n - 1` convention used everywhere else here.
fn var(x: &[f64]) -> Result<f64, &'static str> {
    let n = x.len();
    if n < 2 { return Err("SHAPE"); }
    let m = mean(x);
    Ok(x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (n - 1) as f64)
}

/// Midranks with the tie-correction sum, as in `inference::midranks`.
fn midranks(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|a, b| x[*a].total_cmp(&x[*b]));
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && x[idx[j + 1]] == x[idx[i]] { j += 1; }
        // Ranks are 1-based; a tied block shares the average of its positions.
        let r = (i + j + 2) as f64 / 2.0;
        for &k in &idx[i..=j] { ranks[k] = r; }
        i = j + 1;
    }
    ranks
}

// -------------------------------------------------------------- corrections

#[derive(Clone, Copy)]
enum Method { Bonferroni, Sidak, Holm, Hochberg, Bh, By }

/// Adjusted p-values, returned in the caller's original order.
///
/// The step-down (Holm) and step-up (Hochberg, BH, BY) procedures are defined
/// on the sorted p-values with a running extremum, which is what enforces
/// monotonicity: an adjusted p-value can never fall below one belonging to a
/// smaller raw p-value. Doing it any other way produces a set that can reject a
/// larger p while retaining a smaller one.
fn adjust(args: &[Value], method: Method) -> Result<Value, &'static str> {
    need(args, 1)?;
    let raw = args[0].as_array().ok_or("TYPE")?;
    if raw.is_empty() { return Err("EMPTY"); }
    if raw.len() > MAX_N { return Err("LIMIT"); }
    let p: Vec<f64> = raw.iter().map(probability).collect::<Result<_, _>>()?;
    let m = p.len();
    let mf = m as f64;

    let mut out = vec![0.0; m];
    match method {
        Method::Bonferroni => {
            for (i, v) in p.iter().enumerate() { out[i] = (v * mf).min(1.0); }
        }
        Method::Sidak => {
            // 1 - (1-p)^m via ln1p/expm1 so a small p keeps its relative
            // accuracy; the naive form loses it to the subtraction.
            for (i, v) in p.iter().enumerate() {
                out[i] = (-((mf * (-v).ln_1p()).exp_m1())).min(1.0);
            }
        }
        Method::Holm => {
            // Ascending: multiplier m - i, running maximum.
            let mut idx: Vec<usize> = (0..m).collect();
            idx.sort_by(|a, b| p[*a].total_cmp(&p[*b]));
            let mut running: f64 = 0.0;
            for (i, &k) in idx.iter().enumerate() {
                let v = ((m - i) as f64 * p[k]).min(1.0);
                running = running.max(v);
                out[k] = running;
            }
        }
        Method::Hochberg => {
            // Descending: multiplier i + 1 counting from the largest, running
            // minimum.
            let mut idx: Vec<usize> = (0..m).collect();
            idx.sort_by(|a, b| p[*b].total_cmp(&p[*a]));
            let mut running = f64::INFINITY;
            for (i, &k) in idx.iter().enumerate() {
                let v = ((i + 1) as f64 * p[k]).min(1.0);
                running = running.min(v);
                out[k] = running;
            }
        }
        Method::Bh | Method::By => {
            let c = match method {
                // Benjamini-Yekutieli's correction for arbitrary dependence.
                Method::By => (1..=m).map(|i| 1.0 / i as f64).sum::<f64>(),
                _ => 1.0,
            };
            let mut idx: Vec<usize> = (0..m).collect();
            idx.sort_by(|a, b| p[*b].total_cmp(&p[*a]));
            let mut running = f64::INFINITY;
            for (i, &k) in idx.iter().enumerate() {
                let rank = (m - i) as f64;
                let v = (c * mf / rank * p[k]).min(1.0);
                running = running.min(v);
                out[k] = running;
            }
        }
    }
    if out.iter().any(|v| !v.is_finite()) { return Err("NONFINITE"); }
    Ok(json!(out))
}

/// Largest raw p-value that Benjamini-Hochberg rejects at level `q`, or 0 when
/// the procedure rejects nothing.
///
/// Returned as the threshold rather than as a count so the caller can apply it
/// to the p-values it already holds without re-deriving the ordering.
fn fdr_threshold(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let raw = args[0].as_array().ok_or("TYPE")?;
    if raw.is_empty() { return Err("EMPTY"); }
    if raw.len() > MAX_N { return Err("LIMIT"); }
    let mut p: Vec<f64> = raw.iter().map(probability).collect::<Result<_, _>>()?;
    let q = confidence(&args[1])?;
    p.sort_by(f64::total_cmp);
    let m = p.len() as f64;
    let mut threshold = 0.0;
    for (i, v) in p.iter().enumerate() {
        if *v <= (i + 1) as f64 * q / m { threshold = *v; }
    }
    finite(threshold)
}

// ------------------------------------------------------------ post-hoc pairs

#[derive(Clone, Copy)]
enum Pairwise { Pooled, Welch }

/// Every pair of groups, in index order.
///
/// `Pooled` uses the within-group variance from the one-way ANOVA, so every
/// comparison shares `n - k` degrees of freedom; that is the test the omnibus F
/// decomposes into, and the one the corrections above are normally applied to.
/// `Welch` estimates the standard error and the degrees of freedom per pair
/// instead, for groups whose variances are not comparable.
///
/// Raw p-values are returned. Feed them to `test.p_adjust_*`.
fn pairwise(args: &[Value], mode: Pairwise) -> Result<Value, &'static str> {
    need(args, 1)?;
    let groups = group_list(&args[0])?;
    let k = groups.len();
    let n: usize = groups.iter().map(Vec::len).sum();
    if n <= k { return Err("SHAPE"); }

    let pooled_df = (n - k) as f64;
    let ssw: f64 = groups.iter().map(|g| {
        let m = mean(g);
        g.iter().map(|v| (v - m) * (v - m)).sum::<f64>()
    }).sum();
    let msw = ssw / pooled_df;

    let mut out = Vec::with_capacity(k * (k - 1) / 2);
    for i in 0..k {
        for j in (i + 1)..k {
            let (a, b) = (&groups[i], &groups[j]);
            let (na, nb) = (a.len() as f64, b.len() as f64);
            let diff = mean(a) - mean(b);
            let (se2, df) = match mode {
                Pairwise::Pooled => (msw * (1.0 / na + 1.0 / nb), pooled_df),
                Pairwise::Welch => {
                    let (va, vb) = (var(a)?, var(b)?);
                    let se2 = va / na + vb / nb;
                    let df = se2 * se2
                        / ((va / na).powi(2) / (na - 1.0) + (vb / nb).powi(2) / (nb - 1.0));
                    (se2, df)
                }
            };
            if se2 <= 0.0 || !df.is_finite() || df <= 0.0 { return Err("DEGENERATE"); }
            let t = diff / se2.sqrt();
            if !t.is_finite() { return Err("NONFINITE"); }
            out.push(json!({
                "i": i, "j": j, "diff": diff, "t": t, "df": df,
                "p": 2.0 * t_sf(t.abs(), df)
            }));
        }
    }
    Ok(json!(out))
}

// -------------------------------------------------------------- effect sizes

/// Pooled standard deviation, the same one `test.cohen_d` uses.
fn pooled_sd(a: &[f64], b: &[f64]) -> Result<f64, &'static str> {
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let sp2 = ((na - 1.0) * var(a)? + (nb - 1.0) * var(b)?) / (na + nb - 2.0);
    if sp2 <= 0.0 { return Err("DEGENERATE"); }
    Ok(sp2.sqrt())
}

/// Hedges' g: Cohen's d with the small-sample bias factor.
///
/// The exact factor is a ratio of gamma functions; the standard approximation
/// used here is within 1e-4 of it for the smallest usable samples and is what
/// every reference implementation reports, so it is the comparable value.
fn hedges_g(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two_samples(args)?;
    let d = (mean(&a) - mean(&b)) / pooled_sd(&a, &b)?;
    let n = (a.len() + b.len()) as f64;
    if n <= 2.25 { return Err("SHAPE"); }
    finite(d * (1.0 - 3.0 / (4.0 * n - 9.0)))
}

/// Glass's delta: standardised by the *second* sample's standard deviation.
///
/// The second sample is the control. Using only its spread is the whole point:
/// when a treatment changes the variance, a pooled denominator is a function of
/// the effect being measured.
fn glass_delta(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two_samples(args)?;
    let sd = var(&b)?.sqrt();
    if sd == 0.0 { return Err("DEGENERATE"); }
    finite((mean(&a) - mean(&b)) / sd)
}

#[derive(Clone, Copy)]
enum Explained { Eta, Omega, CohenF }

/// One-way variance-explained measures from the same sums of squares
/// `test.anova_one_way` uses.
///
/// `Eta` is the sample proportion and is biased upward; `Omega` estimates the
/// population value and can go negative, which is reported rather than clamped
/// because a negative estimate is the signal that the grouping explains less
/// than noise would.
fn variance_explained(args: &[Value], which: Explained) -> Result<Value, &'static str> {
    need(args, 1)?;
    let groups = group_list(&args[0])?;
    let k = groups.len();
    let n: usize = groups.iter().map(Vec::len).sum();
    if n <= k { return Err("SHAPE"); }
    let grand = groups.iter().flatten().sum::<f64>() / n as f64;
    let ssb: f64 = groups.iter()
        .map(|g| g.len() as f64 * (mean(g) - grand).powi(2))
        .sum();
    let ssw: f64 = groups.iter().flat_map(|g| {
        let m = mean(g);
        g.iter().map(move |v| (v - m) * (v - m))
    }).sum();
    let sst = ssb + ssw;
    if sst == 0.0 { return Err("DEGENERATE"); }
    let eta = ssb / sst;
    match which {
        Explained::Eta => finite(eta),
        Explained::Omega => {
            let msw = ssw / (n - k) as f64;
            finite((ssb - (k - 1) as f64 * msw) / (sst + msw))
        }
        Explained::CohenF => {
            if eta >= 1.0 { return Err("DEGENERATE"); }
            finite((eta / (1.0 - eta)).sqrt())
        }
    }
}

fn table(v: &Value) -> Result<Vec<Vec<f64>>, &'static str> {
    let rows = v.as_array().ok_or("TYPE")?;
    if rows.len() < 2 { return Err("SHAPE"); }
    let mut out = Vec::with_capacity(rows.len());
    let mut cols = None;
    for r in rows {
        let row = values(r)?;
        if row.len() < 2 { return Err("SHAPE"); }
        match cols {
            Some(c) if c != row.len() => return Err("SHAPE"),
            Some(_) => {}
            None => cols = Some(row.len()),
        }
        if row.iter().any(|x| *x < 0.0) { return Err("DOMAIN"); }
        out.push(row);
    }
    Ok(out)
}

/// Pearson chi-square of independence for a contingency table.
fn contingency_chi2(t: &[Vec<f64>]) -> Result<(f64, f64, usize, usize), &'static str> {
    let rows = t.len();
    let cols = t[0].len();
    let row_sum: Vec<f64> = t.iter().map(|r| r.iter().sum()).collect();
    let mut col_sum = vec![0.0; cols];
    for r in t { for (j, x) in r.iter().enumerate() { col_sum[j] += x; } }
    let total: f64 = row_sum.iter().sum();
    if total == 0.0 { return Err("DOMAIN"); }
    let mut chi2 = 0.0;
    for i in 0..rows {
        for j in 0..cols {
            let e = row_sum[i] * col_sum[j] / total;
            if e == 0.0 { return Err("DEGENERATE"); }
            let d = t[i][j] - e;
            chi2 += d * d / e;
        }
    }
    Ok((chi2, total, rows, cols))
}

/// Cramer's V. For a 2x2 table this equals |phi|.
fn cramers_v(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?;
    let t = table(&args[0])?;
    let (chi2, n, rows, cols) = contingency_chi2(&t)?;
    let min_dim = (rows.min(cols) - 1) as f64;
    if min_dim == 0.0 { return Err("SHAPE"); }
    finite((chi2 / (n * min_dim)).sqrt())
}

/// Signed phi for a 2x2 table given as four counts.
///
/// Signed, unlike Cramer's V, because with two categories the direction of the
/// association is meaningful and recoverable.
fn phi_coefficient(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 4)?;
    let (a, b, c, d) = (nonneg(&args[0])?, nonneg(&args[1])?, nonneg(&args[2])?, nonneg(&args[3])?);
    let denom = (a + b) * (c + d) * (a + c) * (b + d);
    if denom == 0.0 { return Err("DEGENERATE"); }
    finite((a * d - b * c) / denom.sqrt())
}

/// Cohen's w from observed and expected counts (or proportions).
///
/// Both are normalised to proportions first, so the same call works whichever
/// the caller has.
fn cohen_w(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let o = values(&args[0])?;
    let e = values(&args[1])?;
    if o.len() != e.len() || o.is_empty() { return Err("SHAPE"); }
    if o.iter().chain(&e).any(|x| *x < 0.0) { return Err("DOMAIN"); }
    let (so, se) = (o.iter().sum::<f64>(), e.iter().sum::<f64>());
    if so == 0.0 || se == 0.0 { return Err("DEGENERATE"); }
    let mut sum = 0.0;
    for (a, b) in o.iter().zip(&e) {
        let (po, pe) = (a / so, b / se);
        if pe == 0.0 { return Err("DEGENERATE"); }
        sum += (po - pe) * (po - pe) / pe;
    }
    finite(sum.sqrt())
}

/// Cohen's h for two proportions, on the arcsine-transformed scale.
fn cohen_h(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 2)?;
    let p1 = probability(&args[0])?;
    let p2 = probability(&args[1])?;
    finite(2.0 * p1.sqrt().asin() - 2.0 * p2.sqrt().asin())
}

/// Mann-Whitney U for the first sample, tie-aware through midranks.
fn u_statistic(a: &[f64], b: &[f64]) -> f64 {
    let mut all = a.to_vec();
    all.extend_from_slice(b);
    let ranks = midranks(&all);
    let r1: f64 = ranks[..a.len()].iter().sum();
    let n1 = a.len() as f64;
    r1 - n1 * (n1 + 1.0) / 2.0
}

/// Rank-biserial correlation, the effect size belonging to
/// `test.mann_whitney_u`: (U1 - U2) / (n1 * n2), so it runs from -1 to 1.
fn rank_biserial(args: &[Value]) -> Result<Value, &'static str> {
    let (a, b) = two_samples(args)?;
    let (n1, n2) = (a.len() as f64, b.len() as f64);
    let u1 = u_statistic(&a, &b);
    let u2 = n1 * n2 - u1;
    finite((u1 - u2) / (n1 * n2))
}

/// Cliff's delta: the probability that a value from the first sample exceeds
/// one from the second, minus the reverse.
///
/// Computed by sorting rather than by the O(n1*n2) double loop, so it stays
/// usable on the sample sizes this engine's limits allow.
fn cliffs_delta(args: &[Value]) -> Result<Value, &'static str> {
    let (a, mut b) = two_samples(args)?;
    b.sort_by(f64::total_cmp);
    let n2 = b.len();
    let mut greater = 0.0;
    let mut less = 0.0;
    for x in &a {
        // lower = count of b strictly below x, upper = count strictly above.
        let lo = b.partition_point(|v| v.total_cmp(x).is_lt());
        let hi = b.partition_point(|v| v.total_cmp(x).is_le());
        greater += lo as f64;
        less += (n2 - hi) as f64;
    }
    finite((greater - less) / (a.len() as f64 * n2 as f64))
}

#[derive(Clone, Copy)]
enum Ratio { Odds, Risk }

fn risk_ratio(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 4)?;
    let (a, b, c, d) = (nonneg(&args[0])?, nonneg(&args[1])?, nonneg(&args[2])?, nonneg(&args[3])?);
    let (n1, n2) = (a + b, c + d);
    if n1 == 0.0 || n2 == 0.0 || c == 0.0 { return Err("DEGENERATE"); }
    finite((a / n1) / (c / n2))
}

/// Wald interval on the log scale for either ratio.
///
/// Built on the log because both ratios are bounded below by zero and skewed;
/// a symmetric interval on the raw scale can cover negative values.
fn ratio_ci(args: &[Value], which: Ratio) -> Result<Value, &'static str> {
    need(args, 5)?;
    let (a, b, c, d) = (nonneg(&args[0])?, nonneg(&args[1])?, nonneg(&args[2])?, nonneg(&args[3])?);
    let conf = confidence(&args[4])?;
    if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 { return Err("DEGENERATE"); }
    let z = normal_ppf_std(0.5 + conf / 2.0);
    let (point, se) = match which {
        Ratio::Odds => (
            ((a * d) / (b * c)).ln(),
            (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt(),
        ),
        Ratio::Risk => {
            let (n1, n2) = (a + b, c + d);
            (
                ((a / n1) / (c / n2)).ln(),
                (1.0 / a - 1.0 / n1 + 1.0 / c - 1.0 / n2).sqrt(),
            )
        }
    };
    if !point.is_finite() || !se.is_finite() { return Err("NONFINITE"); }
    Ok(json!([(point - z * se).exp(), (point + z * se).exp()]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok(op: &str, args: &[Value]) -> Value {
        execute(op, args).unwrap_or_else(|| panic!("{op} not routed")).unwrap()
    }

    fn err(op: &str, args: &[Value]) -> &'static str {
        execute(op, args).unwrap_or_else(|| panic!("{op} not routed")).unwrap_err()
    }

    fn vec_of(v: &Value) -> Vec<f64> {
        v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect()
    }

    const P: [f64; 6] = [0.001, 0.008, 0.039, 0.041, 0.2, 0.7];

    fn ps() -> Vec<Value> { vec![json!(P.to_vec())] }

    /// Every correction is at least as large as the raw p-value and never
    /// exceeds 1. A procedure violating either would not be a correction.
    #[test]
    fn corrections_stay_between_the_raw_p_and_one() {
        for op in ["test.p_adjust_bonferroni", "test.p_adjust_sidak", "test.p_adjust_holm",
                   "test.p_adjust_hochberg", "test.p_adjust_bh", "test.p_adjust_by"] {
            let adj = vec_of(&ok(op, &ps()));
            for (a, raw) in adj.iter().zip(P.iter()) {
                assert!(*a >= *raw - 1e-15, "{op}: {a} below raw {raw}");
                assert!(*a <= 1.0, "{op}: {a} above 1");
            }
        }
    }

    /// Bonferroni is the most conservative, Holm is uniformly no larger, and BH
    /// no larger again. This ordering is the reason to offer more than one.
    #[test]
    fn corrections_are_ordered_bonferroni_holm_bh() {
        let bonf = vec_of(&ok("test.p_adjust_bonferroni", &ps()));
        let holm = vec_of(&ok("test.p_adjust_holm", &ps()));
        let bh = vec_of(&ok("test.p_adjust_bh", &ps()));
        let by = vec_of(&ok("test.p_adjust_by", &ps()));
        for i in 0..P.len() {
            assert!(bonf[i] >= holm[i] - 1e-15, "bonferroni below holm at {i}");
            assert!(holm[i] >= bh[i] - 1e-15, "holm below bh at {i}");
            assert!(by[i] >= bh[i] - 1e-15, "by below bh at {i}");
        }
    }

    /// Sidak is never more conservative than Bonferroni: 1 - (1-p)^m <= m*p.
    #[test]
    fn sidak_never_exceeds_bonferroni() {
        let s = vec_of(&ok("test.p_adjust_sidak", &ps()));
        let b = vec_of(&ok("test.p_adjust_bonferroni", &ps()));
        for i in 0..P.len() {
            assert!(s[i] <= b[i] + 1e-15, "sidak {} above bonferroni {} at {i}", s[i], b[i]);
        }
    }

    /// Sidak keeps relative accuracy where the naive `1 - (1-p)^m` does not.
    /// At p = 1e-12 with m = 2 the true value is 2e-12 - 1e-24; the naive route
    /// in f64 loses roughly five digits of it.
    #[test]
    fn sidak_is_accurate_for_a_tiny_p() {
        let adj = vec_of(&ok("test.p_adjust_sidak", &[json!([1e-12, 0.5])]));
        let naive = 1.0 - (1.0f64 - 1e-12).powi(2);
        let exact = 2e-12 - 1e-24;
        assert!((adj[0] - exact).abs() / exact < 1e-12, "got {}", adj[0]);
        assert!((naive - exact).abs() / exact > 1e-6,
            "the naive form was expected to be inaccurate, so this test would not prove anything");
    }

    /// With every p-value equal there is no ordering for a step procedure to
    /// exploit, so Holm degenerates to Bonferroni.
    #[test]
    fn holm_equals_bonferroni_when_every_p_is_equal() {
        let flat = vec![json!([0.02, 0.02, 0.02, 0.02])];
        assert_eq!(ok("test.p_adjust_holm", &flat), ok("test.p_adjust_bonferroni", &flat));
    }

    /// A single test cannot be corrected for multiplicity.
    #[test]
    fn one_p_value_is_returned_unchanged() {
        for op in ["test.p_adjust_bonferroni", "test.p_adjust_holm", "test.p_adjust_bh",
                   "test.p_adjust_hochberg", "test.p_adjust_by"] {
            let adj = vec_of(&ok(op, &[json!([0.037])]));
            assert!((adj[0] - 0.037).abs() < 1e-15, "{op} changed a lone p value");
        }
    }

    /// Adjusted values must not reorder the raw ones.
    #[test]
    fn corrections_are_monotone_in_the_raw_ordering() {
        for op in ["test.p_adjust_holm", "test.p_adjust_hochberg", "test.p_adjust_bh",
                   "test.p_adjust_by"] {
            let adj = vec_of(&ok(op, &ps()));
            let mut idx: Vec<usize> = (0..P.len()).collect();
            idx.sort_by(|a, b| P[*a].total_cmp(&P[*b]));
            for w in idx.windows(2) {
                assert!(adj[w[0]] <= adj[w[1]] + 1e-15, "{op} is not monotone");
            }
        }
    }

    /// The threshold and the adjusted p-values must reject the same set:
    /// `p <= threshold` iff `bh_adjusted <= q`. This holds when the threshold is
    /// zero in both directions -- nothing is rejected when no p is zero, and
    /// exactly the zeros are when one is.
    #[test]
    fn fdr_threshold_and_bh_reject_the_same_set() {
        for set in [P.to_vec(), vec![0.0, 0.049, 0.051, 1.0], vec![0.9, 0.95, 0.99]] {
            for q in [0.01, 0.05, 0.2] {
                let arg = vec![json!(set.clone()), json!(q)];
                let thr = ok("test.fdr_threshold", &arg).as_f64().unwrap();
                let adj = vec_of(&ok("test.p_adjust_bh", &[json!(set.clone())]));
                for (i, raw) in set.iter().enumerate() {
                    assert_eq!(*raw <= thr, adj[i] <= q + 1e-15,
                        "disagreement at index {i} of {set:?} with q={q}, threshold={thr}");
                }
            }
        }
    }

    /// Two groups reduce to the pooled two-sample t test the engine already has.
    #[test]
    fn pairwise_t_on_two_groups_matches_the_existing_two_sample_test() {
        let a = json!([1.0, 2.0, 3.0, 4.0, 6.0]);
        let b = json!([3.0, 5.0, 5.0, 7.0, 9.0]);
        let pairs = ok("test.pairwise_t", &[json!([a.clone(), b.clone()])]);
        let row = &pairs.as_array().unwrap()[0];
        let direct = crate::engine::execute("test.t_two_equal_test", &[a, b]).unwrap();
        for key in ["t", "df", "p"] {
            let (x, y) = (row[key].as_f64().unwrap(), direct[key].as_f64().unwrap());
            assert!((x - y).abs() <= 1e-12 * y.abs().max(1.0), "{key}: {x} vs {y}");
        }
    }

    /// Welch degrees of freedom fall below the pooled ones and stay at or above
    /// the smaller group's n-1, which is the property the correction provides.
    #[test]
    fn welch_degrees_of_freedom_sit_below_the_pooled_ones() {
        let groups = json!([[1.0, 2.0, 3.0, 4.0], [10.0, 30.0, 50.0, 70.0, 90.0]]);
        let welch = ok("test.pairwise_welch", std::slice::from_ref(&groups));
        let pooled = ok("test.pairwise_t", &[groups]);
        let wdf = welch[0]["df"].as_f64().unwrap();
        let pdf = pooled[0]["df"].as_f64().unwrap();
        assert!(wdf < pdf, "welch df {wdf} is not below the pooled {pdf}");
        assert!(wdf >= 3.0, "welch df {wdf} fell below the smaller group n-1");
    }

    #[test]
    fn pairwise_returns_every_unordered_pair_in_index_order() {
        let groups = json!([[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 9.0]]);
        let rows = ok("test.pairwise_t", &[groups]);
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.len(), 6);
        let seen: Vec<(u64, u64)> = rows.iter()
            .map(|r| (r["i"].as_u64().unwrap(), r["j"].as_u64().unwrap()))
            .collect();
        assert_eq!(seen, vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
    }

    /// The bias correction always shrinks Cohen d toward zero.
    #[test]
    fn hedges_g_is_cohen_d_shrunk() {
        let a = json!([1.0, 2.0, 3.0, 4.0, 5.0]);
        let b = json!([3.0, 4.0, 5.0, 6.0, 8.0]);
        let g = ok("test.hedges_g", &[a.clone(), b.clone()]).as_f64().unwrap();
        let d = crate::engine::execute("test.cohen_d", &[a, b]).unwrap().as_f64().unwrap();
        assert!(g.abs() < d.abs(), "hedges g {g} did not shrink cohen d {d}");
        assert_eq!(g.signum(), d.signum());
    }

    /// On a 2x2 table Cramer V is the magnitude of phi.
    #[test]
    fn cramers_v_is_absolute_phi_on_a_two_by_two() {
        let cells = [json!(10.0), json!(20.0), json!(30.0), json!(25.0)];
        let phi = ok("test.phi_coefficient", &cells).as_f64().unwrap();
        let v = ok("test.cramers_v", &[json!([[10.0, 20.0], [30.0, 25.0]])]).as_f64().unwrap();
        assert!((v - phi.abs()).abs() < 1e-12, "V {v} against absolute phi {}", phi.abs());
        assert!(phi < 0.0, "this table has a negative association");
    }

    /// Without ties the two dominance measures are the same number, and complete
    /// separation pins it at -1.
    #[test]
    fn cliffs_delta_equals_the_rank_biserial_correlation() {
        let a = json!([1.0, 2.0, 3.0, 4.0, 5.0]);
        let b = json!([6.0, 7.0, 8.0, 9.0]);
        let d = ok("test.cliffs_delta", &[a.clone(), b.clone()]).as_f64().unwrap();
        let r = ok("test.rank_biserial", &[a, b]).as_f64().unwrap();
        assert!((d - r).abs() < 1e-12, "{d} against {r}");
        assert!((d + 1.0).abs() < 1e-12, "complete separation should give -1, got {d}");
    }

    /// Omega squared corrects eta squared downward, and Cohen f is the monotone
    /// transform of eta squared it is defined to be.
    #[test]
    fn variance_explained_measures_agree_with_each_other() {
        let groups = json!([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]]);
        let eta = ok("test.eta_squared", std::slice::from_ref(&groups)).as_f64().unwrap();
        let omega = ok("test.omega_squared", std::slice::from_ref(&groups)).as_f64().unwrap();
        let f = ok("test.cohen_f", &[groups]).as_f64().unwrap();
        assert!((0.0..=1.0).contains(&eta), "eta {eta} outside [0,1]");
        assert!(omega < eta, "omega {omega} is not below eta {eta}");
        assert!((f - (eta / (1.0 - eta)).sqrt()).abs() < 1e-12);
    }

    /// Identical groups explain nothing, and omega goes negative there rather
    /// than being clamped -- that sign is the informative part.
    #[test]
    fn omega_squared_goes_negative_when_grouping_explains_nothing() {
        let groups = json!([[1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]);
        let eta = ok("test.eta_squared", std::slice::from_ref(&groups)).as_f64().unwrap();
        let omega = ok("test.omega_squared", &[groups]).as_f64().unwrap();
        assert!(eta.abs() < 1e-15, "eta {eta} should be zero");
        assert!(omega < 0.0, "omega {omega} should be negative");
    }

    /// Both intervals bracket their point estimate and widen with confidence.
    #[test]
    fn ratio_intervals_contain_their_point_estimate() {
        let cells = [json!(10.0), json!(20.0), json!(30.0), json!(25.0)];
        let rr = ok("test.risk_ratio", &cells).as_f64().unwrap();
        let mut args = cells.to_vec();
        args.push(json!(0.95));
        let ci95 = vec_of(&ok("test.risk_ratio_ci", &args));
        assert!(ci95[0] <= rr && rr <= ci95[1], "{rr} outside {ci95:?}");
        args.pop();
        args.push(json!(0.99));
        let ci99 = vec_of(&ok("test.risk_ratio_ci", &args));
        assert!(ci99[0] < ci95[0] && ci99[1] > ci95[1], "the 99 percent interval is not wider");

        let or = (10.0 * 25.0) / (20.0 * 30.0);
        args.pop();
        args.push(json!(0.95));
        let oci = vec_of(&ok("test.odds_ratio_ci", &args));
        assert!(oci[0] <= or && or <= oci[1], "{or} outside {oci:?}");
    }

    /// Cohen h is antisymmetric and zero on equal proportions.
    #[test]
    fn cohen_h_is_antisymmetric() {
        let h = ok("test.cohen_h", &[json!(0.25), json!(0.75)]).as_f64().unwrap();
        let back = ok("test.cohen_h", &[json!(0.75), json!(0.25)]).as_f64().unwrap();
        assert!((h + back).abs() < 1e-15);
        let zero = ok("test.cohen_h", &[json!(0.4), json!(0.4)]).as_f64().unwrap();
        assert!(zero.abs() < 1e-15);
    }

    /// Cohen w reduces to sqrt(chi2 / n) when both arguments are counts over the
    /// same total.
    #[test]
    fn cohen_w_matches_the_chi_square_it_is_built_from() {
        let o = json!([10.0, 20.0, 30.0]);
        let e = json!([20.0, 20.0, 20.0]);
        let w = ok("test.cohen_w", &[o.clone(), e.clone()]).as_f64().unwrap();
        let chi2 = crate::engine::execute("test.chi_square_gof", &[o, e])
            .unwrap().as_f64().unwrap();
        assert!((w - (chi2 / 60.0).sqrt()).abs() < 1e-12, "w {w}, chi2 {chi2}");
    }

    #[test]
    fn guards_return_their_documented_codes() {
        assert_eq!(err("test.p_adjust_bh", &[json!([])]), "EMPTY");
        assert_eq!(err("test.p_adjust_bh", &[json!([0.5, 1.5])]), "DOMAIN");
        assert_eq!(err("test.p_adjust_bh", &[json!([0.5, -0.1])]), "DOMAIN");
        assert_eq!(err("test.p_adjust_bh", &[json!([0.5]), json!(1)]), "ARG");
        assert_eq!(err("test.fdr_threshold", &[json!([0.1]), json!(1.0)]), "DOMAIN");
        assert_eq!(err("test.pairwise_t", &[json!([[1.0, 2.0]])]), "SHAPE");
        assert_eq!(err("test.pairwise_t", &[json!([[1.0], [2.0]])]), "SHAPE");
        assert_eq!(err("test.cohen_h", &[json!(1.5), json!(0.5)]), "DOMAIN");
        assert_eq!(err("test.phi_coefficient",
            &[json!(0.0), json!(0.0), json!(1.0), json!(1.0)]), "DEGENERATE");
        assert_eq!(err("test.risk_ratio",
            &[json!(1.0), json!(1.0), json!(0.0), json!(1.0)]), "DEGENERATE");
        assert_eq!(err("test.glass_delta",
            &[json!([1.0, 1.0]), json!([2.0, 2.0])]), "DEGENERATE");
        assert_eq!(err("test.cramers_v", &[json!([[0.0, 0.0], [0.0, 0.0]])]), "DOMAIN");
    }

    /// Nothing here is routed by prefix, so an opcode this module does not own
    /// must fall through rather than be answered.
    #[test]
    fn unowned_test_opcodes_fall_through() {
        assert!(execute("test.t_welch_test", &[]).is_none());
        assert!(execute("test.anova_one_way", &[]).is_none());
        assert!(execute("stat.mean", &[]).is_none());
    }
}
