//! Batch execution planning.
//!
//! Decides which batch items may run concurrently and which must run in order on
//! the request task. Produces a plan only; `server.rs` executes it.
//!
//! # Batch is not an independent workload
//!
//! A batch item may reference an earlier item's result:
//!
//! ```text
//! [["math.add",1,2],["math.mul","$0",10]]  ->  {"r":[3.0,30.0]}
//! ```
//!
//! and the reference graph is strictly backward-pointing, because a forward
//! reference resolves against a `results` vector that does not yet contain it:
//!
//! ```text
//! [["math.mul","$1",10],["math.add",1,2]]  ->  {"r":[{"e":"REF"},3.0]}
//! ```
//!
//! Errors are values, not aborts, and a later item may reference one:
//!
//! ```text
//! [["math.div",1,0],["math.mul","$0",10]]  ->  {"r":[{"e":"DIV0"},{"e":"TYPE"}]}
//! ```
//!
//! All three were measured against v1.0.0 before any of this was written, and
//! all three are pinned by tests.
//!
//! # What may move
//!
//! An item is [`Placement::Concurrent`] only when all of these hold:
//!
//! * it parses,
//! * it references no earlier result (`$input` is fine -- it comes from the
//!   request, not from `results`),
//! * its opcode resolves to a built-in classified [`crate::safety::Safety::Pure`].
//!
//! Anything else is [`Placement::Ordered`]: dependent items, user operations,
//! composites, and the `udo.*` control operations, which act as barriers.
//!
//! Ordering is never recovered by sorting. Results are written into slots keyed
//! by input index, so completion order is structurally unable to leak out.

use serde_json::Value;

use crate::{registry, safety, user_ops};

/// Where one batch item may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// May execute on a worker, concurrently with other `Concurrent` items in
    /// the same run.
    Concurrent,
    /// Must execute on the request task, in index order. Acts as a barrier.
    Ordered,
}

/// Whether a value tree references an earlier *result*.
///
/// Only `$<digits>` does. `$input` reads the request's input and is independent
/// of other items, and any other `$`-prefixed string is a `REF` error that also
/// does not depend on results. Keeping `$input` out of this predicate is what
/// lets a batch of `["stat.sum", "$input"]` items parallelise.
fn references_results(v: &Value) -> bool {
    match v {
        Value::String(s) => {
            s.len() > 1 && s.starts_with('$') && s[1..].bytes().all(|b| b.is_ascii_digit())
        }
        Value::Array(xs) => xs.iter().any(references_results),
        Value::Object(obj) => obj.values().any(references_results),
        _ => false,
    }
}

/// Classify every item in a batch.
///
/// Cheap and allocation-light: parsing borrows out of the request, and the only
/// work per item is a registry lookup plus a walk of the argument tree that the
/// request-size check already performs.
pub fn plan_batch(items: &[Value]) -> Vec<Placement> {
    items.iter().map(placement_of).collect()
}

fn placement_of(item: &Value) -> Placement {
    // A malformed item never executes; its result is a parse error. Treat it as
    // ordered so error production stays on the single, obvious path.
    let Ok((opcode, args)) = user_ops::parse_step_ref(item) else {
        return Placement::Ordered;
    };
    if args.iter().any(references_results) {
        return Placement::Ordered;
    }
    // Must be a built-in: user formulas and composites need the registry
    // snapshot and recurse through the dispatcher.
    match registry::resolve(&opcode) {
        Some(spec) if safety::classify(spec.opcode) == safety::Safety::Pure => {
            Placement::Concurrent
        }
        _ => Placement::Ordered,
    }
}

/// Estimated work in one item, split into the part parallelism can remove and
/// the part it cannot.
///
/// A wrong estimate can only pick a worse schedule; it can never change a
/// result. That is why this is allowed to use `registry::cost_code`, which
/// classifies by opcode prefix -- a heuristic explicitly forbidden in
/// `safety.rs`, where a wrong answer would be a race rather than a slower run.
///
/// # Why one number was not enough
///
/// The first model scored an item by `cost_class * argument_size`. Measured
/// against the 1/2/4/8 worker sweep it was **ordered wrongly**: the only
/// workload that actually scaled, a batch of `signal.dft` over 256 samples,
/// scored *below* two batches that gained nothing.
///
/// | workload | old score | measured at 4 workers |
/// |---|---:|---|
/// | 16 x mixed `signal.dft(256)` | 131,592 | **0.40x** |
/// | 8 x `signal.fft(512)` | 262,656 | 1.04x |
/// | 4 x `stat.sum(10000)` | 320,032 | 1.07x |
///
/// No threshold can separate those. The reason is that argument size measures
/// two different things at once. `stat.sum` over 10,000 numbers is O(n) trivial
/// arithmetic whose request time is dominated by *parsing* those numbers --
/// serial work that happens before any wave and that parallelism cannot touch.
/// `signal.dft` over 256 samples is O(n^2) trigonometry on a small payload.
///
/// So the model tracks both:
///
/// * [`Cost::compute`] -- algorithmic work, what a worker removes from the
///   critical path;
/// * [`Cost::payload`] -- argument and result volume, which is parsed and
///   serialized serially no matter how many workers exist.
///
/// A run is worth distributing only when compute is both absolutely large and
/// large *relative to* payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cost {
    pub compute: u64,
    pub payload: u64,
}

