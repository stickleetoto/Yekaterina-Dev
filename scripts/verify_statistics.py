"""Verify the statistical operations against scipy and mpmath.

Run:  python scripts/verify_statistics.py
Needs a release binary, plus scipy and mpmath as independent references.

Three layers, because an error in the foundation would otherwise show up as a
plausible-looking p-value rather than as a failure:

  1. the regularized incomplete gamma and beta, against mpmath at 50 digits;
  2. the distributions built on them, against scipy, including that each
     survival function keeps relative accuracy where 1 - cdf does not and that
     each quantile inverts its own CDF;
  3. the hypothesis tests, confidence intervals and regressions, against scipy
     and numpy.

Where a convention differs between implementations the difference is stated in
the check rather than hidden by a loose tolerance -- the Kolmogorov-Smirnov
p-value here is asymptotic and is compared against the asymptotic form, and
Theil-Sen uses the separate-median intercept.
"""
from __future__ import annotations

import json
import math
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bench"))
from bench_client import BenchClient, mcp_text  # noqa: E402

try:
    import mpmath as mp
    import numpy as np
    from scipy import special as sp
    from scipy import stats
except ImportError as exc:                                  # pragma: no cover
    print(f"SKIP: verify_statistics needs scipy, numpy and mpmath ({exc})")
    raise SystemExit(0)

mp.mp.dps = 50

_exe = ROOT / "target/release/yekaterina.exe"
EXE = str(_exe if _exe.exists() else ROOT / "target/release/yekaterina")

# ==================== from check_special.py ====================




A_GRID = [0.05, 0.5, 1.0, 2.5, 10.0, 50.0, 500.0]
X_GRID = [1e-6, 0.01, 0.5, 1.0, 3.0, 25.0, 200.0, 1000.0]
BETA_AB = [(0.5, 0.5), (1.0, 1.0), (2.0, 3.0), (5.0, 0.5), (30.0, 40.0), (250.0, 3.0)]
BETA_X = [1e-8, 0.001, 0.1, 0.4, 0.5, 0.6, 0.9, 0.999, 1 - 1e-9]


def rel(got, want):
    """Relative error, falling back to absolute where the true value underflows
    to a denormal and a relative comparison stops being meaningful."""
    if abs(want) < 1e-290:
        return abs(got - want)
    return abs(got - want) / abs(want)


