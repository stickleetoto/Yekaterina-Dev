"""Pipelined MCP probe: measures server-side concurrency, not round-trip latency.

`bench/run_bench.py` and `bench/paired_ab.py` both send one request and wait for
its response, so neither can observe anything about how the server behaves when
several requests are in flight at once. That matters from Phase 3 onward:

* Phase 3 moved the snapshot `fsync` out of the registry write lock. The point of
  that change is that a reader no longer stalls behind a writer's disk I/O, which
  is invisible to a serial client.
* Phases 5-7 add parallel execution, whose whole purpose is overlapping work.

This probe writes N JSON-RPC requests back to back without waiting, then reads
the responses and reports per-request completion times. `rmcp` dispatches each
`tools/call` as an independent task, so pipelining is what actually exercises the
server's concurrency.

Usage:
    python bench/pipelined_probe.py --exe target/release/yekaterina.exe
    python bench/pipelined_probe.py --a old.exe --b new.exe --rounds 9
"""
from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]


def pct(xs: list[float], q: float) -> float:
    if not xs:
        return 0.0
    ys = sorted(xs)
    k = (len(ys) - 1) * q
    lo, hi = math.floor(k), math.ceil(k)
    return ys[lo] if lo == hi else ys[lo] * (hi - k) + ys[hi] * (k - lo)