impl Cost {
    fn saturating_add(self, other: Cost) -> Cost {
        Cost {
            compute: self.compute.saturating_add(other.compute),
            payload: self.payload.saturating_add(other.payload),
        }
    }
}

/// Operations whose cost class is "heavy" but whose complexity is n log n, not
/// n^2. Without this the transforms dominate every estimate and crowd out the
/// genuinely quadratic operations that do benefit.
fn is_log_linear(opcode: &str) -> bool {
    matches!(opcode, "signal.fft" | "signal.ifft" | "signal.rfft")
}

/// Families whose work is driven by an iteration count passed as a scalar
/// argument rather than by the size of a container. `num.integrate` with
/// `n = 1000` carries a tiny payload and does a thousand evaluations.
fn iteration_driven(opcode: &str) -> bool {
    opcode.starts_with("num.")
        || opcode.starts_with("ode.")
        || opcode.starts_with("optimize.")
        || opcode.starts_with("series.")
}

const MAGNITUDE_CAP: u64 = 100_000;
const COMPUTE_CAP: u64 = 1_000_000_000;

pub fn estimated_cost(opcode: &str, args: &[Value]) -> Cost {
    let payload = magnitude(args).max(1);
    let compute = match registry::cost_code(opcode) {
        "1" => 1,
        "n" => payload,
        "n3" => payload
            .saturating_mul(payload)
            .saturating_mul(payload)
            .min(COMPUTE_CAP),
        "h" => {
            if is_log_linear(opcode) {
                payload.saturating_mul(payload.max(2).ilog2() as u64)
            } else if iteration_driven(opcode) {
                max_scalar(args).max(1)
            } else {
                payload.saturating_mul(payload).min(COMPUTE_CAP)
            }
        }
        // Control operations never reach a wave.
        _ => 1,
    };
    Cost { compute: compute.min(COMPUTE_CAP), payload }
}

/// Largest non-negative integral scalar anywhere in the arguments, used as the
/// iteration count for solver families.
fn max_scalar(args: &[Value]) -> u64 {
    fn walk(v: &Value, best: &mut u64) {
        match v {
            Value::Number(n) => {
                if let Some(x) = n.as_u64() {
                    *best = (*best).max(x);
                }
            }
            Value::Array(xs) => xs.iter().for_each(|x| walk(x, best)),
            Value::Object(obj) => obj.values().for_each(|x| walk(x, best)),
            _ => {}
        }
    }
    let mut best = 0u64;
    for a in args {
        walk(a, &mut best);
    }
    best.min(MAGNITUDE_CAP)
}

/// Container element count across the argument tree, bounded so a pathological
/// payload cannot make planning expensive.
fn magnitude(args: &[Value]) -> u64 {
    fn walk(v: &Value, budget: &mut u64) {
        if *budget == 0 {
            return;
        }
        match v {
            Value::Array(xs) => {
                for x in xs {
                    *budget = budget.saturating_sub(1);
                    walk(x, budget);
                }
            }
            Value::Object(obj) => {
                for x in obj.values() {
                    *budget = budget.saturating_sub(1);
                    walk(x, budget);
                }
            }
            _ => {}
        }
    }
    let mut budget = MAGNITUDE_CAP;
    for a in args {
        walk(a, &mut budget);
    }
    MAGNITUDE_CAP - budget
}

