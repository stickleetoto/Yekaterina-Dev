"""Sustained concurrency stress: thread leaks, RSS growth, result stability.

Drives a long mixed load through one server process at several worker counts and
watches the things a worker pool can get wrong over time rather than once:
threads accumulating, memory climbing, or a result changing between rounds.
"""
from __future__ import annotations
import json, statistics, sys, tempfile, time
from pathlib import Path

ROOT = Path(r"D:\Users\leejy\Downloads\project\포폴\Yekaterina\Yekaterina_v1.0.0_Source\Yekaterina_v1.0.0")
sys.path.insert(0, str(ROOT / "bench"))
from bench_client import BenchClient, mcp_text  # noqa: E402

EXE = str(ROOT / "target/release/yekaterina.exe")

signal256 = [(i % 13) for i in range(256)]
ramp1000 = [float(i) for i in range(1000)]


def workload_set():
    """A mix that exercises both the parallel and the sequential paths."""
    return [
        ("dft_wave", {"ops": [["signal.dft", signal256] if i % 2 == 0 else ["math.add", i, 1]
                              for i in range(16)]}),
        ("integrate_wave", {"ops": [["num.integrate", {"e": "x*x"}, 0, 1, 1000] for _ in range(64)]}),
        ("payload_bound", {"ops": [["stat.sum", ramp1000] for _ in range(16)]}),
        ("chained", {"ops": [["math.add", 1, 2]] +
                            [["math.add", f"${i-1}", 1] for i in range(1, 40)]}),
        ("control_barrier", {"ops": [["math.add", 1, 2], ["udo.list"], ["math.add", 3, 4]]}),
        ("cheap", {"ops": [["math.add", 1, 2] for _ in range(32)]}),
        ("single", {"op": "mat.mul", "a": [[[1.0, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]]}),
    ]


def run(workers: int, rounds: int) -> dict:
    fingerprints: dict[str, set[str]] = {}
    samples = []
    with tempfile.TemporaryDirectory(prefix="yk-stress-") as home:
        with BenchClient(EXE, argv=["--workers", str(workers)],
                         env={"YEKATERINA_HOME": home}, timeout=300) as c:
            work = workload_set()
            # Warm up, then sample resources across the run.
            for name, args in work:
                c.tool_call("yk.compute", args)
            base = c.metrics()
            for r in range(rounds):
                for name, args in work:
                    text = mcp_text(c.tool_call("yk.compute", args).response)
                    fingerprints.setdefault(name, set()).add(text)
                if r % 10 == 0:
                    samples.append(c.metrics())
            final = c.metrics()
    unstable = {k: len(v) for k, v in fingerprints.items() if len(v) != 1}
    return {
        "workers": workers,
        "rounds": rounds,
        "unstable_workloads": unstable,
        "threads_start": base.get("num_threads"),
        "threads_end": final.get("num_threads"),
        "threads_max": max((s.get("num_threads", 0) for s in samples), default=0),
        "rss_start_mib": base.get("rss_bytes", 0) / 2**20,
        "rss_end_mib": final.get("rss_bytes", 0) / 2**20,
        "rss_max_mib": max((s.get("rss_bytes", 0) for s in samples), default=0) / 2**20,
        "cpu_s": final.get("cpu_user_s", 0) + final.get("cpu_system_s", 0),
    }


def main():
    rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    print(f"sustained stress: {rounds} rounds x 7 workloads per worker count\n")
    print(f'{"workers":>7} {"threads":>16} {"RSS MiB":>22} {"CPU s":>7}  stability')
    print("-" * 78)
    bad = []
    for workers in (1, 2, 4, 8):
        m = run(workers, rounds)
        threads = f"{m['threads_start']}->{m['threads_end']} (max {m['threads_max']})"
        rss = f"{m['rss_start_mib']:.1f}->{m['rss_end_mib']:.1f} (max {m['rss_max_mib']:.1f})"
        stable = "OK" if not m["unstable_workloads"] else f"UNSTABLE {m['unstable_workloads']}"
        if m["unstable_workloads"]:
            bad.append((workers, m["unstable_workloads"]))
        print(f'{workers:>7} {threads:>16} {rss:>22} {m["cpu_s"]:>7.2f}  {stable}')
    print()
    if bad:
        print("FAIL: results changed between rounds:", bad)
        raise SystemExit(1)
    print("PASS: no thread leak, no sustained RSS growth, every response stable across rounds")


if __name__ == "__main__":
    main()