def main_special():
    worst_p = worst_q = worst_b = 0.0
    worst_case = {}
    skipped = []
    checked = 0
    with tempfile.TemporaryDirectory() as home:
        with BenchClient(EXE,
                         env={"YEKATERINA_HOME": home}, timeout=120) as c:
            def call(op, a):
                return json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))

            for a in A_GRID:
                for x in X_GRID:
                    p = call("special.gamma_p", [a, x])
                    q = call("special.gamma_q", [a, x])
                    assert "r" in p and "r" in q, (a, x, p, q)
                    checked += 2
                    # mpmath at 50 digits is the reference; scipy is checked
                    # against it and only used where the two agree, because in
                    # the far tail scipy itself loses relative accuracy.
                    mp_p = float(mp.gammainc(a, 0, x, regularized=True))
                    mp_q = float(mp.gammainc(a, x, mp.inf, regularized=True))
                    sp_p, sp_q = float(sp.gammainc(a, x)), float(sp.gammaincc(a, x))
                    if rel(sp_p, mp_p) > 1e-13 or rel(sp_q, mp_q) > 1e-13:
                        skipped.append(("gamma", a, x, sp_p, mp_p, sp_q, mp_q))
                    ep, eq = rel(p["r"], mp_p), rel(q["r"], mp_q)
                    if ep > worst_p:
                        worst_p, worst_case["gamma_p"] = ep, (a, x, p["r"], mp_p)
                    if eq > worst_q:
                        worst_q, worst_case["gamma_q"] = eq, (a, x, q["r"], mp_q)
                    # P + Q must be 1 to full precision, which a naive
                    # 1-minus implementation loses in the tail.
                    assert abs(p["r"] + q["r"] - 1.0) < 1e-14, (a, x, p["r"], q["r"])

            for (a, b) in BETA_AB:
                for x in BETA_X:
                    r = call("special.beta_inc", [a, b, x])
                    assert "r" in r, (a, b, x, r)
                    checked += 1
                    mp_want = float(mp.betainc(a, b, 0, x, regularized=True))
                    sp_want = float(sp.betainc(a, b, x))
                    if rel(sp_want, mp_want) > 1e-12:
                        skipped.append(("beta", a, b, x, sp_want, mp_want))
                    e = rel(r["r"], mp_want)
                    if e > worst_b:
                        worst_b, worst_case["beta_inc"] = e, (a, b, x, r["r"], mp_want)
                    # The reflection identity, which the two branches must obey.
                    mirror = call("special.beta_inc", [b, a, 1 - x])
                    checked += 1
                    assert abs(r["r"] + mirror["r"] - 1.0) < 1e-12, (a, b, x)

    print(f"checked {checked} values against mpmath at 50 digits")
    if skipped:
        print(f"  {len(skipped)} points where scipy itself disagrees with mpmath "
              f"(deep tail); mpmath used as the reference there")
    print(f"  worst relative error  gamma_p {worst_p:.3e}")
    print(f"                        gamma_q {worst_q:.3e}")
    print(f"                        beta_inc {worst_b:.3e}")
    for k, v in worst_case.items():
        print(f"  worst {k}: {v}")
    limit = 1e-12
    if max(worst_p, worst_q, worst_b) > limit:
        print(f"FAIL: relative error exceeds {limit:g}")
        raise SystemExit(1)
    print("PASS: incomplete gamma and beta agree with scipy and mpmath")




# ==================== from check_dists.py ====================



DF = [1.0, 2.0, 5.0, 30.0, 200.0]
XS = [-40.0, -6.0, -2.5, -0.5, 0.0, 0.5, 2.5, 6.0, 40.0]
POS = [1e-4, 0.5, 1.0, 4.0, 25.0, 120.0]
PS = [1e-8, 0.001, 0.025, 0.5, 0.95, 0.975, 0.999, 1 - 1e-9]


def rel(got, want):
    if abs(want) < 1e-300:
        return abs(got - want)
    return abs(got - want) / abs(want)


class Checker:
    def __init__(self, client):
        self.c = client
        self.worst = {}
        self.n = 0

    def call(self, op, a):
        return json.loads(mcp_text(self.c.tool_call("yk.compute", {"op": op, "a": a}).response))

    def cmp(self, op, a, want, tol=1e-11):
        r = self.call(op, a)
        self.n += 1
        if "r" not in r:
            raise AssertionError(f"{op}{a} -> {r} (expected {want})")
        e = rel(r["r"], want)
        if e > self.worst.get(op, (0.0,))[0]:
            self.worst[op] = (e, a, r["r"], want)
        if e > tol:
            raise AssertionError(f"{op}{a}: got {r['r']!r} want {want!r} rel {e:.3e}")
        return r["r"]


