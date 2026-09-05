#[path = "../src/formula.rs"]
mod formula;
#[path = "../src/registry.rs"]
mod registry;
#[path = "../src/user_ops.rs"]
mod user_ops;

use serde_json::json;
use user_ops::{UserOp, UserRegistry};

#[test]
fn composite_definition_and_refs_are_validated() {
    let mut reg = UserRegistry::default();
    reg.define_formula(&[json!({"op":"user.double","p":["x"],"expr":"x*2"})]).unwrap();
    reg.define_composite(&[json!({
        "op":"user.mean_double",
        "p":["xs"],
        "pipe":[
            ["stat.mean","$a0"],
            ["user.double","$0"]
        ]
    })]).unwrap();
    match reg.lookup("user.mean_double") {
        Some(UserOp::Composite(c)) => assert_eq!(c.pipe.len(), 2),
        _ => panic!("composite not registered"),
    }
}

#[test]
fn composite_rejects_unknown_dependency() {
    let mut reg = UserRegistry::default();
    assert_eq!(reg.define_composite(&[json!({
        "op":"user.bad",
        "p":["x"],
        "pipe":[["user.missing","$a0"]]
    })]), Err("OP"));
}

#[test]
fn composite_rejects_control_opcode() {
    let mut reg = UserRegistry::default();
    assert_eq!(reg.define_composite(&[json!({
        "op":"user.bad",
        "p":["x"],
        "pipe":[["udo.remove","user.any"]]
    })]), Err("CONTROL"));
}

#[test]
fn indirect_cycle_is_rejected() {
    let mut reg = UserRegistry::default();
    reg.define_formula(&[json!({"op":"user.seed","p":["x"],"expr":"x"})]).unwrap();
    reg.define_composite(&[json!({
        "op":"user.a","p":["x"],"pipe":[["user.seed","$a0"]]
    })]).unwrap();
    reg.define_composite(&[json!({
        "op":"user.b","p":["x"],"pipe":[["user.a","$a0"]]
    })]).unwrap();
    assert_eq!(reg.define_composite(&[json!({
        "op":"user.a","p":["x"],"pipe":[["user.b","$a0"]]
    })]), Err("CYCLE"));
}

#[test]
fn export_rewrites_user_references_and_imports_pack() {
    let mut reg = UserRegistry::default();
    reg.define_formula(&[json!({"op":"user.double","p":["x"],"expr":"x*2"})]).unwrap();
    reg.define_composite(&[json!({
        "op":"user.quad",
        "p":["x"],
        "pipe":[["user.double","$a0"],["user.double","$0"]]
    })]).unwrap();

    let pack = reg.export_pack(&[json!({
        "name":"demo",
        "ops":["user.double","user.quad"]
    })]).unwrap();
    let encoded = pack.to_string();
    assert!(encoded.contains("pack.demo.double"));
    assert!(encoded.contains("pack.demo.quad"));
    assert!(!encoded.contains("user.double"));

    let mut other = UserRegistry::default();
    assert_eq!(other.import_pack(&[pack]).unwrap(), json!(2));
    assert!(other.lookup("pack.demo.double").is_some());
    assert!(other.lookup("pack.demo.quad").is_some());
}

#[test]
fn snapshot_round_trip_preserves_ops() {
    let mut reg = UserRegistry::default();
    reg.define_formula(&[json!({"op":"user.energy","p":["m","v"],"expr":"0.5*m*v^2"})]).unwrap();
    let restored = UserRegistry::from_snapshot(reg.snapshot()).unwrap();
    assert!(restored.lookup("user.energy").is_some());
}

#[test]
fn referenced_operation_cannot_be_removed() {
    let mut reg = UserRegistry::default();
    reg.define_formula(&[json!({"op":"user.double","p":["x"],"expr":"x*2"})]).unwrap();
    reg.define_composite(&[json!({
        "op":"user.quad","p":["x"],"pipe":[["user.double","$a0"],["user.double","$0"]]
    })]).unwrap();
    assert_eq!(reg.remove(&[json!("user.double")]), Err("IN_USE"));
}