class PipelinedClient:
    """Minimal stdio JSON-RPC client that does not wait between requests."""

    def __init__(self, exe: str, home: str):
        env = os.environ.copy()
        env["YEKATERINA_HOME"] = home
        self.proc = subprocess.Popen(
            [exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, encoding="utf-8",
            errors="replace", bufsize=1, env=env,
        )
        self.lock = threading.Lock()
        self.done: dict[int, float] = {}
        self.reader = threading.Thread(target=self._pump, daemon=True)
        self.reader.start()
        self._id = 0
        self._rpc("initialize", {
            "protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "yk-pipelined", "version": "1.1"},
        })
        self._send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def _pump(self) -> None:
        for line in self.proc.stdout:
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except json.JSONDecodeError:
                continue
            rid = msg.get("id")
            if rid is not None:
                with self.lock:
                    self.done[rid] = time.perf_counter_ns()

    def _send(self, obj: dict[str, Any]) -> None:
        self.proc.stdin.write(json.dumps(obj, separators=(",", ":")) + "\n")
        self.proc.stdin.flush()

    def _rpc(self, method: str, params: Any) -> None:
        self._id += 1
        self._send({"jsonrpc": "2.0", "id": self._id, "method": method, "params": params})
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            with self.lock:
                if self._id in self.done:
                    return
            time.sleep(0.001)
        raise RuntimeError(f"timeout waiting for {method}")

    def burst(self, calls: list[tuple[str, dict[str, Any]]], timeout: float = 120.0
              ) -> dict[int, float]:
        """Fire every call without waiting; return {index: latency_ms}."""
        with self.lock:
            self.done.clear()
        base = self._id
        sent: dict[int, float] = {}
        for i, (tool, args) in enumerate(calls):
            self._id += 1
            sent[self._id] = time.perf_counter_ns()
            self._send({"jsonrpc": "2.0", "id": self._id, "method": "tools/call",
                        "params": {"name": tool, "arguments": args}})
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            with self.lock:
                if len(self.done) >= len(calls):
                    break
            time.sleep(0.001)
        with self.lock:
            finished = dict(self.done)
        return {rid - base - 1: (finished[rid] - sent[rid]) / 1e6
                for rid in sent if rid in finished}

    def close(self) -> None:
        try:
            self.proc.stdin.close()
            self.proc.terminate()
            self.proc.wait(timeout=2)
        except Exception:
            try:
                self.proc.kill()
            except Exception:
                pass


def mixed_burst(writes: int, reads: int, tag: str) -> list[tuple[str, dict[str, Any]]]:
    """Interleave persisting writes with cheap reads.

    Each write takes the store gate and performs a real fsync. In v1.0.0 that
    happened while holding the registry write lock, so every read behind it
    stalled. Reads here are `math.add`, which never touches the user registry,
    plus `udo.list`, which does.
    """
    calls: list[tuple[str, dict[str, Any]]] = []
    per = max(1, reads // max(writes, 1))
    for w in range(writes):
        calls.append(("yk.compute", {"op": "udo.formula",
                                     "a": [{"op": f"user.{tag}{w}", "p": ["x"], "expr": "x*2"}]}))
        for _ in range(per):
            calls.append(("yk.compute", {"op": "math.add", "a": [1, 2]}))
            calls.append(("yk.compute", {"op": "udo.list", "a": []}))
    return calls


def run(exe: str, rounds: int, writes: int, reads: int) -> dict[str, float]:
    read_lat: list[float] = []
    total: list[float] = []
    for r in range(rounds):
        with tempfile.TemporaryDirectory(prefix="yk-pipe-") as home:
            c = PipelinedClient(exe, home)
            try:
                calls = mixed_burst(writes, reads, f"r{r}_")
                t0 = time.perf_counter_ns()
                lat = c.burst(calls)
                total.append((time.perf_counter_ns() - t0) / 1e6)
                if len(lat) != len(calls):
                    raise RuntimeError(f"{len(calls) - len(lat)} responses missing")
                # Indices that are not writes are reads.
                per = max(1, reads // max(writes, 1))
                stride = 1 + 2 * per
                read_lat.extend(v for i, v in lat.items() if i % stride != 0)
            finally:
                c.close()
    return {
        "read_p50_ms": pct(read_lat, 0.50),
        "read_p95_ms": pct(read_lat, 0.95),
        "read_p99_ms": pct(read_lat, 0.99),
        "read_max_ms": max(read_lat) if read_lat else 0.0,
        "burst_total_ms": statistics.median(total),
        "reads": len(read_lat),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe")
    ap.add_argument("--a")
    ap.add_argument("--b")
    ap.add_argument("--label-a", default="A")
    ap.add_argument("--label-b", default="B")
    ap.add_argument("--rounds", type=int, default=7)
    ap.add_argument("--writes", type=int, default=12)
    ap.add_argument("--reads", type=int, default=120)
    args = ap.parse_args()

    def report(name: str, exe: str) -> dict[str, float]:
        m = run(exe, args.rounds, args.writes, args.reads)
        print(f"{name:<10} reads n={m['reads']:<5} "
              f"p50={m['read_p50_ms']:7.3f}  p95={m['read_p95_ms']:8.3f}  "
              f"p99={m['read_p99_ms']:8.3f}  max={m['read_max_ms']:8.3f}  "
              f"burst={m['burst_total_ms']:8.3f} ms")
        return m

    print(f"pipelined burst: {args.writes} persisting writes interleaved with "
          f"~{args.reads * 2} reads, {args.rounds} rounds")
    print("read latency is what a writer's fsync can stall\n")

    if args.exe:
        report("exe", args.exe)
        return
    if not (args.a and args.b):
        raise SystemExit("give --exe, or both --a and --b")

    # Alternate to cancel drift, as bench/paired_ab.py does.
    ma, mb = [], []
    for r in range(3):
        if r % 2 == 0:
            ma.append(run(args.a, args.rounds, args.writes, args.reads))
            mb.append(run(args.b, args.rounds, args.writes, args.reads))
        else:
            mb.append(run(args.b, args.rounds, args.writes, args.reads))
            ma.append(run(args.a, args.rounds, args.writes, args.reads))
    for key in ("read_p50_ms", "read_p95_ms", "read_p99_ms", "read_max_ms", "burst_total_ms"):
        a = statistics.median(x[key] for x in ma)
        b = statistics.median(x[key] for x in mb)
        ratio = b / a if a > 0 else 0.0
        print(f"{key:<16} {args.label_a}={a:9.3f}  {args.label_b}={b:9.3f}  "
              f"B/A={ratio:6.3f}x")


if __name__ == "__main__":
    main()
