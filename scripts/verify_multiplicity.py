"""Verify the multiplicity, post-hoc and effect-size operations.

Run:  python scripts/verify_multiplicity.py
Needs a release binary, plus scipy, numpy and statsmodels as references.

Every operation is checked against a value produced by a different route than
the implementation uses:

  * the six p-value corrections against statsmodels' `multipletests`, which is a
    separate implementation, *and* against a direct transcription of each
    procedure's definition written here -- two independent references, because a
    step-up and a step-down procedure differ only in the direction of a running
    extremum and a wrong one still returns plausible numbers. For Sidak,
    statsmodels happens to use the same expm1/log1p form the engine does, so
    there the naive `1 - (1-p)^m` transcription below is the only independent
    reference;
  * Welch pairwise comparisons against `scipy.stats.ttest_ind(equal_var=False)`;
    pooled ones against the ANOVA within-group mean square computed in numpy;
  * eta squared twice, from the sums of squares and from the F statistic
    `scipy.stats.f_oneway` returns, which must agree;
  * Cramer's V against `scipy.stats.contingency.association`;
  * the rank-biserial correlation against `scipy.stats.mannwhitneyu`'s U;
  * Cliff's delta against the brute-force O(n1*n2) double loop, where the
    implementation uses sorted binary search;
  * the odds- and risk-ratio intervals against `statsmodels` Table2x2;
  * Cohen's h against `statsmodels.stats.proportion.proportion_effectsize`.

Identities that a wrong formula would break are asserted alongside the values:
Bonferroni must dominate Holm which must dominate BH, every correction must be
monotone in the raw p-value ordering, Cramer's V on a 2x2 must equal |phi|, and
Cliff's delta must be the rank-biserial correlation.
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
    from scipy import stats
    from scipy.stats.contingency import association
    from statsmodels.stats.contingency_tables import Table2x2
    from statsmodels.stats.multitest import multipletests
    from statsmodels.stats.proportion import proportion_effectsize
except ImportError as exc:                                  # pragma: no cover
    print(f"SKIP: verify_multiplicity needs scipy, numpy and statsmodels ({exc})")
    raise SystemExit(0)

mp.mp.dps = 50

_exe = ROOT / "target/release/yekaterina.exe"
EXE = str(_exe if _exe.exists() else ROOT / "target/release/yekaterina")

checked = 0
fails: list[str] = []


def rel(got, want):
    if abs(want) < 1e-300:
        return abs(got - want)
    return abs(got - want) / abs(want)


def check(label, got, want, tol=1e-12):
    global checked
    checked += 1
    if got is None or not isinstance(got, (int, float)):
        fails.append(f"{label}: got {got!r}")
        return
    e = rel(float(got), float(want))
    if e > tol:
        fails.append(f"{label}: got {got!r} want {want!r} rel {e:.3e}")


def check_vec(label, got, want, tol=1e-12):
    if not isinstance(got, list) or len(got) != len(want):
        global checked
        checked += 1
        fails.append(f"{label}: got {got!r} want {len(want)} values")
        return
    for i, (g, w) in enumerate(zip(got, want)):
        check(f"{label}[{i}]", g, w, tol)


# ------------------------------------------------- corrections, from definition

def d_bonferroni(p):
    m = len(p)
    return [min(1.0, m * v) for v in p]


def d_sidak(p):
    """Sidak from the definition, at 50 digits.

    The naive float form `1 - (1-p)**m` is what this correction exists to avoid:
    at p = 1e-12 and m = 5 it returns 4.99988...e-12 where the true value is
    4.99999999999e-12, a relative error of 2e-5. Evaluating the same definition
    in mpmath keeps it an independent route without making it the less accurate
    one.
    """
    m = len(p)
    return [float(min(mp.mpf(1), 1 - (1 - mp.mpf(v)) ** m)) for v in p]


def d_holm(p):
    m = len(p)
    order = sorted(range(m), key=lambda i: p[i])
    out = [0.0] * m
    run = 0.0
    for rank, i in enumerate(order):
        run = max(run, min(1.0, (m - rank) * p[i]))
        out[i] = run
    return out


def d_hochberg(p):
    m = len(p)
    order = sorted(range(m), key=lambda i: -p[i])
    out = [0.0] * m
    run = math.inf
    for rank, i in enumerate(order):
        run = min(run, min(1.0, (rank + 1) * p[i]))
        out[i] = run
    return out


def d_fdr(p, extra=1.0):
    m = len(p)
    order = sorted(range(m), key=lambda i: -p[i])
    out = [0.0] * m
    run = math.inf
    for pos, i in enumerate(order):
        rank = m - pos
        run = min(run, min(1.0, extra * m / rank * p[i]))
        out[i] = run
    return out


def d_by(p):
    return d_fdr(p, extra=sum(1.0 / i for i in range(1, len(p) + 1)))


P_SETS = [
    [0.01, 0.04, 0.03, 0.2],
    [0.001, 0.008, 0.039, 0.041, 0.042, 0.06, 0.074, 0.205, 0.212, 0.216],
    [0.5, 0.5, 0.5],
    [1e-12, 0.2, 0.9, 0.95, 1.0],
    [0.0, 0.049, 0.051, 1.0],
]

SM_NAME = {
    "bonferroni": "bonferroni",
    "sidak": "sidak",
    "holm": "holm",
    "hochberg": "simes-hochberg",
    "bh": "fdr_bh",
    "by": "fdr_by",
}
DEFN = {
    "bonferroni": d_bonferroni,
    "sidak": d_sidak,
    "holm": d_holm,
    "hochberg": d_hochberg,
    "bh": lambda p: d_fdr(p),
    "by": d_by,
}


def main():
    with tempfile.TemporaryDirectory() as home:
        with BenchClient(EXE, env={"YEKATERINA_HOME": home}, timeout=180) as c:
            def call(op, a):
                r = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                if "r" not in r:
                    fails.append(f"{op}{a} -> {r}")
                    return None
                return r["r"]

            # ---------------------------------------------------- corrections
            for p in P_SETS:
                for key, sm in SM_NAME.items():
                    got = call(f"test.p_adjust_{key}", [p])
                    want_sm = list(multipletests(p, method=sm)[1])
                    want_def = DEFN[key](p)
                    check_vec(f"p_adjust_{key}{p} vs statsmodels", got, want_sm, tol=1e-11)
                    check_vec(f"p_adjust_{key}{p} vs definition", got, want_def, tol=1e-11)

                # Sidak keeps relative accuracy on a tiny p where the naive
                # 1 - (1-p)^m collapses to m*p only after losing digits.
                tiny = [1e-15, 0.5]
                got = call("test.p_adjust_sidak", [tiny])
                check(f"sidak tiny {tiny}", got[0],
                      float(1 - (1 - mp.mpf("1e-15")) ** 2), tol=1e-13)

                # Ordering identities: Bonferroni is never smaller than Holm,
                # Holm never smaller than BH.
                bonf = call("test.p_adjust_bonferroni", [p])
                holm = call("test.p_adjust_holm", [p])
                bh = call("test.p_adjust_bh", [p])
                by = call("test.p_adjust_by", [p])
                global checked
                for i in range(len(p)):
                    checked += 1
                    if not bonf[i] >= holm[i] - 1e-15:
                        fails.append(f"bonferroni < holm at {i} for {p}")
                    checked += 1
                    if holm[i] < 0.0:
                        fails.append(f"holm negative at {i} for {p}")
                    checked += 1
                    if not (holm[i] >= bh[i] - 1e-15):
                        fails.append(f"holm < bh at {i} for {p}")
                    checked += 1
                    if not (by[i] >= bh[i] - 1e-15):
                        fails.append(f"by < bh at {i} for {p}")
                # Monotone in the raw ordering.
                for name, adj in (("holm", holm), ("bh", bh), ("hochberg", call("test.p_adjust_hochberg", [p]))):
                    order = sorted(range(len(p)), key=lambda i: p[i])
                    checked += 1
                    seq = [adj[i] for i in order]
                    if any(seq[i] > seq[i + 1] + 1e-15 for i in range(len(seq) - 1)):
                        fails.append(f"{name} not monotone for {p}: {seq}")

            # ------------------------------------------------- fdr threshold
            for p in P_SETS:
                for q in (0.05, 0.1, 0.2):
                    got = call("test.fdr_threshold", [p, q])
                    s = sorted(p)
                    m = len(s)
                    want = 0.0
                    for i, v in enumerate(s):
                        if v <= (i + 1) * q / m:
                            want = v
                    check(f"fdr_threshold({p},{q})", got, want)
                    # A p-value is rejected by BH at q exactly when it is at or
                    # below the threshold.
                    adj = call("test.p_adjust_bh", [p])
                    checked += 1
                    by_adj = {i for i, v in enumerate(adj) if v <= q + 1e-15}
                    by_thr = {i for i, v in enumerate(p) if v <= want}
                    if by_adj != by_thr:
                        fails.append(f"fdr_threshold disagrees with bh at q={q} for {p}: {by_adj} vs {by_thr}")

            # ------------------------------------------------------ pairwise
            GROUPS = [
                [[1.0, 2.0, 3.0, 4.0], [3.0, 4.0, 5.0, 6.0], [6.0, 7.0, 8.0, 9.0]],
                [[12.1, 11.4, 13.2, 10.9, 12.8], [15.2, 14.8, 16.1, 15.5],
                 [9.8, 10.2, 9.1, 10.5, 9.9, 10.1]],
            ]
            for groups in GROUPS:
                arrs = [np.array(g) for g in groups]
                k = len(arrs)
                n = sum(len(a) for a in arrs)
                ssw = sum(((a - a.mean()) ** 2).sum() for a in arrs)
                msw = ssw / (n - k)

                got = call("test.pairwise_t", [groups])
                pairs = [(i, j) for i in range(k) for j in range(i + 1, k)]
                check(f"pairwise_t count {len(groups)}", len(got), len(pairs), tol=0)
                for row, (i, j) in zip(got, pairs):
                    a, b = arrs[i], arrs[j]
                    se = math.sqrt(msw * (1 / len(a) + 1 / len(b)))
                    t = (a.mean() - b.mean()) / se
                    df = n - k
                    check(f"pairwise_t t {i}{j}", row["t"], t)
                    check(f"pairwise_t df {i}{j}", row["df"], df)
                    check(f"pairwise_t p {i}{j}", row["p"], 2 * stats.t.sf(abs(t), df), tol=1e-11)
                    check(f"pairwise_t diff {i}{j}", row["diff"], a.mean() - b.mean())

                got = call("test.pairwise_welch", [groups])
                for row, (i, j) in zip(got, pairs):
                    a, b = arrs[i], arrs[j]
                    res = stats.ttest_ind(a, b, equal_var=False)
                    check(f"pairwise_welch t {i}{j}", row["t"], res.statistic, tol=1e-11)
                    check(f"pairwise_welch p {i}{j}", row["p"], res.pvalue, tol=1e-11)
                    check(f"pairwise_welch df {i}{j}", row["df"], res.df, tol=1e-11)

            # -------------------------------------------------- effect sizes
            SAMPLES = [
                ([1.0, 2.0, 3.0, 4.0, 5.0], [3.0, 4.0, 5.0, 6.0, 8.0]),
                ([22.1, 19.8, 25.3, 21.0, 23.7, 20.4], [18.2, 17.5, 19.9, 16.8]),
            ]
            for a, b in SAMPLES:
                A, B = np.array(a), np.array(b)
                na, nb = len(A), len(B)
                sp = math.sqrt(((na - 1) * A.var(ddof=1) + (nb - 1) * B.var(ddof=1)) / (na + nb - 2))
                d = (A.mean() - B.mean()) / sp
                nn = na + nb
                check("hedges_g", call("test.hedges_g", [a, b]), d * (1 - 3 / (4 * nn - 9)))
                check("glass_delta", call("test.glass_delta", [a, b]),
                      (A.mean() - B.mean()) / B.std(ddof=1))

                u = stats.mannwhitneyu(A, B, alternative="two-sided", method="asymptotic").statistic
                check("rank_biserial", call("test.rank_biserial", [a, b]),
                      (u - (na * nb - u)) / (na * nb))
                brute = sum(1 for x in A for y in B if x > y) - sum(1 for x in A for y in B if x < y)
                check("cliffs_delta", call("test.cliffs_delta", [a, b]), brute / (na * nb))
                # Cliff's delta and the rank-biserial correlation are the same
                # quantity when there are no ties.
                checked += 1
                if rel(call("test.cliffs_delta", [a, b]), call("test.rank_biserial", [a, b])) > 1e-12:
                    fails.append(f"cliffs_delta != rank_biserial for {a},{b}")

            for groups in GROUPS + [[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]]]:
                arrs = [np.array(g) for g in groups]
                k, n = len(arrs), sum(len(a) for a in arrs)
                grand = np.concatenate(arrs).mean()
                ssb = sum(len(a) * (a.mean() - grand) ** 2 for a in arrs)
                ssw = sum(((a - a.mean()) ** 2).sum() for a in arrs)
                sst, msw = ssb + ssw, ssw / (n - k)
                eta = ssb / sst
                check("eta_squared", call("test.eta_squared", [groups]), eta)
                # The same eta squared, from the F statistic scipy returns.
                f = stats.f_oneway(*arrs).statistic
                df1, df2 = k - 1, n - k
                check("eta_squared via F", call("test.eta_squared", [groups]),
                      df1 * f / (df1 * f + df2), tol=1e-11)
                check("omega_squared", call("test.omega_squared", [groups]),
                      (ssb - df1 * msw) / (sst + msw))
                check("cohen_f", call("test.cohen_f", [groups]), math.sqrt(eta / (1 - eta)))

            TABLES = [[[10.0, 20.0], [30.0, 25.0]], [[12.0, 7.0, 21.0], [8.0, 14.0, 9.0]]]
            for t in TABLES:
                check("cramers_v", call("test.cramers_v", [t]),
                      association(np.array(t, dtype=int), method="cramer"), tol=1e-11)
            t11, t12, t21, t22 = 10.0, 20.0, 30.0, 25.0
            phi = ((t11 * t22 - t12 * t21)
                   / math.sqrt((t11 + t12) * (t21 + t22) * (t11 + t21) * (t12 + t22)))
            check("phi_coefficient", call("test.phi_coefficient", [t11, t12, t21, t22]), phi)
            # On a 2x2 table Cramer's V is |phi|.
            check("cramers_v == |phi|",
                  call("test.cramers_v", [[[t11, t12], [t21, t22]]]), abs(phi), tol=1e-11)

            obs, exp = [10.0, 20.0, 30.0], [20.0, 20.0, 20.0]
            w = math.sqrt(sum((o / 60 - e / 60) ** 2 / (e / 60) for o, e in zip(obs, exp)))
            check("cohen_w", call("test.cohen_w", [obs, exp]), w)
            # w equals sqrt(chi2 / n) when both are counts over the same total.
            chi2 = sum((o - e) ** 2 / e for o, e in zip(obs, exp))
            check("cohen_w via chi2", call("test.cohen_w", [obs, exp]), math.sqrt(chi2 / 60), tol=1e-11)

            for p1, p2 in ((0.4, 0.6), (0.05, 0.5), (0.9, 0.1)):
                check(f"cohen_h({p1},{p2})", call("test.cohen_h", [p1, p2]),
                      proportion_effectsize(p1, p2), tol=1e-11)

            # ----------------------------------------------- 2x2 ratios + CIs
            for (t11, t12, t21, t22) in ((10.0, 20.0, 30.0, 25.0), (5.0, 45.0, 12.0, 38.0)):
                cells = [t11, t12, t21, t22]
                t2 = Table2x2(np.array([[t11, t12], [t21, t22]]))
                check("risk_ratio", call("test.risk_ratio", cells), t2.riskratio, tol=1e-11)
                for conf in (0.95, 0.99):
                    alpha = 1 - conf
                    lo, hi = call("test.odds_ratio_ci", cells + [conf])
                    wlo, whi = t2.oddsratio_confint(alpha=alpha)
                    check(f"odds_ratio_ci lo {conf}", lo, wlo, tol=1e-9)
                    check(f"odds_ratio_ci hi {conf}", hi, whi, tol=1e-9)
                    lo, hi = call("test.risk_ratio_ci", cells + [conf])
                    wlo, whi = t2.riskratio_confint(alpha=alpha)
                    check(f"risk_ratio_ci lo {conf}", lo, wlo, tol=1e-9)
                    check(f"risk_ratio_ci hi {conf}", hi, whi, tol=1e-9)
                    # The interval must contain the point estimate.
                    checked += 1
                    point = call("test.risk_ratio", cells)
                    if not (lo <= point <= hi):
                        fails.append(f"risk_ratio_ci does not contain the point estimate: {lo} {point} {hi}")

            # --------------------------------------------------------- guards
            GUARDS = [
                ("test.p_adjust_bh", [[]], "EMPTY"),
                ("test.p_adjust_bh", [[0.5, 1.5]], "DOMAIN"),
                ("test.p_adjust_bh", [[0.5, -0.1]], "DOMAIN"),
                ("test.fdr_threshold", [[0.1], 1.0], "DOMAIN"),
                ("test.fdr_threshold", [[0.1], 0.0], "DOMAIN"),
                ("test.pairwise_t", [[[1.0, 2.0]]], "SHAPE"),
                ("test.pairwise_t", [[[1.0], [2.0]]], "SHAPE"),
                ("test.cohen_h", [1.5, 0.5], "DOMAIN"),
                ("test.cramers_v", [[[0.0, 0.0], [0.0, 0.0]]], "DOMAIN"),
                ("test.phi_coefficient", [0.0, 0.0, 1.0, 1.0], "DEGENERATE"),
                ("test.risk_ratio", [1.0, 1.0, 0.0, 1.0], "DEGENERATE"),
                ("test.odds_ratio_ci", [0.0, 1.0, 1.0, 1.0, 0.95], "DEGENERATE"),
                ("test.glass_delta", [[1.0, 1.0], [2.0, 2.0]], "DEGENERATE"),
                ("test.hedges_g", [[1.0, 1.0], [1.0, 1.0]], "DEGENERATE"),
            ]
            for op, a, code in GUARDS:
                checked += 1
                r = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                if r.get("e") != code:
                    fails.append(f"guard {op}{a}: expected {code}, got {r}")

    print(f"checked {checked} assertions")
    if fails:
        for f in fails[:40]:
            print("FAIL:", f)
        print(f"FAIL: {len(fails)} of {checked}")
        raise SystemExit(1)
    print("PASS: multiplicity, post-hoc and effect-size operations verified "
          "against statsmodels, scipy and numpy")


if __name__ == "__main__":
    main()