def main_dists():
    with tempfile.TemporaryDirectory() as home:
        with BenchClient(EXE,
                         env={"YEKATERINA_HOME": home}, timeout=180) as c:
            k = Checker(c)

            for df in DF:
                d = stats.t(df)
                for x in XS:
                    k.cmp("prob.t_pdf", [x, df], float(d.pdf(x)))
                    k.cmp("prob.t_cdf", [x, df], float(d.cdf(x)))
                    k.cmp("prob.t_sf", [x, df], float(d.sf(x)))
                for p in PS:
                    k.cmp("prob.t_ppf", [p, df], float(d.ppf(p)), tol=1e-9)

                ch = stats.chi2(df)
                for x in POS:
                    k.cmp("prob.chi2_pdf", [x, df], float(ch.pdf(x)))
                    k.cmp("prob.chi2_cdf", [x, df], float(ch.cdf(x)))
                    k.cmp("prob.chi2_sf", [x, df], float(ch.sf(x)))
                for p in PS:
                    k.cmp("prob.chi2_ppf", [p, df], float(ch.ppf(p)), tol=1e-9)

            for d1 in [1.0, 3.0, 12.0]:
                for d2 in [2.0, 12.0, 100.0]:
                    f = stats.f(d1, d2)
                    for x in POS:
                        k.cmp("prob.f_pdf", [x, d1, d2], float(f.pdf(x)))
                        k.cmp("prob.f_cdf", [x, d1, d2], float(f.cdf(x)))
                        k.cmp("prob.f_sf", [x, d1, d2], float(f.sf(x)))
                    for p in [0.001, 0.5, 0.95, 0.999]:
                        k.cmp("prob.f_ppf", [p, d1, d2], float(f.ppf(p)), tol=1e-9)

            n = stats.norm(0, 1)
            for x in XS:
                k.cmp("prob.normal_sf", [x, 0.0, 1.0], float(n.sf(x)))
            for p in PS:
                k.cmp("prob.normal_ppf", [p, 0.0, 1.0], float(n.ppf(p)), tol=1e-10)
            k.cmp("prob.normal_ppf", [0.975, 10.0, 2.0], float(stats.norm(10, 2).ppf(0.975)), tol=1e-10)

            for shape, scale in [(0.5, 1.0), (3.0, 1.5), (20.0, 0.25)]:
                g = stats.gamma(shape, scale=scale)
                for x in POS:
                    k.cmp("prob.gamma_pdf", [x, shape, scale], float(g.pdf(x)))
                    k.cmp("prob.gamma_cdf", [x, shape, scale], float(g.cdf(x)))
                w = stats.weibull_min(shape, scale=scale)
                for x in POS:
                    k.cmp("prob.weibull_pdf", [x, shape, scale], float(w.pdf(x)))
                    k.cmp("prob.weibull_cdf", [x, shape, scale], float(w.cdf(x)))

            for a, b in [(0.5, 0.5), (2.0, 3.0), (30.0, 40.0)]:
                be = stats.beta(a, b)
                for x in [1e-6, 0.1, 0.4, 0.9, 0.999]:
                    k.cmp("prob.beta_pdf", [x, a, b], float(be.pdf(x)))
                    k.cmp("prob.beta_cdf", [x, a, b], float(be.cdf(x)))

            for mu, sigma in [(0.0, 1.0), (1.5, 0.4)]:
                ln = stats.lognorm(sigma, scale=pow(2.718281828459045, mu))
                for x in POS:
                    k.cmp("prob.lognormal_pdf", [x, mu, sigma], float(ln.pdf(x)))
                    k.cmp("prob.lognormal_cdf", [x, mu, sigma], float(ln.cdf(x)))

            # The survival function must beat 1 - cdf in the far tail. The
            # normal at z=10 is where the difference is total rather than
            # merely large: the CDF rounds to exactly 1.0 in f64, so 1 - cdf is
            # exactly zero while the true tail is about 7.6e-24.
            deep = k.call("prob.normal_sf", [10.0, 0.0, 1.0])["r"]
            naive = 1.0 - k.call("prob.normal_cdf", [10.0, 0.0, 1.0])["r"]
            want = float(stats.norm(0, 1).sf(10.0))
            assert rel(deep, want) < 1e-12, (deep, want)
            assert naive == 0.0, f"expected 1-cdf to underflow to zero, got {naive}"
            print(f"  tail check: normal_sf(10) = {deep:.6e} (scipy {want:.6e}); "
                  f"1 - cdf gives {naive}")
            # A milder case, reported rather than asserted, to show the size of
            # the loss where it is not yet total.
            t_deep = k.call("prob.t_sf", [40.0, 5.0])["r"]
            t_naive = 1.0 - k.call("prob.t_cdf", [40.0, 5.0])["r"]
            t_want = float(stats.t(5).sf(40.0))
            print(f"  tail check: t_sf(40,5) rel {rel(t_deep, t_want):.2e} vs "
                  f"1-cdf rel {rel(t_naive, t_want):.2e}")

            # Quantiles must invert their own CDF, independently of scipy.
            for p in [0.01, 0.3, 0.9, 0.999]:
                for df in [1.0, 7.0, 100.0]:
                    x = k.call("prob.t_ppf", [p, df])["r"]
                    assert rel(k.call("prob.t_cdf", [x, df])["r"], p) < 1e-9, (p, df)
                    x = k.call("prob.chi2_ppf", [p, df])["r"]
                    assert rel(k.call("prob.chi2_cdf", [x, df])["r"], p) < 1e-9, (p, df)

    print(f"checked {k.n} distribution values against scipy")
    for op, (e, a, got, want) in sorted(k.worst.items(), key=lambda kv: -kv[1][0])[:5]:
        print(f"  worst {op:20} rel {e:.3e}  at {a}")
    print("PASS: survival functions keep tail accuracy, quantiles invert their CDFs")