/// Total estimated cost of a run of items.
pub fn run_cost(items: &[Value]) -> Cost {
    items
        .iter()
        .map(|item| match user_ops::parse_step_ref(item) {
            Ok((opcode, args)) => match registry::resolve(&opcode) {
                Some(spec) => estimated_cost(spec.opcode, args),
                None => Cost { compute: 1, payload: 1 },
            },
            Err(_) => Cost { compute: 1, payload: 1 },
        })
        .fold(Cost::default(), Cost::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn independent_pure_items_are_concurrent() {
        let items = vec![
            json!(["math.add", 1, 2]),
            json!({"op": "stat.mean", "a": [[1, 2, 3]]}),
            json!(["signal.fft", [1, 0, 0, 0]]),
            json!(["expr.eval", {"e": "1+1"}]),
        ];
        assert_eq!(plan_batch(&items), vec![Placement::Concurrent; 4]);
    }

    #[test]
    fn result_references_force_ordering() {
        let items = vec![
            json!(["math.add", 1, 2]),
            json!(["math.mul", "$0", 10]),
            json!(["math.add", 1, 2]),
        ];
        assert_eq!(
            plan_batch(&items),
            vec![Placement::Concurrent, Placement::Ordered, Placement::Concurrent]
        );
    }

    /// `$input` comes from the request, not from `results`, so it does not
    /// serialize anything.
    #[test]
    fn input_references_stay_concurrent() {
        let items = vec![json!(["stat.sum", "$input"]), json!(["stat.mean", "$input"])];
        assert_eq!(plan_batch(&items), vec![Placement::Concurrent; 2]);
    }

    #[test]
    fn nested_result_references_are_found() {
        for arg in [
            json!(["math.add", [1, "$0"], 2]),
            json!(["math.add", {"k": "$3"}, 2]),
            json!(["math.add", [[["$12"]]], 2]),
            json!({"op": "math.add", "a": [{"deep": ["$0"]}]}),
        ] {
            assert_eq!(plan_batch(std::slice::from_ref(&arg)), vec![Placement::Ordered], "{arg}");
        }
    }

    /// A `$`-prefixed string that is not a reference is a REF error either way,
    /// and does not depend on results.
    #[test]
    fn invalid_dollar_strings_do_not_reference_results() {
        assert!(!references_results(&json!("$nope")));
        assert!(!references_results(&json!("$input")));
        assert!(!references_results(&json!("$")));
        assert!(references_results(&json!("$0")));
        assert!(references_results(&json!("$999")));
    }

    #[test]
    fn control_and_dynamic_operations_are_barriers() {
        let items = vec![
            json!(["udo.formula", {"op": "user.x", "p": ["x"], "expr": "x*2"}]),
            json!(["udo.list"]),
            json!(["udo.export", {"name": "p"}]),
            json!(["user.x", 1]),
            json!(["pack.demo.thing", 1]),
            json!(["zzz.nope"]),
        ];
        assert_eq!(plan_batch(&items), vec![Placement::Ordered; 6]);
    }

    #[test]
    fn malformed_items_are_ordered() {
        let items = vec![json!({"nope": 1}), json!([]), json!("scalar"), json!(42)];
        assert_eq!(plan_batch(&items), vec![Placement::Ordered; 4]);
    }

    #[test]
    fn aliases_resolve_to_their_canonical_placement() {
        assert_eq!(plan_batch(&[json!(["avg", [1, 2]])]), vec![Placement::Concurrent]);
        assert_eq!(plan_batch(&[json!(["Math.Add", 1, 2])]), vec![Placement::Concurrent]);
    }

    /// The ordering the 1/2/4/8 worker sweep revealed. These three ran at
    /// 0.40x, 1.04x and 1.07x respectively at four workers, so any model that
    /// does not rank them in that order cannot drive the adaptive decision.
    #[test]
    fn cost_model_ranks_by_measured_benefit() {
        let dft: Vec<Value> = (0..16)
            .map(|i| {
                if i % 2 == 0 {
                    json!(["signal.dft", (0..256).map(|_| json!(0)).collect::<Vec<_>>()])
                } else {
                    json!(["math.add", 1, 2])
                }
            })
            .collect();
        let fft: Vec<Value> = (0..8)
            .map(|_| json!(["signal.fft", (0..512).map(|_| json!(0)).collect::<Vec<_>>()]))
            .collect();
        let sum: Vec<Value> = (0..4)
            .map(|_| json!(["stat.sum", (0..10000).map(|_| json!(0)).collect::<Vec<_>>()]))
            .collect();

        let (a, b, c) = (run_cost(&dft), run_cost(&fft), run_cost(&sum));
        assert!(
            a.compute > b.compute && a.compute > c.compute,
            "the workload that scales must rank highest: dft={a:?} fft={b:?} sum={c:?}"
        );
        // And the two that do not benefit must be payload-heavy relative to
        // their compute, which is what keeps them off the parallel path.
        assert!(a.compute > a.payload * 20, "dft should be compute-bound: {a:?}");
        assert!(c.compute <= c.payload * 20, "stat.sum should be payload-bound: {c:?}");
    }

    /// Iteration counts arrive as scalar arguments, not as container size.
    #[test]
    fn iteration_driven_operations_are_scored_by_their_step_count() {
        let cheap = estimated_cost("num.integrate", &[json!({"e": "x*x"}), json!(0), json!(1), json!(10)]);
        let heavy = estimated_cost("num.integrate", &[json!({"e": "x*x"}), json!(0), json!(1), json!(10000)]);
        assert!(heavy.compute > cheap.compute * 100, "cheap={cheap:?} heavy={heavy:?}");
        assert!(heavy.compute > heavy.payload * 20, "should be compute-bound: {heavy:?}");
    }

    /// A transform that is n log n must not be scored as if it were n^2.
    #[test]
    fn log_linear_transforms_are_not_scored_as_quadratic() {
        let n = 512;
        let arg = vec![json!((0..n).map(|_| json!(0)).collect::<Vec<_>>())];
        let fft = estimated_cost("signal.fft", &arg);
        let dft = estimated_cost("signal.dft", &arg);
        assert!(dft.compute > fft.compute * 20, "fft={fft:?} dft={dft:?}");
    }

    #[test]
    fn trivial_work_stays_trivial() {
        let cheap: Vec<Value> = (0..32).map(|i| json!(["math.add", i, 1])).collect();
        let c = run_cost(&cheap);
        assert!(c.compute < 1000, "{c:?}");
    }
}
