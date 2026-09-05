#[path = "../src/formula.rs"]
mod formula;
#[path = "../src/registry.rs"]
mod registry;
#[path = "../src/user_ops.rs"]
mod user_ops;

use std::collections::HashMap;
use serde_json::json;

#[test]
fn formula_parser_works() {
    let vars = HashMap::from([
        ("m".to_string(), 2.0),
        ("v".to_string(), 10.0),
    ]);
    assert_eq!(formula::eval("0.5*m*v^2", &vars).unwrap(), 100.0);
}

#[test]
fn formula_registry_define_and_run() {
    let mut reg = user_ops::UserRegistry::default();
    reg.define_formula(&[json!({
        "op":"user.energy",
        "p":["m","v"],
        "expr":"0.5*m*v^2"
    })]).unwrap();
    let result = reg.execute_formula("user.energy", &[json!(2), json!(10)]).unwrap().unwrap();
    assert_eq!(result, json!(100.0));
}

#[test]
fn formula_rejects_non_user_namespace() {
    let mut reg = user_ops::UserRegistry::default();
    assert_eq!(reg.define_formula(&[json!({
        "op":"math.hijack",
        "p":["x"],
        "expr":"x+1"
    })]), Err("NAME"));
}
