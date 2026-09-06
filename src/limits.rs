//! Request and result size accounting.
//!
//! Extracted from `server.rs` in v1.1 Phase 2A. The limits themselves, and the
//! exact traversal order and saturation behaviour of [`measure_value`], are
//! frozen v1.0.0 behaviour and must not change: they determine both whether a
//! request is rejected and, for batches and pipelines, *which item index* first
//! crosses the limit.
//!
//! # The v1.0.0 cost problem
//!
//! `run_batch`, `run_pipeline` and the composite executor each re-walked the
//! entire accumulated result vector after appending every item. That is a fold
//! recomputed from scratch on each step, so validation cost grew quadratically
//! in the number of results. Measured on the frozen baseline
//! (`bench_results/v1.0.0-frozen`), a batch of `stat.cumsum` over 100 floats:
//!
//! | items | wall  | per item |
//! |------:|------:|---------:|
//! |    10 |  0.38 ms |  39.9 us |
//! |   100 |  4.46 ms |  44.6 us |
//! |   400 | 29.28 ms |  72.9 us |
//!
//! 40x the items cost 77x the time, against roughly 2 us of actual compute per
//! item.
//!
//! # Why the incremental form is exactly equivalent
//!
//! [`measure_value`] is a left fold over `(nodes, strings)` with early exit. The
//! v1.0.0 check starts that fold at `nodes = 1, strings = 0` and replays it over
//! `results[0..k]` on every step `k`. [`ResultBudget`] seeds the same initial
//! state once and folds each newly appended value in exactly once, so after `k`
//! appends its accumulator holds precisely the state a full replay would
//! produce. The first step at which it reports "too large" is therefore the same
//! step, with the same accumulator, as the replay would report.
//!
//! [`values_too_large_replay`] preserves the original algorithm verbatim. It is
//! not used on the hot path; it is the oracle the equivalence tests check
//! [`ResultBudget`] against.

use serde_json::Value;

/// Maximum node count accepted in an inbound request payload.
pub const MAX_VALUE_NODES: usize = 200_000;
/// Maximum cumulative string/key bytes accepted in an inbound request payload.
pub const MAX_STRING_BYTES: usize = 1_000_000;
/// Maximum node count accepted in an outbound result.
pub const MAX_RESULT_NODES: usize = 100_000;
/// Maximum cumulative string/key bytes accepted in an outbound result.
pub const MAX_RESULT_BYTES: usize = 1_000_000;

/// Fold one value into `(nodes, strings)`, returning `false` on the first
/// violation.
///
/// Frozen v1.0.0 behaviour, moved verbatim:
///
/// * a node is counted before it is tested, so the limit is "more than
///   `max_nodes` nodes", not "at least";
/// * object keys contribute their byte length to `strings`, and a key is
///   charged before its value is walked;
/// * `strings` saturates rather than overflowing;
/// * arrays and objects short-circuit through `all`, leaving the accumulator in
///   the partially-updated state reached at the point of failure.
pub fn measure_value(
    v: &Value,
    nodes: &mut usize,
    strings: &mut usize,
    max_nodes: usize,
    max_strings: usize,
) -> bool {
    *nodes += 1;
    if *nodes > max_nodes {
        return false;
    }
    match v {
        Value::String(s) => {
            *strings = strings.saturating_add(s.len());
            *strings <= max_strings
        }
        Value::Array(xs) => xs
            .iter()
            .all(|x| measure_value(x, nodes, strings, max_nodes, max_strings)),
        Value::Object(obj) => obj.iter().all(|(k, x)| {
            *strings = strings.saturating_add(k.len());
            *strings <= max_strings && measure_value(x, nodes, strings, max_nodes, max_strings)
        }),
        _ => true,
    }
}

/// Whether one standalone result value exceeds the outbound limits.
///
/// Starts from `nodes = 0`, matching v1.0.0's `output_value_too_large`.
pub fn value_too_large(v: &Value) -> bool {
    let mut nodes = 0usize;
    let mut strings = 0usize;
    !measure_value(v, &mut nodes, &mut strings, MAX_RESULT_NODES, MAX_RESULT_BYTES)
}

