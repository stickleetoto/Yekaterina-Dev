"""Frozen benchmark workload definitions for Yekaterina v1.0.0 / v1.1.

Everything here is deterministic: no randomness, no clock-dependent input, no
environment-dependent sizing. The same workload id must mean the same bytes on
every machine and in every release, otherwise baseline comparison is meaningless.

WORKLOAD_SET_VERSION must be bumped if an existing workload payload changes.
Adding a new workload id does not require a bump; changing an existing one does,
and invalidates comparison against previously frozen baselines.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable

WORKLOAD_SET_VERSION = 1

# Server-side limits (mirrored from src/server.rs) that constrain workload sizing.
MAX_BATCH = 1024
MAX_PIPE = 256
MAX_RESULT_NODES = 100_000

Calls = Callable[[], list[tuple[str, dict[str, Any]]]]


def ramp(n: int) -> list[float]:
    """Deterministic float vector 0.0 .. n-1."""
    return [float(i) for i in range(n)]


def matrix(n: int) -> list[list[float]]:
    """Deterministic n x n matrix with small bounded integer-valued entries."""
    return [[float((i * j) % 7 + 1) for j in range(n)] for i in range(n)]


def signal(n: int) -> list[float]:
    return [float(i % 13) for i in range(n)]


@dataclass
class Workload:
    id: str
    category: str
    description: str
    # One measured iteration = one or more tool calls. ``calls`` returns the
    # list of (tool_name, arguments) executed per iteration. The pseudo tool
    # name "__tools_list__" issues a raw tools/list request.
    calls: Calls | None
    reps: int = 15
    warmup: int = 3
    # Number of logical operations per iteration, for ops/sec derivation.
    ops_per_iter: int = 1
    # Iterations that legitimately return an {"e":...} envelope (limit probes).
    expect_error: bool = False
    # Driver-implemented workloads that cannot be expressed as replayable calls.
    special: str = ""
    notes: str = ""
    tags: list[str] = field(default_factory=list)


def one(tool: str, args: dict[str, Any]) -> Calls:
    return lambda: [(tool, args)]


def batch(items: list[Any]) -> Calls:
    return lambda: [("yk.compute", {"ops": items})]


def build_workloads() -> list[Workload]:
    w: list[Workload] = []

    # ---- 1. protocol floor -------------------------------------------------
    w.append(Workload(
        "protocol.floor_arg_error", "protocol",
        "Empty compute request; round trip with zero compute.",
        one("yk.compute", {}), reps=200, warmup=20, expect_error=True,
        notes="Upper bound on protocol cost; includes harness overhead.",
        tags=["floor"]))
    w.append(Workload(
        "protocol.tools_list", "protocol",
        "tools/list round trip.",
        lambda: [("__tools_list__", {})], reps=100, warmup=10, tags=["floor"]))
    w.append(Workload(
        "protocol.find", "protocol",
        "yk.find lazy discovery.",
        one("yk.find", {"q": "mean"}), reps=100, warmup=10))
    w.append(Workload(
        "protocol.spec", "protocol",
        "yk.spec contract lookup.",
        one("yk.spec", {"op": "stat.mean"}), reps=100, warmup=10))

    # ---- 2. single-op latency ---------------------------------------------
    singles = [
        ("single.math_add", {"op": "math.add", "a": [1, 2]},
         "Cheapest possible op."),
        ("single.stat_mean_100", {"op": "stat.mean", "a": [ramp(100)]},
         "Small array reduction."),
        ("single.mat_mul_100", {"op": "mat.mul", "a": [matrix(100), matrix(100)]},
         "1,000,000 MAC; below the 2M product cap."),
        ("single.mat_shape_100", {"op": "mat.shape", "a": [matrix(100)]},
         "Same input shape as mat_mul_100 with O(1) compute; isolates the input path."),
        ("single.signal_fft_2048", {"op": "signal.fft", "a": [signal(2048)]},
         "O(n log n) transform with a large result payload."),
        ("single.num_integrate_10k", {"op": "num.integrate", "a": [{"e": "x*x"}, 0, 1, 10000]},
         "10,001 expression evaluations."),
        ("single.ode_rk4_10k", {"op": "ode.rk4", "a": [{"e": "y"}, 0, 1, 1, 10000]},
         "40,000 expression evaluations."),
        ("single.int_pow_exact", {"op": "int.pow", "a": ["7", 2000]},
         "BigInt exact path."),
        ("single.dec_mul_exact", {"op": "dec.mul", "a": ["1.0000000000000000001", "3.5"]},
         "BigDecimal exact path."),
        ("single.expr_eval", {"op": "expr.eval", "a": [{"e": "a*b+1", "v": {"a": 2, "b": 5}}]},
         "Stateless expression evaluation."),
    ]
    for wid, args, desc in singles:
        w.append(Workload(wid, "single", desc, one("yk.compute", args), reps=25, warmup=5))

    # ---- 3. batch scaling (scalar, independent) ---------------------------
    for n in (10, 100, 1000):
        w.append(Workload(
            f"batch.scalar_{n}", "batch",
            f"{n} independent math.add items in one batch.",
            batch([["math.add", 1, 2] for _ in range(n)]),
            reps=15 if n < 1000 else 11, warmup=3, ops_per_iter=n,
            tags=["independent"]))
    # 10k exceeds MAX_BATCH (1024) and must be chunked, exactly as the historical
    # alpha benchmark did (10 calls of 1000).
    w.append(Workload(
        "batch.scalar_10000_chunked", "batch",
        "10,000 math.add items as 10 sequential batches of 1,000 (MAX_BATCH=1024).",
        lambda: [("yk.compute", {"ops": [["math.add", 1, 2] for _ in range(1000)]})
                 for _ in range(10)],
        reps=7, warmup=2, ops_per_iter=10000, tags=["independent"]))

    # ---- 4. cheap-batch regression guard ----------------------------------
    for n in (2, 4, 8, 16, 32):
        w.append(Workload(
            f"cheap.scalar_{n}", "cheap",
            f"{n} trivial items; must never regress and should stay sequential.",
            batch([["math.add", 1, 2] for _ in range(n)]),
            reps=200, warmup=20, ops_per_iter=n,
            tags=["independent", "regression-guard"]))

    # ---- 5. accumulated-result rescan probe (the O(n^2) finding) ----------
    for n in (10, 50, 100, 200, 400):
        w.append(Workload(
            f"rescan.cumsum100_x{n}", "rescan",
            f"{n} x stat.cumsum over 100 floats; exercises accumulated-result validation.",
            batch([["stat.cumsum", ramp(100)] for _ in range(n)]),
            reps=9 if n <= 100 else 7, warmup=2, ops_per_iter=n,
            tags=["independent", "quadratic-probe"]))

    # ---- 6. heavy independent batch (parallel candidates) ------------------
    for n in (4, 16, 64):
        w.append(Workload(
            f"heavy.integrate1000_x{n}", "heavy",
            f"{n} x num.integrate (1,000 steps); independent, compute-bound, small payload.",
            batch([["num.integrate", {"e": "x*x"}, 0, 1, 1000] for _ in range(n)]),
            reps=9, warmup=2, ops_per_iter=n,
            tags=["independent", "parallel-candidate"]))
    for n in (8, 16, 32):
        # 32 x 512 complex results is about 49k nodes, safely under MAX_RESULT_NODES.
        w.append(Workload(
            f"heavy.fft512_x{n}", "heavy",
            f"{n} x signal.fft over 512 samples; independent, large result payload.",
            batch([["signal.fft", signal(512)] for _ in range(n)]),
            reps=9, warmup=2, ops_per_iter=n,
            tags=["independent", "parallel-candidate"]))
    w.append(Workload(
        "heavy.mixed_skew_16", "heavy",
        "Skewed batch: alternating signal.dft(256) and math.add, so completion "
        "order necessarily differs from input order.",
        batch([(["signal.dft", signal(256)] if i % 2 == 0 else ["math.add", 1, 2])
               for i in range(16)]),
        reps=7, warmup=2, ops_per_iter=16,
        tags=["independent", "parallel-candidate", "skew"]))

    # ---- 6b. large array arguments (argument-clone probe) -----------------
    # Added in Phase 2B. v1.0.0 deep-copied each item's arguments twice: once in
    # parse_step and again in resolve_args, even with no $ references present.
    # These workloads carry the argument weight that makes that visible.
    for n, items in ((1000, 16), (10000, 4)):
        w.append(Workload(
            f"bigarg.sum{n}_x{items}", "bigarg",
            f"{items} x stat.sum over {n} floats; large reference-free arguments.",
            batch([["stat.sum", ramp(n)] for _ in range(items)]),
            reps=9, warmup=2, ops_per_iter=items,
            tags=["independent", "argument-clone-probe"]))
    w.append(Workload(
        "bigarg.ref_sum1000_x16", "bigarg",
        "Same shape but every item references $input, forcing the substituting "
        "path; guards against the borrow optimization regressing that path.",
        lambda: [("yk.compute", {"ops": [["stat.sum", "$input"] for _ in range(16)],
                                 "input": ramp(1000)})],
        reps=9, warmup=2, ops_per_iter=16,
        tags=["dependent", "argument-clone-probe"]))

    # ---- 6c. expression-driven optimizers --------------------------------
    # Added in Phase 2C-3. The optimize.* family evaluates its expression in a
    # loop the same way num./ode. do; argmin_grid alone permits 1,000,000
    # evaluations per request. Individually the iterative solvers land below the
    # 0.20 ms noise floor, so they are batched to make them measurable.
    w.append(Workload(
        "single.optimize_argmin_100k", "optimize",
        "optimize.argmin_grid over 100,000 grid points; 100,000 evaluations.",
        one("yk.compute", {"op": "optimize.argmin_grid",
                           "a": [{"e": "(x-2)*(x-2)"}, -10, 10, 100000]}),
        reps=9, warmup=2, tags=["expression-solver"]))
    w.append(Workload(
        "heavy.optimize_nelder_x32", "optimize",
        "32 x optimize.nelder_mead2d; exercises the comparator that calls the "
        "evaluator twice inside one expression.",
        batch([["optimize.nelder_mead2d", {"e": "(x-1)*(x-1)+(y-2)*(y-2)"}, 0, 0]
               for _ in range(32)]),
        reps=9, warmup=2, ops_per_iter=32,
        tags=["independent", "expression-solver"]))
    w.append(Workload(
        "heavy.optimize_gd2d_x32", "optimize",
        "32 x optimize.gradient_descent2d; two-variable evaluation loop.",
        batch([["optimize.gradient_descent2d",
                {"e": "(x-1)*(x-1)+(y-2)*(y-2)"}, 0, 0, 0.05] for _ in range(32)]),
        reps=9, warmup=2, ops_per_iter=32,
        tags=["independent", "expression-solver"]))

    # ---- 7. dependent batch (must NOT be parallelised naively) ------------
    w.append(Workload(
        "dependent.chain_64", "dependent",
        "64-item batch where each item references the previous via $N.",
        batch([["math.add", 1, 2]] + [["math.add", f"${i - 1}", 1] for i in range(1, 64)]),
        reps=15, warmup=3, ops_per_iter=64, tags=["dependent"]))
    w.append(Workload(
        "dependent.half_64", "dependent",
        "64-item batch: first 32 independent, last 32 chained; exercises wave planning.",
        batch([["math.add", 1, 2] for _ in range(32)]
              + [["math.add", f"${31 + i}", 1] for i in range(32)]),
        reps=15, warmup=3, ops_per_iter=64, tags=["mixed"]))

    # ---- 8. pipeline -------------------------------------------------------
    for n in (10, 100, 256):
        w.append(Workload(
            f"pipeline.chain_{n}", "pipeline",
            f"{n}-step chained pipeline, last value only.",
            one("yk.compute", {"pipe": [["math.add", 1, 2]]
                               + [["math.add", f"${i - 1}", 1] for i in range(1, n)]}),
            reps=15, warmup=3, ops_per_iter=n))
    for n in (10, 100, 256):
        w.append(Workload(
            f"pipeline.cumsum100_all_{n}", "pipeline",
            f"{n} independent cumsum steps with all=true; exercises result accumulation.",
            one("yk.compute", {"pipe": [["stat.cumsum", ramp(100)] for _ in range(n)],
                               "all": True}),
            reps=9, warmup=2, ops_per_iter=n, tags=["quadratic-probe"]))

    # ---- 9. input / output path isolation ---------------------------------
    for n in (1000, 10000, 50000):
        w.append(Workload(
            f"iopath.count_{n}", "iopath",
            f"stat.count over {n} floats: input path with O(1) compute and tiny output.",
            one("yk.compute", {"op": "stat.count", "a": [ramp(n)]}),
            reps=9, warmup=2, tags=["input-path"]))
        w.append(Workload(
            f"iopath.cumsum_{n}", "iopath",
            f"stat.cumsum over {n} floats: same input, {n}-element output. "
            f"Delta against iopath.count_{n} isolates the output path.",
            one("yk.compute", {"op": "stat.cumsum", "a": [ramp(n)]}),
            reps=9, warmup=2, tags=["output-path"]))

    # ---- 10. stateful ------------------------------------------------------
    w.append(Workload(
        "stateful.udo_formula_define", "stateful",
        "udo.formula definition: registry write lock, full snapshot, fsync, rename.",
        None, reps=20, warmup=2, special="udo_define",
        tags=["serialized", "fsync"]))

    return w


def workload_index() -> dict[str, Workload]:
    return {x.id: x for x in build_workloads()}
