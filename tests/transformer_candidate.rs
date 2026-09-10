#[path = "../src/transformer_candidate.rs"]
mod transformer_candidate;

use serde_json::{Value, json};

fn run(op: &str, args: Vec<Value>) -> Value {
    transformer_candidate::execute(op, &args)
        .expect("candidate operation must be claimed")
        .expect("candidate operation must succeed")
}

fn constraints() -> Value {
    json!([
        {"metric":"bmax_t","max":1.65},
        {"metric":"window_fill","max":0.50},
        {"metric":"winding_c","max":105.0},
        {"metric":"impedance_pct","min":5.0,"max":6.5}
    ])
}

fn objectives() -> Value {
    json!([
        {"metric":"total_loss_w","kind":"minimize","good":900.0,"bad":1500.0,"weight":0.6},
        {"metric":"mass_kg","kind":"minimize","good":350.0,"bad":550.0,"weight":0.25},
        {"metric":"impedance_pct","kind":"target","target":5.75,"tolerance":0.75,"weight":0.15}
    ])
}

#[test]
fn feasible_candidate_reports_pass_and_bounded_score() {
    let metrics = json!({
        "bmax_t":1.55,
        "window_fill":0.43,
        "winding_c":92.0,
        "impedance_pct":5.8,
        "total_loss_w":1080.0,
        "mass_kg":410.0
    });
    let out = run("xfmr.candidate_evaluate", vec![metrics, constraints(), objectives()]);
    assert_eq!(out["pass"], json!(true));
    assert_eq!(out["violations"].as_array().unwrap().len(), 0);
    let score = out["score"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&score));
    assert!(score > 0.60);
}

#[test]
fn constraint_failures_are_explicit() {
    let metrics = json!({
        "bmax_t":1.72,
        "window_fill":0.51,
        "winding_c":110.0,
        "impedance_pct":4.7,
        "total_loss_w":1300.0,
        "mass_kg":470.0
    });
    let out = run("xfmr.candidate_evaluate", vec![metrics, constraints(), objectives()]);
    assert_eq!(out["pass"], json!(false));
    let violations = out["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 4);
    assert_eq!(violations[0]["metric"], json!("bmax_t"));
    assert_eq!(violations[3]["metric"], json!("impedance_pct"));
}

#[test]
fn desirability_endpoints_are_interpretable() {
    let c = json!([]);
    let o = json!([
        {"metric":"loss","kind":"minimize","good":100.0,"bad":200.0},
        {"metric":"margin","kind":"maximize","good":20.0,"bad":5.0},
        {"metric":"z","kind":"target","target":6.0,"tolerance":1.0}
    ]);
    let perfect = run(
        "xfmr.candidate_evaluate",
        vec![json!({"loss":90.0,"margin":25.0,"z":6.0}), c.clone(), o.clone()],
    );
    assert!((perfect["score"].as_f64().unwrap() - 1.0).abs() < 1e-12);

    let poor = run(
        "xfmr.candidate_evaluate",
        vec![json!({"loss":220.0,"margin":2.0,"z":7.5}), c, o],
    );
    assert!((poor["score"].as_f64().unwrap() - 0.0).abs() < 1e-12);
}

#[test]
fn ranking_places_feasible_before_infeasible_then_sorts_score() {
    let candidates = json!([
        {
            "bmax_t":1.55,"window_fill":0.43,"winding_c":92.0,"impedance_pct":5.8,
            "total_loss_w":1200.0,"mass_kg":430.0
        },
        {
            "bmax_t":1.70,"window_fill":0.43,"winding_c":92.0,"impedance_pct":5.8,
            "total_loss_w":950.0,"mass_kg":360.0
        },
        {
            "bmax_t":1.52,"window_fill":0.41,"winding_c":88.0,"impedance_pct":5.7,
            "total_loss_w":980.0,"mass_kg":390.0
        }
    ]);
    let out = run("xfmr.candidate_rank", vec![candidates, constraints(), objectives()]);
    let ranking = out["ranking"].as_array().unwrap();
    assert_eq!(ranking.len(), 3);
    assert_eq!(ranking[0]["index"], json!(2));
    assert_eq!(ranking[0]["pass"], json!(true));
    assert_eq!(ranking[2]["index"], json!(1));
    assert_eq!(ranking[2]["pass"], json!(false));
}

#[test]
fn deterministic_ties_keep_original_index_order() {
    let candidates = json!([
        {"x":10.0},
        {"x":10.0},
        {"x":10.0}
    ]);
    let out = run(
        "xfmr.candidate_rank",
        vec![
            candidates,
            json!([]),
            json!([{"metric":"x","kind":"minimize","good":0.0,"bad":20.0}])
        ],
    );
    let ranking = out["ranking"].as_array().unwrap();
    assert_eq!(ranking[0]["index"], json!(0));
    assert_eq!(ranking[1]["index"], json!(1));
    assert_eq!(ranking[2]["index"], json!(2));
}

#[test]
fn malformed_specs_fail_closed() {
    let metrics = json!({"x":1.0});
    assert_eq!(
        transformer_candidate::execute(
            "xfmr.candidate_evaluate",
            &[metrics.clone(), json!([{"metric":"x"}]), json!([])]
        ),
        Some(Err("ARG"))
    );
    assert_eq!(
        transformer_candidate::execute(
            "xfmr.candidate_evaluate",
            &[
                metrics.clone(),
                json!([]),
                json!([{"metric":"x","kind":"minimize","good":2.0,"bad":1.0}])
            ]
        ),
        Some(Err("DOMAIN"))
    );
    assert_eq!(
        transformer_candidate::execute(
            "xfmr.candidate_evaluate",
            &[
                metrics,
                json!([]),
                json!([{"metric":"missing","kind":"target","target":1.0,"tolerance":1.0}])
            ]
        ),
        Some(Err("ARG"))
    );
}