# ==================== from check_inference.py ====================



A = [5.1, 4.9, 6.2, 5.8, 5.0, 6.6, 5.3, 4.4]
B = [4.1, 4.4, 3.9, 4.6, 5.2, 4.0, 4.8, 5.9]
P1 = [5.1, 4.9, 6.2, 5.8, 6.0, 5.5, 4.7, 6.1]
P2 = [4.8, 4.7, 5.9, 5.5, 5.4, 5.6, 4.9, 5.7]
G = [[1.0, 2.0, 3.0, 9.0], [4.0, 5.0, 6.0, 7.0], [7.0, 8.0, 10.0, 11.0]]
NORMAL = [0.1, -0.5, 1.2, 0.3, -1.1, 0.7, 2.0, -0.2, 0.9, -1.4]

fails, checked = [], 0


def rel(got, want):
    if abs(want) < 1e-300:
        return abs(got - want)
    return abs(got - want) / abs(want)


def check(label, got, want, tol=1e-10):
    global checked
    checked += 1
    if got is None or not isinstance(got, (int, float)):
        fails.append(f"{label}: got {got!r}")
        return
    e = rel(float(got), float(want))
    if e > tol:
        fails.append(f"{label}: got {got!r} want {want!r} rel {e:.3e}")


def main_inference():
    with tempfile.TemporaryDirectory() as home:
        with BenchClient(EXE,
                         env={"YEKATERINA_HOME": home}, timeout=180) as c:
            def call(op, a):
                r = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                if "r" not in r:
                    fails.append(f"{op}{a} -> {r}")
                    return None
                return r["r"]

            # --- generic p-values -------------------------------------------
            for t, df in [(2.5, 10.0), (-1.2, 3.0), (0.0, 50.0), (9.0, 2.0)]:
                check(f"p_t2({t},{df})", call("test.p_t", [t, df, 2]), 2 * stats.t(df).sf(abs(t)))
                check(f"p_t1({t},{df})", call("test.p_t", [t, df, 1]), stats.t(df).sf(t))
            for z in [1.96, -0.5, 0.0, 6.0]:
                check(f"p_z2({z})", call("test.p_z", [z, 2]), 2 * stats.norm.sf(abs(z)))
                check(f"p_z1({z})", call("test.p_z", [z, 1]), stats.norm.sf(z))
            for x, df in [(9.488, 4.0), (0.5, 1.0), (100.0, 10.0)]:
                check(f"p_chi2({x},{df})", call("test.p_chi2", [x, df]), stats.chi2(df).sf(x))
            for f, d1, d2 in [(3.49, 3.0, 12.0), (0.2, 5.0, 5.0), (50.0, 2.0, 30.0)]:
                check(f"p_f({f})", call("test.p_f", [f, d1, d2]), stats.f(d1, d2).sf(f))

            # --- t tests -----------------------------------------------------
            r = call("test.t_one_sample_test", [A, 5.0])
            s = stats.ttest_1samp(A, 5.0)
            check("t1.t", r["t"], s.statistic); check("t1.p", r["p"], s.pvalue)
            check("t1.df", r["df"], len(A) - 1)

            r = call("test.t_two_equal_test", [A, B])
            s = stats.ttest_ind(A, B, equal_var=True)
            check("t2.t", r["t"], s.statistic); check("t2.p", r["p"], s.pvalue)

            r = call("test.t_welch_test", [A, B])
            s = stats.ttest_ind(A, B, equal_var=False)
            check("welch.t", r["t"], s.statistic); check("welch.p", r["p"], s.pvalue)
            check("welch.df", r["df"], s.df)

            r = call("test.t_paired_test", [P1, P2])
            s = stats.ttest_rel(P1, P2)
            check("paired.t", r["t"], s.statistic); check("paired.p", r["p"], s.pvalue)
            check("paired.stat", call("test.t_paired", [P1, P2]), s.statistic)

            # --- ANOVA and variance homogeneity ------------------------------
            r = call("test.anova_one_way", [G])
            s = stats.f_oneway(*G)
            check("anova.f", r["f"], s.statistic); check("anova.p", r["p"], s.pvalue)

            r = call("test.levene", [G])
            s = stats.levene(*G, center="median")
            check("levene.f", r["f"], s.statistic); check("levene.p", r["p"], s.pvalue)

            r = call("test.bartlett", [G])
            s = stats.bartlett(*G)
            check("bartlett.chi2", r["chi2"], s.statistic); check("bartlett.p", r["p"], s.pvalue)

            # --- rank tests --------------------------------------------------
            r = call("test.mann_whitney_u", [A, B])
            s = stats.mannwhitneyu(A, B, alternative="two-sided", method="asymptotic")
            # scipy reports U for the first sample; the engine reports both.
            check("mwu.u1", r["u1"], s.statistic)
            check("mwu.p", r["p"], s.pvalue, tol=1e-9)

            r = call("test.wilcoxon_signed_rank", [P1, P2])
            s = stats.wilcoxon(P1, P2, method="approx", correction=True)
            check("wilcoxon.w", r["w"], s.statistic)
            check("wilcoxon.p", r["p"], s.pvalue, tol=1e-9)

            # --- KS ------------------------------------------------------------
            r = call("test.ks_normal", [NORMAL, 0.0, 1.0])
            s = stats.kstest(NORMAL, "norm", args=(0.0, 1.0))
            check("ks.d", r["d"], s.statistic)
            # The p is the asymptotic Kolmogorov form, which is what
            # kstwobign gives; scipy's default exact value differs for n=10 and
            # that difference is the point of labelling ours asymptotic.
            check("ks.p_asymptotic", r["p"],
                  stats.kstwobign.sf(math.sqrt(len(NORMAL)) * s.statistic), tol=1e-9)

            # --- categorical ---------------------------------------------------
            table = [12.0, 5.0, 8.0, 15.0]
            r = call("test.chi_square_yates", table)
            s = stats.chi2_contingency([[12, 5], [8, 15]], correction=True)
            check("yates.chi2", r["chi2"], s.statistic); check("yates.p", r["p"], s.pvalue)

            check("fisher", call("test.fisher_exact", table),
                  stats.fisher_exact([[12, 5], [8, 15]]).pvalue, tol=1e-10)
            for k, n, p0 in [(7, 20, 0.5), (0, 10, 0.3), (10, 10, 0.9), (3, 17, 0.25)]:
                check(f"binom({k},{n},{p0})", call("test.binomial_test", [k, n, p0]),
                      stats.binomtest(k, n, p0).pvalue, tol=1e-10)

            r = call("test.mcnemar", [12.0, 5.0])
            chi2 = (abs(12 - 5) - 1) ** 2 / (12 + 5)
            check("mcnemar.chi2", r["chi2"], chi2)
            check("mcnemar.p", r["p"], stats.chi2(1).sf(chi2))

            check("corr_p", call("test.correlation_p", [0.8, 12]),
                  2 * stats.t(10).sf(0.8 * math.sqrt(10 / (1 - 0.64))))

            # --- intervals -------------------------------------------------------
            lo, hi = call("test.ci_mean", [A, 0.95])
            m, se = float(sum(A)) / len(A), stats.sem(A)
            want = stats.t.interval(0.95, len(A) - 1, loc=m, scale=se)
            check("ci_mean.lo", lo, want[0]); check("ci_mean.hi", hi, want[1])

            lo, hi = call("test.ci_proportion", [7, 20, 0.95])
            want = stats.binomtest(7, 20).proportion_ci(confidence_level=0.95, method="wilson")
            check("ci_prop.lo", lo, want.low, tol=1e-9)
            check("ci_prop.hi", hi, want.high, tol=1e-9)

            lo, hi = call("test.ci_variance", [A, 0.95])
            n, s2 = len(A), stats.tvar(A)
            check("ci_var.lo", lo, (n - 1) * s2 / stats.chi2(n - 1).ppf(0.975), tol=1e-9)
            check("ci_var.hi", hi, (n - 1) * s2 / stats.chi2(n - 1).ppf(0.025), tol=1e-9)

            lo, hi = call("test.ci_mean_diff", [A, B, 0.95])
            s = stats.ttest_ind(A, B, equal_var=False).confidence_interval(0.95)
            check("ci_diff.lo", lo, s.low, tol=1e-9)
            check("ci_diff.hi", hi, s.high, tol=1e-9)

            # --- sizing (formula, verified against an independent restatement) --
            za, zb = stats.norm.ppf(0.975), stats.norm.ppf(0.8)
            check("n_mean", call("test.sample_size_mean", [0.5, 1.0, 0.05, 0.8]),
                  math.ceil(((za + zb) * 1.0 / 0.5) ** 2))
            p1, p2 = 0.5, 0.65
            pbar = (p1 + p2) / 2
            num = za * math.sqrt(2 * pbar * (1 - pbar)) + zb * math.sqrt(p1 * (1 - p1) + p2 * (1 - p2))
            check("n_prop", call("test.sample_size_proportion", [p1, p2, 0.05, 0.8]),
                  math.ceil((num / (p1 - p2)) ** 2))

            power = call("test.power_t", [30, 0.5, 0.05])
            crit = stats.t(29).ppf(0.975)
            ncp = 0.5 * math.sqrt(30)
            check("power_t", power, stats.norm.sf(crit - ncp) + stats.norm.sf(crit + ncp), tol=1e-9)

    print(f"checked {checked} inference values against scipy")
    if fails:
        print(f"\n{len(fails)} FAILURES:")
        for f in fails:
            print("  " + f)
        raise SystemExit(1)
    print("PASS: every inference operation agrees with scipy")




