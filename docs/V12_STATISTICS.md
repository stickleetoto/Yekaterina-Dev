# Yekaterina v1.2 — statistical inference

**1,318 → 1,387. Sixty-nine operations, and the reason they were the right
sixty-nine.**

Before this batch the engine could compute a t statistic, a chi-square statistic,
an F ratio and Cohen's d. It could not compute a single p-value. `test.*` held
ten operations that all returned a number and left the decision to whoever read
it — which, for an engine whose whole purpose is to do the arithmetic an LLM
gets wrong, is the arithmetic left undone.

The gap was structural rather than an oversight in the test family: `prob.*` had
fifty-five operations and **no t, chi-square or F distribution**, and `special.*`
had eighteen and no regularized incomplete gamma or beta. Without those two
functions none of the three distributions can be written, and without the
distributions there are no p-values. So the batch is three layers, built bottom
up, each verified before the next was written.

| layer | operations | what it unlocks |
|---|---:|---|
| `special.*` | 3 | the regularized incomplete gamma and beta |
| `prob.*` | 22 | t, chi-square, F, gamma, beta, lognormal, Weibull |
| `test.*` | 27 | p-values, complete tests, intervals, sample sizing |
| `reg.*` | 17 | multiple regression and its diagnostics |

## The foundation

`special.gamma_p`, `special.gamma_q` and `special.beta_inc` are the standard
series/continued-fraction pair, with a **fixed iteration cap and a fixed
tolerance**. The cap is not a convergence criterion — at 300 iterations both have
long since converged for every argument the domain guards admit. It is there so
the loop count cannot depend on the input in a way that might differ between
platforms, because this engine's contract is byte-identical output.

Verified against **mpmath at 50 digits**, not against scipy: on a grid of 220
values the worst relative error is `2.6e-13`, and it occurs at values like
`6.17e-71`. Relative accuracy that far into the tail is the whole point — a
p-value of `1e-15` has to mean something.

Three of those 220 points are ones where **scipy itself disagrees with mpmath**
past the tolerance. They are in the deep tail where scipy loses relative
accuracy. The check reports them and uses mpmath as the reference there rather
than treating the disagreement as our error.

## Tail probabilities are computed, not subtracted

Every distribution exposes a survival function next to its CDF. This is not
symmetry for its own sake:

```
prob.normal_sf(10, 0, 1)      = 7.619853e-24     (scipy agrees)
1 - prob.normal_cdf(10, 0, 1) = 0.0
```

The CDF at z = 10 rounds to exactly 1.0 in f64, so the naive form returns zero
where the true answer is 7.6e-24. The loss is total there and merely large
elsewhere: at `t_sf(40, 5)` the survival branch is accurate to `7e-16` and
`1 - cdf` to `2.9e-10`. A p-value is a tail probability, and the tail is exactly
where the subtraction throws away the digits that matter.

The same reasoning shapes the quantiles. Asking for the `1e-8` quantile by
solving `cdf(x) = 1 - 1e-8` discards eight digits before the search starts,
because `1 - 1e-8` is only known to about `1e-16` absolute. **The first
implementation did exactly that** and was caught by the scipy comparison at
`t_ppf(1e-8, df=1)`, off by a relative `1e-8`. Each quantile now solves the CDF
for the lower half and the survival function for the upper half, and never forms
`1 - p` on the tail it is already working on. Quantiles are found by bisection
with a fixed iteration count, for the same determinism reason as above; Newton
would be faster and would make the answer depend on the starting guess.

## What the tests give back

The complete tests return an object — statistic, degrees of freedom and p —
rather than a bare number, because all three are needed to report a result and
recomputing the df at the call site is where mistakes get made. `test.p_t`,
`test.p_z`, `test.p_chi2` and `test.p_f` are the generic converters for anyone
who already has a statistic from elsewhere.

Beyond the p-values the batch adds what was simply absent: a paired t test,
one-way ANOVA, Mann-Whitney U, Wilcoxon signed rank, Levene, Bartlett,
Kolmogorov-Smirnov, Fisher's exact test, the exact binomial test, McNemar,
Yates-corrected chi-square, four confidence intervals, two sample-size
calculations and a power estimate.

