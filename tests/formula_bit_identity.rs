//! Bit-identity regression corpus for the expression evaluator.
//!
//! Phase 2C optimizes `formula::eval`, which sits inside every iteration of the
//! `num.`, `ode.`, `optimize.` and `series.` solvers. The acceptance condition
//! for that work is not "close enough" but **bit-identical `f64` output**:
//! a changed rounding or reassociation would silently move numerical results
//! that Golden pins only to a tolerance.
//!
//! The fixture in `tests/fixtures/formula_bits.json` was generated from the
//! pre-Phase-2C evaluator and records the exact `f64` bit pattern (and exact
//! error code) for every case. It is an oracle, not an expectation to be
//! updated: if a change makes this test fail, the change altered numerical
//! behaviour and must be rejected, not re-baselined.
//!
//! Regenerate deliberately, and only when you intend to redefine the oracle:
//!
//! ```text
//! YK_REGEN_FORMULA_FIXTURE=1 cargo test --test formula_bit_identity
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{Value, json};
use yekaterina::formula;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/formula_bits.json")
}

/// Expressions chosen to pin operator precedence, associativity, the exact
/// points at which `DIV0` and `VAR` are raised, whitespace handling, numeric
/// literal parsing, and the depth limit.
fn expressions() -> Vec<&'static str> {
    vec![
        // arithmetic and precedence
        "x", "x+1", "1+x", "x*x", "x*x*x", "x+x*x", "(x+1)*(x-1)",
        "x-1-1", "x-(1-1)", "1-2-3-x", "x/2/2", "x/(2/2)",
        "x*y", "x*y+y*x", "x+y*x-y/2",
        // associativity of ^ is right, and it binds tighter than unary minus
        "x^2", "x^2^3", "2^x", "-x^2", "(-x)^2", "x^-1", "x^0.5",
        // unary chains
        "-x", "--x", "+-x", "-+x", "---x", "-(x+y)",
        // modulo and division, including the exact DIV0 trigger points
        "x%3", "10%x", "x/y", "10/x", "1/0+x", "x+1/0", "0/x", "x%0", "1%0+x",
        // parentheses and whitespace
        "  x  +  1  ", "((x))", "(((x+1)))", "( x * ( y + 1 ) )",
        // numeric literals
        "1e3*x", "1E3*x", "1.5e-3*x", "0.1+x", ".5+x", "1.+x", "1e0*x",
        "3.141592653589793*x", "0.30000000000000004+x",
        // accumulation order effects
        "0.1+0.2+x", "x+0.1+0.2", "(0.1+0.2)+x", "0.1+(0.2+x)",
        "1e308*x", "1e-308*x", "1e308+1e308+x",
        // variable resolution and its precedence against other errors
        "unknown", "x+unknown", "unknown+x", "1/0+unknown", "unknown+1/0",
        "x*unknown", "_u", "x_1",
        // malformed input
        "", "   ", "x+", "+", "(", ")", "()", "x)", "(x", "x y", "1 2", "*x",
        "x**y", "x^", "$", "x$",
    ]
}

/// Variable bindings, including edge values that expose rounding differences.
fn bindings() -> Vec<(&'static str, HashMap<String, f64>)> {
    let mk = |x: f64, y: f64| {
        let mut m = HashMap::new();
        m.insert("x".to_string(), x);
        m.insert("y".to_string(), y);
        m.insert("x_1".to_string(), x + 1.0);
        m.insert("_u".to_string(), x - 1.0);
        m
    };
    vec![
        ("zero", mk(0.0, 0.0)),
        ("neg_zero", mk(-0.0, -0.0)),
        ("one", mk(1.0, 1.0)),
        ("small", mk(1e-7, 3.0)),
        ("third", mk(1.0 / 3.0, 7.0 / 11.0)),
        ("negative", mk(-2.5, -0.75)),
        ("large", mk(1e15, 1e-15)),
        ("huge", mk(1e300, 1e-300)),
        ("subnormal", mk(5e-324, 2.2250738585072014e-308)),
        ("irrational", mk(std::f64::consts::PI, std::f64::consts::E)),
        ("empty", HashMap::new()),
    ]
}

/// A deeply nested expression that sits either side of the parser depth limit.
fn depth_cases() -> Vec<String> {
    vec![
        format!("{}x{}", "(".repeat(10), ")".repeat(10)),
        format!("{}x{}", "(".repeat(30), ")".repeat(30)),
        format!("{}x{}", "(".repeat(80), ")".repeat(80)),
        "-".repeat(70) + "x",
    ]
}

/// Bit pattern for a result, or the exact error code. `to_bits` is used rather
/// than a float comparison so that a one-ulp change, a sign-of-zero change, or a
/// different NaN payload all fail loudly.
fn outcome(expr: &str, vars: &HashMap<String, f64>) -> Value {
    match formula::eval(expr, vars) {
        Ok(v) => json!({"bits": v.to_bits().to_string(), "repr": format!("{v:?}")}),
        Err(e) => json!({"err": e}),
    }
}

fn build_corpus() -> Vec<Value> {
    let mut out = Vec::new();
    let binds = bindings();
    let mut exprs: Vec<String> = expressions().into_iter().map(String::from).collect();
    exprs.extend(depth_cases());
    for expr in &exprs {
        for (bind_name, vars) in &binds {
            out.push(json!({
                "expr": expr,
                "bind": bind_name,
                "out": outcome(expr, vars),
            }));
        }
    }
    out
}

#[test]
fn evaluator_output_is_bit_identical_to_the_frozen_corpus() {
    let corpus = build_corpus();
    let path = fixture_path();

    if std::env::var("YK_REGEN_FORMULA_FIXTURE").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("create fixture dir");
        let payload = json!({
            "schema": "yekaterina.formula-bits/v1",
            "note": "Oracle generated from the pre-Phase-2C evaluator. \
                     A failure means numerical behaviour changed; do not re-baseline \
                     to make it pass.",
            "cases": corpus,
        });
        std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap() + "\n")
            .expect("write fixture");
        eprintln!("regenerated {} cases -> {}", corpus.len(), path.display());
        return;
    }

    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing fixture {}: {e}. Generate it with \
             YK_REGEN_FORMULA_FIXTURE=1 cargo test --test formula_bit_identity",
            path.display()
        )
    });
    let stored: Value = serde_json::from_str(&raw).expect("parse fixture");
    let expected = stored["cases"].as_array().expect("cases array");

    assert_eq!(
        expected.len(),
        corpus.len(),
        "corpus size changed: fixture has {} cases, this build produced {}",
        expected.len(),
        corpus.len()
    );

    let mut failures = Vec::new();
    for (want, got) in expected.iter().zip(corpus.iter()) {
        assert_eq!(want["expr"], got["expr"], "corpus order changed");
        assert_eq!(want["bind"], got["bind"], "corpus order changed");
        if want["out"] != got["out"] {
            failures.push(format!(
                "expr {:?} with bindings {:?}: expected {}, got {}",
                got["expr"].as_str().unwrap_or(""),
                got["bind"].as_str().unwrap_or(""),
                want["out"],
                got["out"]
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "evaluator output changed in {} of {} cases:\n{}",
        failures.len(),
        corpus.len(),
        failures.join("\n")
    );
}
