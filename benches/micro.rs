//! In-process microbenchmarks for Yekaterina.
//!
//! `bench/run_bench.py` measures end-to-end MCP round trips, which include the
//! Python harness, the pipe and JSON-RPC framing. It therefore cannot say how
//! much of a request is deserialization, dispatch, compute or serialization.
//! This benchmark answers that by calling the library directly.
//!
//! Deliberately harness-free (`harness = false` in Cargo.toml) so it adds no
//! dependency to a crate whose dependency set is exact-pinned and audited.
//!
//! Run:
//!     cargo bench --bench micro
//!     cargo bench --bench micro -- --json bench_results/micro.json

use std::hint::black_box;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use yekaterina::{engine, formula, registry, user_ops};

/// Target time per measured case. Iteration count adapts so that cheap cases
/// still get enough samples to be meaningful and expensive ones stay quick.
///
/// Overridable with `YK_BENCH_TARGET_MS`. Sustained tight loops settle at a
/// lower CPU clock than the bursty, round-trip-paced work `bench/run_bench.py`
/// measures, so the two harnesses are NOT directly comparable in absolute terms;
/// varying this knob is how that effect is quantified rather than assumed.
const DEFAULT_TARGET_MS: u64 = 120;
const MIN_ITERS: u64 = 32;

fn target() -> Duration {
    let ms = std::env::var("YK_BENCH_TARGET_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TARGET_MS);
    Duration::from_millis(ms)
}

struct Row {
    group: &'static str,
    name: String,
    ns_per_op: f64,
    iters: u64,
    unit: &'static str,
    per_unit_ns: Option<f64>,
    mode: &'static str,
}

/// Operations at least this expensive are measured one-shot with idle gaps
/// rather than in a tight loop. See `bench` for why.
const BURST_THRESHOLD_NS: f64 = 200_000.0;
const BURST_ROUNDS: usize = 15;
const LOOP_ROUNDS: usize = 7;

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 { xs[n / 2] } else { (xs[n / 2 - 1] + xs[n / 2]) / 2.0 }
}

/// Measure `f`, reporting the median per-op time across rounds.
///
/// Two modes, because a single mode cannot measure both honestly:
///
/// * **Loop mode** for cheap operations: iterate until the time budget is met.
///   Per-call overhead would otherwise swamp a nanosecond-scale operation.
/// * **Burst mode** for operations at or above `BURST_THRESHOLD_NS`: time one
///   call at a time with an idle gap between them.
///
/// Burst mode models how the MCP server actually meets an expensive operation:
/// one request at a time, not in a tight loop.
///
/// ## Absolute numbers from this benchmark are NOT trustworthy
///
/// Measured on the v1.0.0 baseline machine, `ode.rk4 n=10000` reported 6.4 ms,
/// 13.3 ms and 14.6 ms on three runs that differed only in `YK_BENCH_TARGET_MS`
/// -- and in all three the calibrator chose the identical iteration count, so
/// the case itself ran identically every time. The spread came from how much
/// CPU work the *preceding* cases had done: a 2.3x swing in reported cost for
/// unchanged code, driven purely by accumulated thermal/clock state.
///
/// Consequences, which the reporting honours:
///
/// * Do not use these numbers for absolute attribution ("compute is N% of a
///   request"). Cross-harness arithmetic against `bench/run_bench.py` is invalid.
/// * Do use them to compare two builds of the same case, run the same way, in
///   the same run order.
/// * The `canary` rows measure a fixed workload at the start and the end of the
///   run. Their ratio is the drift that occurred during the run and is printed
///   with every report. Treat any case whose change is smaller than that drift
///   as unmeasured.
/// * For decisions under ~10%, prefer `bench/paired_ab.py`, which interleaves
///   two binaries and cancels drift.
///
/// Never compare numbers produced under different `YK_BENCH_TARGET_MS` values.
fn bench<F: FnMut()>(group: &'static str, name: impl Into<String>, units: u64,
                     unit: &'static str, mut f: F) -> Row {
    for _ in 0..3 {
        f();
    }

    // Probe one call to choose the mode.
    let t0 = Instant::now();
    f();
    let probe_ns = t0.elapsed().as_nanos() as f64;

    let (ns_per_op, iters) = if probe_ns >= BURST_THRESHOLD_NS {
        let mut samples = Vec::with_capacity(BURST_ROUNDS);
        for _ in 0..BURST_ROUNDS {
            // Let the core return to its boost clock, as it would between
            // MCP requests.
            std::thread::sleep(Duration::from_millis(2));
            let t = Instant::now();
            f();
            samples.push(t.elapsed().as_nanos() as f64);
        }
        (median(samples), BURST_ROUNDS as u64)
    } else {
        let budget = target();
        let mut iters = MIN_ITERS;
        loop {
            let t = Instant::now();
            for _ in 0..iters {
                f();
            }
            let dt = t.elapsed();
            if dt >= budget || iters > 1 << 30 {
                break;
            }
            let scale = (budget.as_secs_f64() / dt.as_secs_f64()).clamp(2.0, 16.0);
            iters = ((iters as f64) * scale) as u64;
        }
        let mut samples = Vec::with_capacity(LOOP_ROUNDS);
        for _ in 0..LOOP_ROUNDS {
            let t = Instant::now();
            for _ in 0..iters {
                f();
            }
            samples.push(t.elapsed().as_nanos() as f64 / iters as f64);
        }
        (median(samples), iters)
    };

    Row {
        group,
        name: name.into(),
        ns_per_op,
        iters,
        unit,
        per_unit_ns: if units > 1 { Some(ns_per_op / units as f64) } else { None },
        mode: if probe_ns >= BURST_THRESHOLD_NS { "burst" } else { "loop" },
    }
}

