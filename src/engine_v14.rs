//! v1.4 execution shim.
//!
//! Newly registered v1.4 math operations dispatch here first. Every v1.3
//! operation continues through the frozen v1.3 engine shim.

use serde_json::Value;

use crate::{engine_v13, math_v14, registry};

pub fn execute(opcode: &str, args: &[Value]) -> Result<Value, &'static str> {
    let spec = registry::resolve(opcode).ok_or("OP")?;
    if let Some(result) = math_v14::execute(spec.opcode, args) {
        return result;
    }
    engine_v13::execute(spec.opcode, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn v13_dispatch_is_preserved() {
        assert_eq!(execute("math.add", &[json!(20), json!(22)]).unwrap(), json!(42.0));
        assert!(execute("xfmr.skin_depth", &[json!(60.0), json!(1.724e-8), json!(1.0)]).is_ok());
    }

    #[test]
    fn v14_math_dispatch_is_live() {
        assert_eq!(execute("linear_equation", &[json!(2), json!(-8)]).unwrap(), json!(4.0));
        assert_eq!(
            execute(
                "linalg.solve",
                &[json!([[2.0, 1.0], [1.0, -1.0]]), json!([5.0, 1.0])],
            ).unwrap(),
            json!([2.0, 1.0])
        );
    }
}