/// Running total for a growing result vector.
///
/// Replaces the per-append full rescan. Seed once before the loop, then call
/// [`ResultBudget::admit`] with each value as it is appended, in append order.
#[derive(Debug, Clone)]
pub struct ResultBudget {
    nodes: usize,
    strings: usize,
}

impl Default for ResultBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultBudget {
    /// Seeded to `nodes = 1`, matching v1.0.0's `output_values_too_large`, which
    /// charged one node for the enclosing array on every call.
    pub fn new() -> Self {
        Self { nodes: 1, strings: 0 }
    }

    /// Fold one appended value in. Returns `false` the first time the
    /// accumulated result vector exceeds the outbound limits.
    ///
    /// Must be called exactly once per appended value, with the value actually
    /// appended. In `run_batch` an oversized item is replaced by an
    /// `{"e":"OUT_LIMIT"}` marker before being pushed, and it is that marker,
    /// not the original value, that is charged.
    pub fn admit(&mut self, v: &Value) -> bool {
        measure_value(
            v,
            &mut self.nodes,
            &mut self.strings,
            MAX_RESULT_NODES,
            MAX_RESULT_BYTES,
        )
    }

    /// Accumulated `(nodes, strings)`. Test and diagnostic use only.
    pub fn state(&self) -> (usize, usize) {
        (self.nodes, self.strings)
    }
}