fn ramp(n: usize) -> Vec<f64> {
    (0..n).map(|i| i as f64).collect()
}

fn matrix(n: usize) -> Vec<Vec<f64>> {
    (0..n).map(|i| (0..n).map(|j| ((i * j) % 7 + 1) as f64).collect()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json_out = args.windows(2).find(|w| w[0] == "--json").map(|w| w[1].clone());

    let mut rows: Vec<Row> = Vec::new();

    // A fixed workload measured first and last. The ratio between the two is
    // how much this machine drifted while the benchmark ran, and it bounds what
    // any other row in this report can honestly claim.
    let canary_args = vec![json!(ramp(4096))];
    let canary = |rows: &mut Vec<Row>, name: &'static str| {
        rows.push(bench("canary", name, 4096, "element", || {
            black_box(engine::execute(black_box("stat.cumsum"),
                                      black_box(&canary_args)).unwrap());
        }));
    };
    canary(&mut rows, "stat.cumsum n=4096 (start)");

    // ---------------------------------------------------------- registry
    rows.push(bench("registry", "resolve canonical (math.add)", 1, "call", || {
        black_box(registry::resolve(black_box("math.add")));
    }));
    rows.push(bench("registry", "resolve alias (avg)", 1, "call", || {
        black_box(registry::resolve(black_box("avg")));
    }));
    rows.push(bench("registry", "resolve mixed case (Math.Add)", 1, "call", || {
        black_box(registry::resolve(black_box("Math.Add")));
    }));
    rows.push(bench("registry", "resolve miss (zzz.nope)", 1, "call", || {
        black_box(registry::resolve(black_box("zzz.nope")));
    }));
    rows.push(bench("registry", "search(\"mean\", 5)", 1, "call", || {
        black_box(registry::search(black_box("mean"), 5));
    }));
    rows.push(bench("registry", "search(\"norm\", 20)", 1, "call", || {
        black_box(registry::search(black_box("norm"), 20));
    }));

    // ---------------------------------------------------- dispatch parsing
    let step_obj = json!({"op": "math.add", "a": [1, 2]});
    let step_arr = json!(["math.add", 1, 2]);
    rows.push(bench("dispatch", "parse_step object form", 1, "call", || {
        black_box(user_ops::parse_step(black_box(&step_obj)).unwrap());
    }));
    rows.push(bench("dispatch", "parse_step array form", 1, "call", || {
        black_box(user_ops::parse_step(black_box(&step_arr)).unwrap());
    }));

    // --------------------------------------------------- execution only
    // Arguments are pre-built Values, so these exclude all JSON parsing.
    let a_add = vec![json!(1.0), json!(2.0)];
    rows.push(bench("execute", "math.add", 1, "call", || {
        black_box(engine::execute(black_box("math.add"), black_box(&a_add)).unwrap());
    }));

    for n in [100usize, 10_000] {
        let a = vec![json!(ramp(n))];
        rows.push(bench("execute", format!("stat.sum n={n}"), n as u64, "element", || {
            black_box(engine::execute(black_box("stat.sum"), black_box(&a)).unwrap());
        }));
        rows.push(bench("execute", format!("stat.cumsum n={n}"), n as u64, "element", || {
            black_box(engine::execute(black_box("stat.cumsum"), black_box(&a)).unwrap());
        }));
    }

    let m = json!(matrix(100));
    let a_mul = vec![m.clone(), m.clone()];
    rows.push(bench("execute", "mat.mul 100x100", 1_000_000, "MAC", || {
        black_box(engine::execute(black_box("mat.mul"), black_box(&a_mul)).unwrap());
    }));
    let a_shape = vec![m.clone()];
    rows.push(bench("execute", "mat.shape 100x100", 1, "call", || {
        black_box(engine::execute(black_box("mat.shape"), black_box(&a_shape)).unwrap());
    }));

    let sig = vec![json!((0..2048).map(|i| (i % 13) as f64).collect::<Vec<f64>>())];
    rows.push(bench("execute", "signal.fft n=2048", 2048, "sample", || {
        black_box(engine::execute(black_box("signal.fft"), black_box(&sig)).unwrap());
    }));

    let a_int = vec![json!({"e": "x*x"}), json!(0), json!(1), json!(10_000)];
    rows.push(bench("execute", "num.integrate n=10000", 10_001, "eval", || {
        black_box(engine::execute(black_box("num.integrate"), black_box(&a_int)).unwrap());
    }));
    let a_ode = vec![json!({"e": "y"}), json!(0), json!(1), json!(1), json!(10_000)];
    rows.push(bench("execute", "ode.rk4 n=10000", 40_000, "eval", || {
        black_box(engine::execute(black_box("ode.rk4"), black_box(&a_ode)).unwrap());
    }));

    // -------------------------------------------- expression evaluator
    // The dominant cost inside num./ode./optimize./series. is here.
    let mut vars = std::collections::HashMap::new();
    vars.insert("x".to_string(), 1.5_f64);
    rows.push(bench("formula", "eval \"x*x\"", 1, "eval", || {
        black_box(formula::eval(black_box("x*x"), black_box(&vars)).unwrap());
    }));
    rows.push(bench("formula", "eval \"(x+1)*(x-1)/2+x^2\"", 1, "eval", || {
        black_box(formula::eval(black_box("(x+1)*(x-1)/2+x^2"), black_box(&vars)).unwrap());
    }));
    // The per-evaluation map clone that num./ode./optimize. perform today.
    rows.push(bench("formula", "clone vars map + insert x (per-eval overhead)", 1, "op", || {
        let mut v = black_box(&vars).clone();
        v.insert("x".to_string(), black_box(2.0));
        black_box(v);
    }));

    // ------------------------------------------------- input / output path
    for n in [1000usize, 10_000, 50_000] {
        let text = serde_json::to_string(&ramp(n)).unwrap();
        rows.push(bench("input", format!("serde_json parse {n} floats"), n as u64, "element", || {
            black_box(serde_json::from_str::<Value>(black_box(&text)).unwrap());
        }));
        let value: Value = serde_json::from_str(&text).unwrap();
        rows.push(bench("output", format!("Value::to_string {n} floats"), n as u64, "element", || {
            black_box(black_box(&value).to_string());
        }));
        let arr = value.as_array().unwrap().clone();
        rows.push(bench("input", format!("Value -> Vec<f64> {n}"), n as u64, "element", || {
            let v: Vec<f64> = black_box(&arr).iter().map(|x| x.as_f64().unwrap()).collect();
            black_box(v);
        }));
    }

    let batch_text = serde_json::to_string(
        &(0..1000).map(|_| json!(["math.add", 1, 2])).collect::<Vec<_>>()).unwrap();
    rows.push(bench("input", "serde_json parse 1000-item batch", 1000, "item", || {
        black_box(serde_json::from_str::<Value>(black_box(&batch_text)).unwrap());
    }));

    canary(&mut rows, "stat.cumsum n=4096 (end)");

    // ------------------------------------------------------------ report
    let drift = {
        let first = rows.iter().find(|r| r.name.contains("(start)")).map(|r| r.ns_per_op);
        let last = rows.iter().find(|r| r.name.contains("(end)")).map(|r| r.ns_per_op);
        match (first, last) {
            (Some(a), Some(b)) if a > 0.0 => b / a,
            _ => 1.0,
        }
    };
    println!();
    println!("{:<10} {:<42} {:>14} {:>16} {:>7} {:>10}",
             "group", "case", "per call", "per unit", "mode", "iters");
    println!("{}", "-".repeat(105));
    let mut last = "";
    for r in &rows {
        if r.group != last {
            if !last.is_empty() {
                println!();
            }
            last = r.group;
        }
        let per_call = if r.ns_per_op >= 1_000_000.0 {
            format!("{:.3} ms", r.ns_per_op / 1e6)
        } else if r.ns_per_op >= 1000.0 {
            format!("{:.3} us", r.ns_per_op / 1e3)
        } else {
            format!("{:.1} ns", r.ns_per_op)
        };
        let per_unit = match r.per_unit_ns {
            Some(x) => format!("{x:.2} ns/{}", r.unit),
            None => "-".to_string(),
        };
        println!("{:<10} {:<42} {:>14} {:>16} {:>7} {:>10}",
                 r.group, r.name, per_call, per_unit, r.mode, r.iters);
    }
    println!("machine drift during this run (end canary / start canary): {:.2}x", drift);
    if (drift - 1.0).abs() > 0.10 {
        println!("WARNING: drift exceeds 10%. Absolute figures above are unreliable;");
        println!("         treat any change smaller than {:.0}% as unmeasured, and use",
                 (drift - 1.0).abs() * 100.0);
        println!("         bench/paired_ab.py for A/B decisions.");
    }
    println!();

    if let Some(path) = json_out {
        let items: Vec<Value> = rows.iter().map(|r| json!({
            "group": r.group, "case": r.name, "ns_per_op": r.ns_per_op,
            "iters": r.iters, "unit": r.unit, "per_unit_ns": r.per_unit_ns,
            "mode": r.mode,
        })).collect();
        let payload = json!({"benchmark": "yekaterina-micro", "drift_ratio": drift,
                             "rows": items});
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap() + "\n")
            .expect("write micro benchmark json");
        println!("wrote {path}");
    }
}
