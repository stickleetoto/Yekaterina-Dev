use serde_json::{Map, Value, json};

const MAX_METRICS: usize = 128;
const MAX_CONSTRAINTS: usize = 128;
const MAX_OBJECTIVES: usize = 64;
const MAX_CANDIDATES: usize = 256;

/// Structured transformer-candidate evaluation for optimization loops.
///
/// This module is intentionally not on the frozen v1.2 registry. It is a v1.3
/// candidate surface and is promoted only after the v1.3 registry/audit policy
/// is approved.
pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    match op {
        "xfmr.candidate_evaluate" => Some(candidate_evaluate(args)),
        "xfmr.candidate_rank" => Some(candidate_rank(args)),
        _ => None,
    }
}

fn candidate_evaluate(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let metrics = metrics(&args[0])?;
    let constraints = array_limited(&args[1], MAX_CONSTRAINTS)?;
    let objectives = array_limited(&args[2], MAX_OBJECTIVES)?;
    evaluate(&metrics, constraints, objectives)
}

fn candidate_rank(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 3)?;
    let candidates = array_limited(&args[0], MAX_CANDIDATES)?;
    if candidates.is_empty() {
        return Err("EMPTY");
    }
    let constraints = array_limited(&args[1], MAX_CONSTRAINTS)?;
    let objectives = array_limited(&args[2], MAX_OBJECTIVES)?;
    if objectives.is_empty() {
        return Err("EMPTY");
    }

    let mut evaluated = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.iter().enumerate() {
        let candidate_metrics = metrics(candidate)?;
        let report = evaluate(&candidate_metrics, constraints, objectives)?;
        let passed = report.get("pass").and_then(Value::as_bool).ok_or("TYPE")?;
        let score = report.get("score").and_then(Value::as_f64).ok_or("TYPE")?;
        evaluated.push((index, passed, score, report));
    }

    // Feasible candidates always rank above infeasible candidates. Within each
    // group, higher normalized desirability wins. Index is the deterministic
    // final tie-breaker.
    evaluated.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.total_cmp(&a.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    let ranking: Vec<Value> = evaluated
        .into_iter()
        .enumerate()
        .map(|(rank, (index, passed, score, report))| {
            json!({
                "rank": rank + 1,
                "index": index,
                "pass": passed,
                "score": score,
                "report": report
            })
        })
        .collect();

    Ok(json!({"ranking": ranking}))
}

fn evaluate(
    metrics: &Map<String, Value>,
    constraints: &[Value],
    objectives: &[Value],
) -> Result<Value, &'static str> {
    let mut constraint_results = Vec::with_capacity(constraints.len());
    let mut violations = Vec::new();

    for raw in constraints {
        let c = raw.as_object().ok_or("TYPE")?;
        let metric = required_str(c, "metric")?;
        let value = metric_value(metrics, metric)?;
        let min = optional_finite(c, "min")?;
        let max = optional_finite(c, "max")?;
        if min.is_none() && max.is_none() {
            return Err("ARG");
        }
        if let (Some(lo), Some(hi)) = (min, max) {
            if lo > hi {
                return Err("DOMAIN");
            }
        }

        let min_ok = min.is_none_or(|lo| value >= lo);
        let max_ok = max.is_none_or(|hi| value <= hi);
        let passed = min_ok && max_ok;
        if !passed {
            violations.push(json!({
                "metric": metric,
                "value": value,
                "min": min,
                "max": max
            }));
        }
        constraint_results.push(json!({
            "metric": metric,
            "value": value,
            "min": min,
            "max": max,
            "pass": passed
        }));
    }

    let mut objective_results = Vec::with_capacity(objectives.len());
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for raw in objectives {
        let o = raw.as_object().ok_or("TYPE")?;
        let metric = required_str(o, "metric")?;
        let kind = required_str(o, "kind")?;
        let value = metric_value(metrics, metric)?;
        let weight = optional_finite(o, "weight")?.unwrap_or(1.0);
        if weight <= 0.0 {
            return Err("DOMAIN");
        }

        let desirability = match kind {
            "minimize" => {
                let good = required_finite(o, "good")?;
                let bad = required_finite(o, "bad")?;
                if good >= bad {
                    return Err("DOMAIN");
                }
                clamp01((bad - value) / (bad - good))
            }
            "maximize" => {
                let good = required_finite(o, "good")?;
                let bad = required_finite(o, "bad")?;
                if good <= bad {
                    return Err("DOMAIN");
                }
                clamp01((value - bad) / (good - bad))
            }
            "target" => {
                let target = required_finite(o, "target")?;
                let tolerance = required_finite(o, "tolerance")?;
                if tolerance <= 0.0 {
                    return Err("DOMAIN");
                }
                clamp01(1.0 - (value - target).abs() / tolerance)
            }
            _ => return Err("ARG"),
        };

        weighted_sum += desirability * weight;
        total_weight += weight;
        objective_results.push(json!({
            "metric": metric,
            "kind": kind,
            "value": value,
            "weight": weight,
            "desirability": desirability
        }));
    }

    let score = if objectives.is_empty() {
        1.0
    } else {
        if !weighted_sum.is_finite() || !total_weight.is_finite() || total_weight <= 0.0 {
            return Err("NONFINITE");
        }
        weighted_sum / total_weight
    };

    Ok(json!({
        "pass": violations.is_empty(),
        "score": score,
        "violations": violations,
        "constraints": constraint_results,
        "objectives": objective_results
    }))
}

fn metrics(v: &Value) -> Result<Map<String, Value>, &'static str> {
    let obj = v.as_object().ok_or("TYPE")?;
    if obj.is_empty() {
        return Err("EMPTY");
    }
    if obj.len() > MAX_METRICS {
        return Err("LIMIT");
    }
    for value in obj.values() {
        let x = value.as_f64().ok_or("TYPE")?;
        if !x.is_finite() {
            return Err("NONFINITE");
        }
    }
    Ok(obj.clone())
}

fn metric_value(metrics: &Map<String, Value>, key: &str) -> Result<f64, &'static str> {
    metrics
        .get(key)
        .ok_or("ARG")?
        .as_f64()
        .ok_or("TYPE")
        .and_then(finite_number)
}

fn array_limited(v: &Value, limit: usize) -> Result<&[Value], &'static str> {
    let xs = v.as_array().ok_or("TYPE")?;
    if xs.len() > limit {
        return Err("LIMIT");
    }
    Ok(xs)
}

fn required_str<'a>(obj: &'a Map<String, Value>, key: &str) -> Result<&'a str, &'static str> {
    let s = obj.get(key).and_then(Value::as_str).ok_or("ARG")?;
    if s.is_empty() || s.len() > 128 {
        return Err("ARG");
    }
    Ok(s)
}

fn required_finite(obj: &Map<String, Value>, key: &str) -> Result<f64, &'static str> {
    obj.get(key)
        .and_then(Value::as_f64)
        .ok_or("ARG")
        .and_then(finite_number)
}

fn optional_finite(obj: &Map<String, Value>, key: &str) -> Result<Option<f64>, &'static str> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(v.as_f64().ok_or("TYPE").and_then(finite_number)?)),
    }
}

fn finite_number(x: f64) -> Result<f64, &'static str> {
    if x.is_finite() { Ok(x) } else { Err("NONFINITE") }
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn need(args: &[Value], n: usize) -> Result<(), &'static str> {
    if args.len() == n { Ok(()) } else { Err("ARG") }
}
