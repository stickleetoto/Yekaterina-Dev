use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ComputeParams {
    /// Single opcode. Omit when using ops or pipe.
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
    /// Optional pipeline/batch input available as $input.
    #[serde(default)]
    pub input: Option<Value>,
    /// Return all pipeline intermediate results. Default false.
    #[serde(default)]
    pub all: bool,
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