# ==================== regression ====================

RX = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]
RY = [2.1, 3.9, 6.2, 7.8, 10.1, 12.2, 13.8]
RMX = [[1.0, 2.0], [2.0, 1.0], [3.0, 4.0], [4.0, 3.0], [5.0, 6.0], [6.0, 5.0], [7.0, 8.0]]
RMY = [3.1, 3.9, 7.2, 7.1, 11.3, 11.0, 15.2]


def main_regression():
    fails, n = [], 0

    def chk(label, got, want, tol=1e-9):
        nonlocal n
        n += 1
        if not isinstance(got, (int, float)) or abs(got - want) > tol * max(1.0, abs(want)):
            fails.append(f"{label}: got {got!r} want {want!r}")

    with tempfile.TemporaryDirectory() as home:
        with BenchClient(EXE, env={"YEKATERINA_HOME": home}, timeout=180) as c:
            def call(op, a):
                r = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                if "r" not in r:
                    fails.append(f"{op} -> {r}")
                    return None
                return r["r"]

            lr = stats.linregress(RX, RY)
            chk("slope_se", call("reg.slope_se", [RX, RY]), lr.stderr)
            chk("intercept_se", call("reg.intercept_se", [RX, RY]), lr.intercept_stderr)
            chk("slope_t", call("reg.slope_t", [RX, RY]), lr.slope / lr.stderr)
            chk("slope_p", call("reg.slope_p", [RX, RY]), lr.pvalue)
            lo, hi = call("reg.slope_ci", [RX, RY, 0.95])
            half = stats.t.ppf(0.975, len(RX) - 2) * lr.stderr
            chk("slope_ci.lo", lo, lr.slope - half)
            chk("slope_ci.hi", hi, lr.slope + half)
            f = call("reg.f_test", [RX, RY])
            chk("f_test.f", f["f"], (lr.slope / lr.stderr) ** 2)
            chk("f_test.p", f["p"], lr.pvalue)

            design = np.column_stack([np.ones(len(RMX)), np.array(RMX)])
            beta, *_ = np.linalg.lstsq(design, np.array(RMY), rcond=None)
            m = call("reg.multiple", [RMX, RMY])
            chk("multiple.intercept", m["intercept"], beta[0], tol=1e-7)
            for i, b in enumerate(beta[1:]):
                chk(f"multiple.coef{i}", m["coefficients"][i], b, tol=1e-7)
            yhat = design @ beta
            sst = float(((np.array(RMY) - np.mean(RMY)) ** 2).sum())
            sse = float(((np.array(RMY) - yhat) ** 2).sum())
            chk("multiple.r2", m["r2"], 1 - sse / sst)

            got = call("reg.polynomial", [RX, RY, 2])
            for i, v in enumerate(np.polyfit(RX, RY, 2)):
                chk(f"polynomial[{i}]", got[i], float(v), tol=1e-6)

            chk("kendall_tau", call("reg.kendall_tau", [RX, RY]),
                float(stats.kendalltau(RX, RY).statistic))

            e = [0.1, -0.2, 0.3, -0.1, 0.2, -0.3, 0.1]
            chk("durbin_watson", call("reg.durbin_watson", [e]),
                float(np.sum(np.diff(e) ** 2) / np.sum(np.array(e) ** 2)))

            ts = stats.theilslopes(RY, RX)
            t = call("reg.theil_sen", [RX, RY])
            chk("theil_sen.slope", t["slope"], float(ts.slope))
            chk("theil_sen.intercept", t["intercept"], float(ts.intercept))

            ex = call("reg.exponential", [RX, RY])
            le = stats.linregress(RX, np.log(RY))
            chk("exponential.b", ex["b"], le.slope)
            chk("exponential.a", ex["a"], math.exp(le.intercept))
            pw = call("reg.power", [RX, RY])
            lp = stats.linregress(np.log(RX), np.log(RY))
            chk("power.b", pw["b"], lp.slope)
            chk("power.a", pw["a"], math.exp(lp.intercept))
            lg = call("reg.logarithmic", [RX, RY])
            ll = stats.linregress(np.log(RX), RY)
            chk("logarithmic.b", lg["b"], ll.slope)
            chk("logarithmic.a", lg["a"], ll.intercept)

            fit = list(lr.intercept + lr.slope * np.array(RX))
            nn = len(RY)
            resid = float(((np.array(RY) - np.array(fit)) ** 2).sum())
            loglik = -0.5 * nn * (math.log(2 * math.pi * resid / nn) + 1)
            chk("aic", call("reg.aic", [RY, fit, 1]), 2 * 2 - 2 * loglik)
            chk("bic", call("reg.bic", [RY, fit, 1]), 2 * math.log(nn) - 2 * loglik)
            chk("adjusted_r2", call("reg.adjusted_r2", [RY, fit, 1]),
                1 - (1 - lr.rvalue ** 2) * (nn - 1) / (nn - 2))

    print(f"checked {n} regression values against scipy and numpy")
    if fails:
        print(f"{len(fails)} FAILURES:")
        for f in fails:
            print("  " + f)
        raise SystemExit(1)
    print("PASS: every regression operation agrees with scipy and numpy")


if __name__ == "__main__":
    main_special()
    print()
    main_dists()
    print()
    main_inference()
    print()
    main_regression()
    print()
    print("PASS: statistical operations verified against scipy, numpy and mpmath")