/// The v1.0.0 whole-vector rescan, preserved verbatim as a test oracle.
///
/// Not used on the hot path. Kept so the equivalence tests compare against the
/// original algorithm rather than against a restatement of it.
pub fn values_too_large_replay(values: &[Value]) -> bool {
    let mut nodes = 1usize;
    let mut strings = 0usize;
    values.iter().any(|v| {
        !measure_value(
            v,
            &mut nodes,
            &mut strings,
            MAX_RESULT_NODES,
            MAX_RESULT_BYTES,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Deterministic value generator. No `rand` dependency: the crate's
    /// dependency set is exact-pinned and audited, and a seeded LCG makes any
    /// failure reproducible from its seed alone.
    struct Gen(u64);

    impl Gen {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }

        fn value(&mut self, depth: usize) -> Value {
            match self.next() % if depth >= 3 { 5 } else { 7 } {
                0 => Value::Null,
                1 => json!(self.next() % 1000),
                2 => json!((self.next() % 1000) as f64 / 7.0),
                3 => json!(self.next().is_multiple_of(2)),
                4 => Value::String("s".repeat((self.next() % 40) as usize)),
                5 => {
                    let n = (self.next() % 6) as usize;
                    Value::Array((0..n).map(|_| self.value(depth + 1)).collect())
                }
                _ => {
                    let n = (self.next() % 5) as usize;
                    let mut m = serde_json::Map::new();
                    for i in 0..n {
                        m.insert(format!("k{i}"), self.value(depth + 1));
                    }
                    Value::Object(m)
                }
            }
        }
    }

    /// The core Phase 2A guarantee: for every prefix of every sequence, the
    /// incremental budget agrees with the v1.0.0 replay, and the first step at
    /// which they report "too large" is the same step.
    #[test]
    fn incremental_budget_matches_replay_step_for_step() {
        for seed in 0..400u64 {
            let mut g = Gen(seed.wrapping_mul(0x9E3779B97F4A7C15));
            let values: Vec<Value> = (0..24).map(|_| g.value(0)).collect();

            let mut budget = ResultBudget::new();
            let mut pushed: Vec<Value> = Vec::new();
            for (i, v) in values.iter().enumerate() {
                pushed.push(v.clone());
                let incremental_ok = budget.admit(v);
                let replay_too_large = values_too_large_replay(&pushed);
                assert_eq!(
                    !incremental_ok, replay_too_large,
                    "seed {seed} step {i}: incremental and replay disagree"
                );
                if replay_too_large {
                    break;
                }
            }
        }
    }

    /// The accumulator must match the replay's internal state exactly, not just
    /// its boolean verdict, or a later step could diverge.
    #[test]
    fn accumulator_state_matches_replay_state() {
        for seed in 0..200u64 {
            let mut g = Gen(seed.wrapping_add(1) * 2654435761);
            let values: Vec<Value> = (0..12).map(|_| g.value(0)).collect();

            let mut budget = ResultBudget::new();
            for (i, v) in values.iter().enumerate() {
                if !budget.admit(v) {
                    break;
                }
                let mut nodes = 1usize;
                let mut strings = 0usize;
                for w in &values[..=i] {
                    assert!(measure_value(
                        w, &mut nodes, &mut strings, MAX_RESULT_NODES, MAX_RESULT_BYTES
                    ));
                }
                assert_eq!(
                    budget.state(),
                    (nodes, strings),
                    "seed {seed} step {i}: accumulator diverged from replay"
                );
            }
        }
    }

    #[test]
    fn node_limit_crossing_index_is_preserved() {
        // Each item is an array of 1000 numbers => 1001 nodes. With the
        // enclosing array seeded at 1 node, item k pushes the total to
        // 1 + 1001*(k+1); the limit is crossed during item 99.
        let item = Value::Array((0..1000).map(|i| json!(i)).collect());
        let mut budget = ResultBudget::new();
        let mut pushed = Vec::new();
        let mut incremental_fail = None;
        let mut replay_fail = None;
        for i in 0..200 {
            pushed.push(item.clone());
            if incremental_fail.is_none() && !budget.admit(&item) {
                incremental_fail = Some(i);
            }
            if replay_fail.is_none() && values_too_large_replay(&pushed) {
                replay_fail = Some(i);
            }
            if incremental_fail.is_some() && replay_fail.is_some() {
                break;
            }
        }
        assert_eq!(incremental_fail, replay_fail);
        assert_eq!(incremental_fail, Some(99));
    }

    #[test]
    fn string_limit_crossing_index_is_preserved() {
        let item = Value::String("x".repeat(100_000));
        let mut budget = ResultBudget::new();
        let mut pushed = Vec::new();
        let mut incremental_fail = None;
        let mut replay_fail = None;
        for i in 0..40 {
            pushed.push(item.clone());
            if incremental_fail.is_none() && !budget.admit(&item) {
                incremental_fail = Some(i);
            }
            if replay_fail.is_none() && values_too_large_replay(&pushed) {
                replay_fail = Some(i);
            }
            if incremental_fail.is_some() && replay_fail.is_some() {
                break;
            }
        }
        assert_eq!(incremental_fail, replay_fail);
        // 10 items of 100_000 bytes each reach exactly the 1_000_000 limit and
        // are accepted; the eleventh (index 10) crosses it.
        assert_eq!(incremental_fail, Some(10));
    }

    #[test]
    fn object_keys_are_charged_before_their_values() {
        let mut nodes = 0usize;
        let mut strings = 0usize;
        let v = json!({"abc": "de"});
        assert!(measure_value(&v, &mut nodes, &mut strings, 100, 100));
        // one object node plus one string node; "abc" (3) + "de" (2).
        assert_eq!((nodes, strings), (2, 5));
    }

    #[test]
    fn empty_vector_matches_v1_seed() {
        let budget = ResultBudget::new();
        assert_eq!(budget.state(), (1, 0));
        assert!(!values_too_large_replay(&[]));
    }

    #[test]
    fn single_value_check_starts_from_zero_nodes() {
        // value_too_large must not charge the enclosing-array node.
        let v = Value::Array((0..MAX_RESULT_NODES - 1).map(|i| json!(i)).collect());
        assert!(!value_too_large(&v));
        let v2 = Value::Array((0..MAX_RESULT_NODES).map(|i| json!(i)).collect());
        assert!(value_too_large(&v2));
    }
}