**Where a convention is contested the choice is stated rather than defaulted:**

* `test.levene` centres on the median (Brown-Forsythe), which stays honest on
  skewed data.
* `test.ks_normal` takes the normal's parameters as arguments instead of
  estimating them from the sample. Estimating them turns this into Lilliefors'
  test, whose null distribution is different; reporting a KS p-value for it would
  overstate the fit. Its p is the asymptotic Kolmogorov form and is labelled
  asymptotic in the operation summary and compared against the asymptotic form.
* `test.fisher_exact` and `test.binomial_test` are two-sided by the "sum every
  outcome at least as extreme" rule.
* The rank tests use the tie-corrected normal approximation with a continuity
  correction. The exact permutation p-value is not offered: it is tractable only
  for small samples, and an operation whose cost explodes with input size does
  not belong in a batch engine.
* `reg.theil_sen` uses the separate-median intercept, `median(y) − slope ·
  median(x)`. The alternative, `median(y − slope · x)`, is also in use and gives
  a different answer on the same data. This one matches the common
  implementations, so a result can be checked against them without the difference
  being mistaken for an error.

## Regression

`reg.*` could fit a line through two vectors and say nothing about whether the
line meant anything. It now has `reg.multiple` (least squares with the intercept
column added by the engine, so nobody accidentally fits through the origin), the
standard errors, t, p and confidence interval for a slope, the overall F test,
adjusted R², AIC and BIC, Durbin-Watson, polynomial fitting, Kendall's tau-b, the
three linearisable fits, and Theil-Sen for when the data has an outlier in it.

`stat.spearman` already existed, so no rank correlation was duplicated.

## Verification

`scripts/verify_statistics.py` drives the real binary over MCP and checks **961
values** against scipy, numpy and mpmath — references that are genuinely separate
implementations rather than restatements of the Rust. It runs in CI.

Not all of it is value comparison. It also asserts that:

* `P(a,x) + Q(a,x)` is 1 to `1e-14`, which a naive `1 - P` would not satisfy;
* `I_x(a,b) + I_(1-x)(b,a)` is 1, the reflection both incomplete-beta branches
  must obey;
* each survival function beats `1 - cdf` in the far tail, with the normal at
  z = 10 as the case where the difference is total;
* each quantile inverts its own CDF, independently of scipy.

17 Rust tests carry identities that need no external reference: F equals t² on
one degree of freedom and the two p-values must be identical; a paired test is a
one-sample test on the differences; Welch and the pooled test coincide at equal
sizes and variances; the two Mann-Whitney statistics sum to n₁n₂ and swapping the
samples must leave p unchanged; a wider confidence level must contain the
narrower interval; multiple regression must recover an exactly linear
relationship with R² of one; Theil-Sen must hold near the true slope when one
point is moved to 900 while least squares is dragged past 50.

**Two of those tests were wrong on the first run, and the code was right.**
Durbin-Watson on six alternating residuals is exactly 20/6, not the >3.9 the
bound suggests — 4 is approached only asymptotically. And BIC exceeds AIC only
once `ln(n) > 2`, so at n = 6 it is the smaller of the two; the test's own comment
said "more than seven points" while its assertion said otherwise. Both
expectations were corrected and the crossover is now pinned in both directions.

## A defect this batch exposed

Adding operations to `test.*` made the dispatcher's shape matter, and it turned
out `advanced_stats::execute` claims the whole `test.` and `reg.` prefixes with
`op.starts_with`. Anything it does not itself implement gets `Some(Err("OP"))`,
so `.or_else` never runs and every operation added to those families afterwards
would have been swallowed. `inference::execute` is now asked first; it matches an
explicit list, so going first cannot shadow anything that already worked.

Separately, and more seriously, `docs`-driven review of this tree turned up a
**v1.1 defect that this batch did not cause but did have to fix**: `expr.eval`
was classified `Pure` and could therefore be sent to a worker, where
`engine::execute` has no arm for it. See `docs/V11_SAFETY_MODEL.md` for the full
account. It broke the byte-identical-across-worker-counts invariant, no existing
test covered the shape that triggers it, and the test that existed asserted the
wrong property and pinned the bug in place.
