use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct RawComputeParams {
    #[serde(default)]
    op: Option<String>,
    #[serde(default)]
    a: Vec<Value>,
    #[serde(default)]
    ops: Vec<Value>,
    #[serde(default)]
    pipe: Vec<Value>,
    #[serde(default)]
    program: Option<Value>,
    #[serde(default)]
    input: Option<Value>,
    #[serde(default)]
    all: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(try_from = "RawComputeParams")]
pub struct ComputeParams {
    /// Single opcode. Omit when using ops, pipe, or program.
    #[serde(default)]
    pub op: Option<String>,
    /// Arguments for a single opcode.
    #[serde(default)]
    pub a: Vec<Value>,
    /// Batch calls. Each item may be {"op":"...","a":[...]} or ["opcode", arg1, ...].
    #[serde(default)]
    pub ops: Vec<Value>,
    /// Sequential pipeline. References: $input, $0, $1, ... . Returns only the last value by default.
    #[serde(default)]
    pub pipe: Vec<Value>,
    /// v1.3 named compute program. Compiled to pipe during request deserialization.
    #[serde(default)]
    pub program: Option<Value>,
    /// Optional pipeline/batch/program input available as $input.
    #[serde(default)]
    pub input: Option<Value>,
    /// Return all pipeline/program intermediate results. Default false.
    #[serde(default)]
    pub all: bool,
}

impl TryFrom<RawComputeParams> for ComputeParams {
    type Error = String;

    fn try_from(raw: RawComputeParams) -> Result<Self, Self::Error> {
        if let Some(program) = raw.program.as_ref() {
            if raw.op.is_some() || !raw.a.is_empty() || !raw.ops.is_empty() || !raw.pipe.is_empty() {
                return Err("YK_PROGRAM_ARG".to_string());
            }
            let compiled = crate::program::compile_program(program, raw.input.as_ref(), raw.all)
                .map_err(|e| format!("YK_PROGRAM_{e}"))?;
            return Ok(Self {
                op: None,
                a: Vec::new(),
                ops: Vec::new(),
                pipe: compiled.pipe,
                // The original graph is intentionally discarded after validation.
                // request_too_large measures the compiled pipe/input, while the
                // compiler separately measures the raw graph before pruning.
                program: None,
                input: raw.input,
                all: compiled.all,
            });
        }

        Ok(Self {
            op: raw.op,
            a: raw.a,
            ops: raw.ops,
            pipe: raw.pipe,
            program: None,
            input: raw.input,
            all: raw.all,
        })
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindParams {
    /// Search text.
    pub q: String,
    /// Maximum results. Default 5, maximum 20.
    #[serde(default)]
    pub l: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SpecParams {
    /// Opcode to inspect.
    pub op: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_compute_params_deserialize_unchanged_without_program() {
        let p: ComputeParams = serde_json::from_value(json!({
            "op":"math.add",
            "a":[1,2],
            "input":{"x":1},
            "all":true
        })).unwrap();
        assert_eq!(p.op.as_deref(), Some("math.add"));
        assert_eq!(p.a, vec![json!(1), json!(2)]);
        assert!(p.ops.is_empty());
        assert!(p.pipe.is_empty());
        assert!(p.program.is_none());
        assert_eq!(p.input, Some(json!({"x":1})));
        assert!(p.all);
    }

    #[test]
    fn program_deserialization_compiles_to_legacy_pipe() {
        let p: ComputeParams = serde_json::from_value(json!({
            "input":{"x":5},
            "program":{
                "steps":[
                    {"id":"a","op":"math.add","a":["$input.x",1]},
                    {"id":"b","op":"math.mul","a":["$a",10]}
                ],
                "return":"b"
            }
        })).unwrap();
        assert!(p.op.is_none());
        assert!(p.ops.is_empty());
        assert_eq!(p.pipe, vec![
            json!({"op":"math.add","a":[5,1]}),
            json!({"op":"math.mul","a":["$0",10]})
        ]);
        assert!(p.program.is_none());
        assert!(!p.all);
    }

    #[test]
    fn program_is_exclusive_with_legacy_compute_modes() {
        let err = serde_json::from_value::<ComputeParams>(json!({
            "op":"math.add",
            "a":[1,2],
            "program":{"steps":[{"id":"a","op":"math.add","a":[1,2]}]}
        })).unwrap_err().to_string();
        assert!(err.contains("YK_PROGRAM_ARG"));
    }

    #[test]
    fn compiler_error_identity_is_preserved_in_deserialization_error() {
        let err = serde_json::from_value::<ComputeParams>(json!({
            "program":{"steps":[
                {"id":"a","op":"math.add","a":["$b",1]},
                {"id":"b","op":"math.add","a":["$a",1]}
            ]}
        })).unwrap_err().to_string();
        assert!(err.contains("YK_PROGRAM_CYCLE"));
    }
}
