"""Interleaved paired A/B comparison between two Yekaterina binaries.

Why this exists alongside ``run_bench.py --compare``:

Comparing a fresh run against a stored baseline conflates code differences with
machine drift. During Phase 0 the stored-baseline comparison reported a
consistent +14% to +26% shift across every sub-millisecond workload after a
change that turned out to be performance-neutral; a paired run of the same two
binaries measured -0.35%. The difference was entirely session-to-session drift.

This script alternates A and B inside one wall-clock window, flips which binary
goes first on every round, and reports the paired median ratio plus a sign test.
Slow drift and thermal effects hit both binaries equally and cancel.

Use ``run_bench.py --compare`` for the full suite and release reporting.
Use this for any single decision where the expected effect is under ~10%.

Absolute times here run higher than run_bench.py because both servers are alive
at once. Only the ratio is meaningful.

Usage:
    python bench/paired_ab.py --a old.exe --b new.exe --only batch,pipeline
"""
from __future__ import annotations

import argparse
import math
import statistics
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bench"))

from bench_client import BenchClient  # noqa: E402
import workloads as W  # noqa: E402


def pct(xs: list[float], q: float) -> float:
    ys = sorted(xs)
    k = (len(ys) - 1) * q
    lo, hi = math.floor(k), math.ceil(k)
    return ys[lo] if lo == hi else ys[lo] * (hi - k) + ys[hi] * (k - lo)


def sample(client: BenchClient, plan, reps: int, warmup: int) -> list[float]:
    def issue():
        for tool, args in plan:
            if tool == "__tools_list__":
                client.tools_list()
            else:
                client.tool_call(tool, args)

    for _ in range(warmup):
        issue()
    ts = []
    for _ in range(reps):
        t0 = time.perf_counter_ns()
        issue()
        ts.append((time.perf_counter_ns() - t0) / 1e6)
    return ts


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--a", required=True, help="baseline binary")
    ap.add_argument("--b", required=True, help="candidate binary")
    ap.add_argument("--label-a", default="A")
    ap.add_argument("--label-b", default="B")
    ap.add_argument("--rounds", type=int, default=9)
    ap.add_argument("--reps", type=int, default=0,
                    help="override reps per round; 0 uses each workload's own value")
    ap.add_argument("--only", default=None,
                    help="comma-separated substrings; matches workload ids")
    ap.add_argument("--argv-a", default="", help="space separated argv for A")
    ap.add_argument("--argv-b", default="", help="space separated argv for B")
    args = ap.parse_args()

    filters = [x.strip() for x in args.only.split(",")] if args.only else []
    wls = [x for x in W.build_workloads()
           if x.calls is not None and (not filters or any(f in x.id for f in filters))]
    if not wls:
        raise SystemExit("FAIL: no workloads matched --only")

    per: dict[str, dict[str, list[float]]] = {x.id: {"a": [], "b": []} for x in wls}
    plans = {x.id: x.calls() for x in wls}  # type: ignore[misc]

    with tempfile.TemporaryDirectory(prefix="yk-ab-a-") as ha, \
            tempfile.TemporaryDirectory(prefix="yk-ab-b-") as hb:
        with BenchClient(args.a, argv=args.argv_a.split() or None,
                         env={"YEKATERINA_HOME": ha}, timeout=600) as ca, \
             BenchClient(args.b, argv=args.argv_b.split() or None,
                         env={"YEKATERINA_HOME": hb}, timeout=600) as cb:
            for r in range(args.rounds):
                order = [("a", ca), ("b", cb)] if r % 2 == 0 else [("b", cb), ("a", ca)]
                for wl in wls:
                    reps = args.reps or wl.reps
                    for tag, cl in order:
                        per[wl.id][tag].append(
                            pct(sample(cl, plans[wl.id], reps, wl.warmup), 0.50))
                print(f"  round {r + 1}/{args.rounds}", flush=True)

    print()
    print(f'{"workload":<34}{args.label_a:>11}{args.label_b:>11}{"B/A":>9}{"sign":>13}')
    print("-" * 78)
    ratios = []
    for wl in wls:
        a = statistics.median(per[wl.id]["a"])
        b = statistics.median(per[wl.id]["b"])
        ratio = b / a if a > 0 else 0.0
        ratios.append(ratio)
        slower = sum(1 for x, y in zip(per[wl.id]["a"], per[wl.id]["b"]) if y > x)
        n = len(per[wl.id]["a"])
        print(f"{wl.id:<34}{a:>10.4f}m{b:>10.4f}m{ratio:>8.3f}x{f'{slower}/{n} slower':>13}")

    med = statistics.median(ratios)
    slower_cases = sum(1 for r in ratios if r > 1.0)
    print()
    print(f"median B/A ratio: {med:.4f}  ({(med - 1) * 100:+.2f}%)")
    print(f"cases where B is slower: {slower_cases}/{len(ratios)}")
    print()
    if med > 1.02:
        print("VERDICT: B is slower. Investigate before merging.")
    elif med < 0.98:
        print("VERDICT: B is faster.")
    else:
        print("VERDICT: no paired difference outside +/-2%.")


if __name__ == "__main__":
    main()
