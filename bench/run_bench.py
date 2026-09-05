"""Reproducible in-repo MCP benchmark for Yekaterina.

Scope: MCP-only. No LLM, no network, no external service.

The harness that produced ``benchmark_results/alpha3-*`` and
``benchmark_results/mcp-realworld-alpha6/`` was never committed, so v1.0.0 has
published numbers that cannot be reproduced or compared against. This script
replaces it with a versioned, deterministic, in-repo suite.

Design rules:

* Workload payloads are frozen in ``bench/workloads.py``. Same id means same
  bytes on every machine and in every release.
* Every measured workload also produces a SHA-256 of its concatenated response
  text. That fingerprint is the byte-identical determinism check used to compare
  worker counts, and it makes an accidental behaviour change impossible to miss.
* All timing is wall-clock round trip from the client, which includes harness
  overhead. Server-internal attribution is done by ``benches/micro.rs``, not
  here. The ``protocol.floor_arg_error`` workload bounds the harness cost.
* ``--runs`` repeats the whole suite in fresh processes. Percentiles are pooled
  across runs; per-run medians are also reported so run-to-run variance stays
  visible instead of being averaged away.

Usage:
    python bench/run_bench.py --exe target/release/yekaterina.exe --out bench_results/latest
    python bench/run_bench.py --exe ... --workers 4 --compare bench_results/v1.0.0-frozen/result.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import platform
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bench"))

from bench_client import BenchClient, mcp_text  # noqa: E402
import workloads as W  # noqa: E402

HARNESS_VERSION = "1.0"


# ---------------------------------------------------------------- statistics

def pct(xs: list[float], q: float) -> float:
    if not xs:
        return 0.0
    ys = sorted(xs)
    k = (len(ys) - 1) * q
    lo, hi = math.floor(k), math.ceil(k)
    return ys[lo] if lo == hi else ys[lo] * (hi - k) + ys[hi] * (k - lo)


def summarize(samples: list[float], run_medians: list[float], ops_per_iter: int) -> dict[str, Any]:
    out: dict[str, Any] = {
        "n": len(samples),
        "p50_ms": pct(samples, 0.50),
        "p95_ms": pct(samples, 0.95),
        "min_ms": min(samples) if samples else 0.0,
        "mean_ms": statistics.fmean(samples) if samples else 0.0,
        "run_p50_ms": run_medians,
    }
    if len(samples) >= 100:
        out["p99_ms"] = pct(samples, 0.99)
    if run_medians:
        med = statistics.median(run_medians)
        out["p50_stable_ms"] = med
        out["run_spread_pct"] = (
            (max(run_medians) - min(run_medians)) / med * 100.0 if med > 0 else 0.0
        )
    if out["p50_ms"] > 0:
        out["ops_per_sec"] = ops_per_iter / (out["p50_ms"] / 1000.0)
        if ops_per_iter > 1:
            out["us_per_op"] = out["p50_ms"] * 1000.0 / ops_per_iter
    return out


# ------------------------------------------------------------------ metadata

def tool_command(exe: Path, workers: str | None) -> list[str]:
    return [] if not workers else ["--workers", workers]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def source_fingerprint() -> dict[str, str]:
    """Hash the inputs that determine runtime behaviour."""
    h = hashlib.sha256()
    parts = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "rust-toolchain.toml"]
    parts += sorted((ROOT / "src").glob("*.rs"))
    for p in parts:
        h.update(p.read_bytes())
    return {"src_sha256": h.hexdigest(), "files": str(len(parts))}


def tool_version(cmd: list[str]) -> str:
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=20).stdout.strip()
    except Exception:
        return "unavailable"


def environment(exe: Path) -> dict[str, Any]:
    import os

    env: dict[str, Any] = {
        "harness_version": HARNESS_VERSION,
        "workload_set_version": W.WORKLOAD_SET_VERSION,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "python": platform.python_version(),
        "rustc": tool_version(["rustc", "--version"]),
        "cargo": tool_version(["cargo", "--version"]),
        "logical_cpus": os.cpu_count(),
        "exe": str(exe),
        "exe_sha256": sha256_file(exe),
        "exe_bytes": exe.stat().st_size,
    }
    try:
        import psutil  # type: ignore

        env["physical_cpus"] = psutil.cpu_count(logical=False)
        env["total_ram_bytes"] = psutil.virtual_memory().total
    except Exception:
        env["physical_cpus"] = None
    env.update(source_fingerprint())
    return env


# ------------------------------------------------------------------ measuring

def measure_cold_start(exe: Path, argv: list[str], reps: int) -> dict[str, Any]:
    """Process creation through MCP initialize, in fresh processes."""
    samples = []
    for _ in range(reps):
        with tempfile.TemporaryDirectory(prefix="yk-bench-cold-") as home:
            t0 = time.perf_counter_ns()
            c = BenchClient(str(exe), argv=argv, env={"YEKATERINA_HOME": home}, timeout=60)
            c.start()
            samples.append((time.perf_counter_ns() - t0) / 1e6)
            c.close()
    return summarize(samples, [pct(samples, 0.50)], 1)


def run_workload(c: BenchClient, wl: W.Workload) -> tuple[list[float], str, dict[str, Any]]:
    """Return (samples_ms, response_fingerprint, info)."""
    digest = hashlib.sha256()
    info: dict[str, Any] = {}

    if wl.special == "udo_define":
        # Each iteration must define a distinct opcode: redefining the same name
        # is a different code path (overwrite) than a fresh insert.
        samples = []
        for i in range(wl.warmup + wl.reps):
            args = {"op": "udo.formula",
                    "a": [{"op": f"user.bench_{i}", "p": ["x"], "expr": "x*2"}]}
            t0 = time.perf_counter_ns()
            r = c.tool_call("yk.compute", args)
            dt = (time.perf_counter_ns() - t0) / 1e6
            if i >= wl.warmup:
                samples.append(dt)
                digest.update(b"|")  # opcode name varies by design; do not fingerprint text
        info["note"] = "response text intentionally excluded from fingerprint (opcode name varies)"
        return samples, digest.hexdigest(), info

    assert wl.calls is not None
    plan = wl.calls()

    def issue() -> str:
        texts = []
        for tool, args in plan:
            if tool == "__tools_list__":
                resp = c.tools_list().response
                texts.append(json.dumps(resp.get("result", {}), sort_keys=True,
                                        separators=(",", ":")))
            else:
                texts.append(mcp_text(c.tool_call(tool, args).response))
        return "\n".join(texts)

    for _ in range(wl.warmup):
        issue()

    samples = []
    last = ""
    for _ in range(wl.reps):
        t0 = time.perf_counter_ns()
        last = issue()
        samples.append((time.perf_counter_ns() - t0) / 1e6)
        digest.update(last.encode("utf-8"))

    info["response_bytes"] = len(last)
    info["response_head"] = last[:60]
    is_error = last.startswith('{"e":')
    info["error_envelope"] = is_error
    if is_error and not wl.expect_error:
        info["UNEXPECTED_ERROR"] = True
    return samples, digest.hexdigest(), info


def run_suite(exe: Path, argv: list[str], runs: int, only: str | None) -> dict[str, Any]:
    wls = [x for x in W.build_workloads() if not only or only in x.id]
    per_run: dict[str, list[list[float]]] = {x.id: [] for x in wls}
    fingerprints: dict[str, set[str]] = {x.id: set() for x in wls}
    infos: dict[str, dict[str, Any]] = {}
    proc_metrics: list[dict[str, Any]] = []

    for run_i in range(runs):
        with tempfile.TemporaryDirectory(prefix="yk-bench-") as home:
            with BenchClient(str(exe), argv=argv,
                             env={"YEKATERINA_HOME": home}, timeout=300) as c:
                for wl in wls:
                    samples, fp, info = run_workload(c, wl)
                    per_run[wl.id].append(samples)
                    fingerprints[wl.id].add(fp)
                    infos[wl.id] = info
                    print(f"  run{run_i + 1} {wl.id:<34} p50={pct(samples, .50):8.3f}ms",
                          flush=True)
                m = c.metrics()
                if m:
                    proc_metrics.append(m)

    results = {}
    for wl in wls:
        pooled = [s for run in per_run[wl.id] for s in run]
        medians = [pct(run, 0.50) for run in per_run[wl.id]]
        entry = summarize(pooled, medians, wl.ops_per_iter)
        entry["category"] = wl.category
        entry["description"] = wl.description
        entry["ops_per_iter"] = wl.ops_per_iter
        entry["tags"] = wl.tags
        entry["response_sha256"] = sorted(fingerprints[wl.id])
        entry["deterministic_across_runs"] = len(fingerprints[wl.id]) == 1
        entry.update(infos.get(wl.id, {}))
        results[wl.id] = entry

    return {"workloads": results, "process_metrics": proc_metrics}


# -------------------------------------------------------------------- schema

# The frozen v1.0.0 MCP surface invariant, asserted by
# scripts/check_benchmark_result.py and RELEASE_CHECKLIST_V1.md.
FROZEN_SCHEMA_TOKENS = 412
FROZEN_SCHEMA_BYTES = 1725
FROZEN_TOOL_NAMES = ["yk.compute", "yk.find", "yk.spec"]


def canonical_schema_blob(result: dict[str, Any]) -> str:
    """The exact serialization that reproduces the frozen 412 tokens / 1725 bytes.

    This form was recovered by reconciling the published alpha.3/alpha.6 numbers
    against a live tools/list on the v1.0.0 binary. It is the whole tools/list
    ``result`` object (not the bare tools array), compact separators, sorted
    keys. Serializing the bare array instead yields 1715 bytes / 410 tokens, and
    leaving keys unsorted yields 411 tokens.

    Do not "clean this up". The exact form is the invariant.
    """
    return json.dumps({"tools": result.get("tools", [])},
                      separators=(",", ":"), sort_keys=True)


def schema_surface(exe: Path, argv: list[str]) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="yk-bench-schema-") as home:
        with BenchClient(str(exe), argv=argv,
                         env={"YEKATERINA_HOME": home}, timeout=60) as c:
            result = c.tools_list().response.get("result", {})
    tools = result.get("tools", [])
    blob = canonical_schema_blob(result)
    names = sorted(x.get("name") for x in tools)
    out: dict[str, Any] = {
        "tools": len(tools),
        "names": names,
        "schema_bytes": len(blob.encode("utf-8")),
        "serialization": "json.dumps({'tools':...}, separators=(',',':'), sort_keys=True)",
    }
    try:
        import tiktoken  # type: ignore

        out["schema_tokens"] = len(tiktoken.get_encoding("o200k_base").encode(blob))
        out["tokenizer"] = "tiktoken:o200k_base"
    except Exception:
        out["schema_tokens"] = None
        out["tokenizer"] = "unavailable"

    drift = []
    if names != FROZEN_TOOL_NAMES:
        drift.append(f"tool names {names} != {FROZEN_TOOL_NAMES}")
    if out["schema_bytes"] != FROZEN_SCHEMA_BYTES:
        drift.append(f"schema bytes {out['schema_bytes']} != {FROZEN_SCHEMA_BYTES}")
    if out["schema_tokens"] is not None and out["schema_tokens"] != FROZEN_SCHEMA_TOKENS:
        drift.append(f"schema tokens {out['schema_tokens']} != {FROZEN_SCHEMA_TOKENS}")
    out["frozen_surface_ok"] = not drift
    out["drift"] = drift
    return out


# ------------------------------------------------------------------ reporting

# A workload whose own run-to-run median spread exceeds the change being claimed
# cannot support that claim. The frozen v1.0.0 baseline shows 45-90% spread on
# sub-0.2 ms workloads (they sit on the ~0.08 ms protocol floor), so a flat 2%
# rule would fire on scheduler noise rather than on code.
REGRESSION_PCT = 2.0
NOISE_FLOOR_MS = 0.20


def compare(current: dict[str, Any], baseline_path: Path) -> dict[str, Any]:
    base = json.loads(baseline_path.read_text(encoding="utf-8-sig"))
    bw = base.get("workloads", {})
    if base.get("environment", {}).get("workload_set_version") != \
            current["environment"]["workload_set_version"]:
        raise SystemExit(
            "FAIL: workload set version differs from the baseline; "
            "payloads changed and the comparison would be meaningless")

    rows = []
    for wid, cur in current["workloads"].items():
        b = bw.get(wid)
        if not b:
            continue
        cb = b.get("p50_stable_ms") or b["p50_ms"]
        cc = cur.get("p50_stable_ms") or cur["p50_ms"]
        delta = (cc - cb) / cb * 100.0 if cb > 0 else 0.0
        # Noise band: the larger of the two runs' own median spread, floored at
        # the stated regression threshold.
        noise = max(b.get("run_spread_pct", 0.0), cur.get("run_spread_pct", 0.0),
                    REGRESSION_PCT)
        below_floor = cb < NOISE_FLOOR_MS
        significant = delta > REGRESSION_PCT and delta > noise and not below_floor
        rows.append({
            "id": wid, "category": cur["category"],
            "baseline_p50_ms": cb, "current_p50_ms": cc,
            "delta_pct": delta,
            "speedup": cb / cc if cc > 0 else 0.0,
            "noise_band_pct": noise,
            "below_noise_floor": below_floor,
            "regression": significant,
            "watch": delta > REGRESSION_PCT and not significant,
            "fingerprint_match": sorted(b.get("response_sha256", []))
                                 == sorted(cur.get("response_sha256", [])),
        })
    rows.sort(key=lambda r: r["delta_pct"], reverse=True)
    return {
        "baseline": str(baseline_path),
        "baseline_label": base.get("label"),
        "regression_pct": REGRESSION_PCT,
        "noise_floor_ms": NOISE_FLOOR_MS,
        "rows": rows,
        "regressions": [r for r in rows if r["regression"]],
        "watch": [r["id"] for r in rows if r["watch"]],
        "fingerprint_mismatches": [r["id"] for r in rows if not r["fingerprint_match"]],
    }


def write_report(out: Path, data: dict[str, Any]) -> None:
    env, schema = data["environment"], data["schema"]
    L = [
        f"# Yekaterina Benchmark - {data['label']}",
        "",
        "Scope: **MCP-only; no LLM, no network.**",
        "",
        "## Environment",
        "",
        "| Field | Value |",
        "|---|---|",
        f"| Harness / workload set | v{env['harness_version']} / v{env['workload_set_version']} |",
        f"| Platform | {env['platform']} |",
        f"| Logical / physical CPUs | {env['logical_cpus']} / {env.get('physical_cpus')} |",
        f"| rustc | {env['rustc']} |",
        f"| Workers arg | `{data['workers'] or '(default)'}` |",
        f"| Binary | {env['exe_bytes']:,} B, sha256 `{env['exe_sha256'][:16]}...` |",
        f"| Source fingerprint | `{env['src_sha256'][:16]}...` ({env['files']} files) |",
        f"| Runs x reps | {data['runs']} |",
        "",
        "## MCP surface (frozen invariant)",
        "",
        f"- Tools: **{schema['tools']}** -> {schema['names']}",
        f"- Schema bytes: **{schema['schema_bytes']}**",
        f"- Schema tokens: **{schema['schema_tokens']}** ({schema['tokenizer']})",
        f"- Frozen surface (3 tools / {FROZEN_SCHEMA_TOKENS} tokens / {FROZEN_SCHEMA_BYTES} bytes): "
        f"**{'PASS' if schema['frozen_surface_ok'] else 'FAIL -> ' + str(schema['drift'])}**",
        f"- Serialization: `{schema['serialization']}`",
        "",
        "## Cold start",
        "",
        f"- p50 **{data['cold_start']['p50_ms']:.3f} ms**, "
        f"p95 {data['cold_start']['p95_ms']:.3f} ms, "
        f"min {data['cold_start']['min_ms']:.3f} ms "
        f"(n={data['cold_start']['n']}, process spawn through MCP initialize)",
        "",
        "## Workloads",
        "",
    ]
    by_cat: dict[str, list[tuple[str, dict[str, Any]]]] = {}
    for wid, r in data["workloads"].items():
        by_cat.setdefault(r["category"], []).append((wid, r))

    for cat in sorted(by_cat):
        L += [f"### {cat}", "",
              "| Workload | p50 | p95 | stable p50 | spread | ops/s | us/op | det |",
              "|---|---:|---:|---:|---:|---:|---:|:--:|"]
        for wid, r in sorted(by_cat[cat]):
            ops = f"{r['ops_per_sec']:,.0f}" if r.get("ops_per_sec") else "-"
            uop = f"{r['us_per_op']:.2f}" if r.get("us_per_op") else "-"
            det = "ok" if r.get("deterministic_across_runs") else "**VARIES**"
            L.append(
                f"| `{wid}` | {r['p50_ms']:.3f} ms | {r['p95_ms']:.3f} ms | "
                f"{r.get('p50_stable_ms', 0):.3f} ms | {r.get('run_spread_pct', 0):.1f}% | "
                f"{ops} | {uop} | {det} |")
        L.append("")

    if data.get("process_metrics"):
        m = data["process_metrics"]
        L += ["## Process metrics (end of each run)", "",
              "| Run | RSS | Peak RSS | CPU user | CPU sys | Threads |",
              "|---:|---:|---:|---:|---:|---:|"]
        for i, x in enumerate(m, 1):
            L.append(f"| {i} | {x.get('rss_bytes', 0) / 1048576:.2f} MiB | "
                     f"{x.get('peak_rss_bytes', 0) / 1048576:.2f} MiB | "
                     f"{x.get('cpu_user_s', 0):.3f} s | {x.get('cpu_system_s', 0):.3f} s | "
                     f"{x.get('num_threads', 0)} |")
        L.append("")

    nondet = [w for w, r in data["workloads"].items() if not r.get("deterministic_across_runs")]
    unexpected = [w for w, r in data["workloads"].items() if r.get("UNEXPECTED_ERROR")]
    L += ["## Integrity", "",
          f"- Non-deterministic responses across runs: **{len(nondet)}**"
          + (f" -> {nondet}" if nondet else " (none)"),
          f"- Unexpected error envelopes: **{len(unexpected)}**"
          + (f" -> {unexpected}" if unexpected else " (none)"), ""]

    if data.get("comparison"):
        cmp = data["comparison"]
        L += ["## Comparison against baseline", "",
              f"Baseline: `{cmp['baseline']}` ({cmp.get('baseline_label')})", "",
              f"Regression rule: delta > {cmp['regression_pct']}% **and** greater than "
              f"the workload's own run-to-run spread, for workloads whose baseline "
              f"p50 is at least {cmp['noise_floor_ms']} ms. Faster-but-noisy and "
              f"below-floor rows are reported as `watch`, not as regressions.",
              "",
              "| Workload | baseline p50 | current p50 | delta | noise band | speedup | fp |",
              "|---|---:|---:|---:|---:|---:|:--:|"]
        for r in cmp["rows"]:
            flag = ("**REGRESSION**" if r["regression"]
                    else ("(watch)" if r["watch"] else ""))
            if r["below_noise_floor"]:
                flag += " _below floor_"
            fp = "ok" if r["fingerprint_match"] else "**DIFFERS**"
            L.append(f"| `{r['id']}` | {r['baseline_p50_ms']:.3f} ms | "
                     f"{r['current_p50_ms']:.3f} ms | {r['delta_pct']:+.1f}% {flag} | "
                     f"+/-{r['noise_band_pct']:.0f}% | "
                     f"{r['speedup']:.2f}x | {fp} |")
        L += ["",
              f"- Significant regressions: **{len(cmp['regressions'])}**"
              + (f" -> {[r['id'] for r in cmp['regressions']]}" if cmp["regressions"] else ""),
              f"- Watch (over threshold but within noise): **{len(cmp['watch'])}**"
              + (f" -> {cmp['watch']}" if cmp["watch"] else ""),
              f"- Fingerprint mismatches: **{len(cmp['fingerprint_mismatches'])}**"
              + (f" -> {cmp['fingerprint_mismatches']}" if cmp["fingerprint_mismatches"] else ""),
              ""]

    (out / "REPORT.md").write_text("\n".join(L) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------- main

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", required=True)
    ap.add_argument("--out", default="bench_results/latest")
    ap.add_argument("--label", default="")
    ap.add_argument("--workers", default=None,
                    help="passed through as --workers N; ignored by v1.0.0 binaries")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--cold-reps", type=int, default=15)
    ap.add_argument("--only", default=None, help="substring filter on workload id")
    ap.add_argument("--compare", default=None, help="path to a baseline result.json")
    args = ap.parse_args()

    exe = Path(args.exe).resolve()
    if not exe.exists():
        raise SystemExit(f"FAIL: binary not found: {exe}")
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    argv = tool_command(exe, args.workers)

    print(f"Yekaterina benchmark  exe={exe.name}  workers={args.workers or '(default)'}  "
          f"runs={args.runs}")
    env = environment(exe)
    schema = schema_surface(exe, argv)
    print(f"  MCP tools={schema['tools']} schema_tokens={schema['schema_tokens']} "
          f"bytes={schema['schema_bytes']} frozen_ok={schema['frozen_surface_ok']}")
    if schema["drift"]:
        for d in schema["drift"]:
            print(f"    SCHEMA DRIFT: {d}")
    print("  cold start...", flush=True)
    cold = measure_cold_start(exe, argv, args.cold_reps)
    print(f"    p50={cold['p50_ms']:.3f}ms")
    suite = run_suite(exe, argv, args.runs, args.only)

    data: dict[str, Any] = {
        "benchmark": "yekaterina-bench",
        "label": args.label or f"{exe.name} workers={args.workers or 'default'}",
        "workers": args.workers,
        "runs": args.runs,
        "environment": env,
        "schema": schema,
        "cold_start": cold,
        "workloads": suite["workloads"],
        "process_metrics": suite["process_metrics"],
    }
    if args.compare:
        data["comparison"] = compare(data, Path(args.compare))

    (out / "result.json").write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    write_report(out, data)

    nondet = [w for w, r in data["workloads"].items() if not r.get("deterministic_across_runs")]
    unexpected = [w for w, r in data["workloads"].items() if r.get("UNEXPECTED_ERROR")]
    print(f"\nWrote {out / 'result.json'} and {out / 'REPORT.md'}")
    print(f"  non-deterministic workloads: {len(nondet)} {nondet if nondet else ''}")
    print(f"  unexpected error envelopes:  {len(unexpected)} {unexpected if unexpected else ''}")
    failures = []
    if data.get("comparison"):
        c = data["comparison"]
        print(f"  significant regressions: {len(c['regressions'])} "
              f"{[r['id'] for r in c['regressions']] if c['regressions'] else ''}")
        print(f"  watch (within noise):    {len(c['watch'])}")
        print(f"  fingerprint mismatches:  {len(c['fingerprint_mismatches'])} "
              f"{c['fingerprint_mismatches'] if c['fingerprint_mismatches'] else ''}")
        if c["fingerprint_mismatches"]:
            failures.append(
                "response fingerprints differ from the baseline -> externally "
                f"observable behaviour changed: {c['fingerprint_mismatches']}")
    if unexpected:
        failures.append(f"unexpected error envelopes: {unexpected}")
    if nondet:
        failures.append(f"non-deterministic responses across runs: {nondet}")
    if not schema["frozen_surface_ok"]:
        failures.append(f"frozen MCP surface drift: {schema['drift']}")
    if failures:
        print("")
        print("BENCH GATE: FAIL")
        for f in failures:
            print(f"- {f}")
        raise SystemExit(1)
    print("")
    print("BENCH GATE: PASS")


if __name__ == "__main__":
    main()
