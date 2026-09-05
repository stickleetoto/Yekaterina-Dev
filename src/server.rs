use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};

use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::{
    engine, formula,
    model::{ComputeParams, FindParams, SpecParams},
    registry, storage,
    user_ops::{self, UserOp, UserRegistry},
};

const MAX_BATCH: usize = 1024;
const MAX_PIPE: usize = 256;
const MAX_VALUE_NODES: usize = 200_000;
const MAX_STRING_BYTES: usize = 1_000_000;
const MAX_RESULT_NODES: usize = 100_000;
const MAX_RESULT_BYTES: usize = 1_000_000;
const MAX_UDO_DEPTH: usize = 32;

#[derive(Clone)]
pub struct Yekaterina {
    user_ops: Arc<RwLock<UserRegistry>>,
    store_dir: Arc<PathBuf>,
}

impl Default for Yekaterina {
    fn default() -> Self { Self::new() }
}

impl Yekaterina {
    pub fn new() -> Self { Self::with_store_dir(storage::default_store_dir()) }

    pub fn with_store_dir(store_dir: PathBuf) -> Self {
        let registry = storage::load(&store_dir).unwrap_or_default();
        Self { user_ops: Arc::new(RwLock::new(registry)), store_dir: Arc::new(store_dir) }
    }

    async fn execute_any(&self, opcode: &str, args: &[Value]) -> Result<Value, &'static str> {
        self.execute_any_depth(opcode, args, 0).await
    }

    fn execute_any_depth<'a>(
        &'a self,
        opcode: &'a str,
        args: &'a [Value],
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Value, &'static str>> + Send + 'a>> {
        Box::pin(async move {
            if depth > MAX_UDO_DEPTH { return Err("CYCLE"); }
            if depth > 0 && opcode.trim().to_ascii_lowercase().starts_with("udo.") { return Err("CONTROL"); }

            if let Some(spec) = registry::resolve(opcode) {
                match spec.opcode {
                    "udo.formula" => return self.mutate_registry(|r| r.define_formula(args)).await,
                    "udo.composite" => return self.mutate_registry(|r| r.define_composite(args)).await,
                    "udo.remove" => return self.mutate_registry(|r| r.remove(args)).await,
                    "udo.import" => return self.mutate_registry(|r| r.import_pack(args)).await,
                    "udo.uninstall" => return self.mutate_registry(|r| r.uninstall_pack(args)).await,
                    "udo.list" => {
                        if !args.is_empty() { return Err("ARG"); }
                        return Ok(json!(self.user_ops.read().await.list()));
                    }
                    "udo.export" => return self.user_ops.read().await.export_pack(args),
                    "expr.eval" => return eval_expression(args),
                    canonical => return engine::execute(canonical, args),
                }
            }

            let dynamic = { self.user_ops.read().await.lookup(opcode) };
            match dynamic {
                Some(UserOp::Formula(spec)) => user_ops::execute_formula_spec(&spec, args),
                Some(UserOp::Composite(spec)) => {
                    if args.len() != spec.params.len() { return Err("ARG"); }
                    let mut results = Vec::with_capacity(spec.pipe.len());
                    for item in &spec.pipe {
                        let (child, raw_args) = user_ops::parse_step(item)?;
                        let resolved = user_ops::resolve_composite_args(&raw_args, args, &results)?;
                        let value = self.execute_any_depth(&child, &resolved, depth + 1).await?;
                        if output_value_too_large(&value) { return Err("OUT_LIMIT"); }
                        results.push(value);
                        if output_values_too_large(&results) { return Err("OUT_LIMIT"); }
                    }
                    results.pop().ok_or("ARG")
                }
                None => Err("OP"),
            }
        })
    }

    async fn mutate_registry<F>(&self, f: F) -> Result<Value, &'static str>
    where
        F: FnOnce(&mut UserRegistry) -> Result<Value, &'static str>,
    {
        let mut guard = self.user_ops.write().await;
        let before = guard.clone();
        let result = f(&mut guard)?;
        if let Err(e) = storage::save(&self.store_dir, &guard) {
            *guard = before;
            return Err(e);
        }
        Ok(result)
    }

    async fn run_batch(&self, items: &[Value], input: Option<&Value>) -> Value {
        if items.len() > MAX_BATCH { return json!({"e":"LIMIT"}); }
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let parsed = parse_batch_item(item);
            let value = match parsed {
                Ok((op, raw_args)) => match resolve_args(&raw_args, input, &results) {
                    Ok(args) => match self.execute_any(&op, &args).await { Ok(v) => v, Err(e) => json!({"e":e}) },
                    Err(e) => json!({"e":e}),
                },
                Err(e) => json!({"e":e}),
            };
            if output_value_too_large(&value) { results.push(json!({"e":"OUT_LIMIT"})); }
            else { results.push(value); }
            if output_values_too_large(&results) { return json!({"e":"OUT_LIMIT"}); }
        }
        json!({"r":results})
    }

    async fn run_pipeline(&self, items: &[Value], input: Option<&Value>, all: bool) -> Value {
        if items.is_empty() { return json!({"e":"ARG"}); }
        if items.len() > MAX_PIPE { return json!({"e":"LIMIT"}); }
        let mut results = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let (op, raw_args) = match parse_batch_item(item) { Ok(v) => v, Err(e) => return json!({"e":e,"i":i}) };
            let args = match resolve_args(&raw_args, input, &results) { Ok(v) => v, Err(e) => return json!({"e":e,"i":i}) };
            match self.execute_any(&op, &args).await {
                Ok(v) => {
                    if output_value_too_large(&v) { return json!({"e":"OUT_LIMIT","i":i}); }
                    results.push(v);
                    if output_values_too_large(&results) { return json!({"e":"OUT_LIMIT","i":i}); }
                }
                Err(e) => return json!({"e":e,"i":i}),
            }
        }
        if all { json!({"r":results}) } else { json!({"r":results.pop().unwrap_or(Value::Null)}) }
    }
}

#[tool_router]
impl Yekaterina {
    #[tool(
        name = "yk.compute",
        description = "Run one opcode, compact batch, pipeline, expression, or persistent UDO control. Exact integers/decimals use strings."
    )]
    async fn compute(&self, Parameters(p): Parameters<ComputeParams>) -> String {
        if request_too_large(&p) { return render(json!({"e":"LIMIT"})); }
        let out = if !p.pipe.is_empty() {
            self.run_pipeline(&p.pipe, p.input.as_ref(), p.all).await
        } else if !p.ops.is_empty() {
            self.run_batch(&p.ops, p.input.as_ref()).await
        } else if let Some(op) = p.op.as_deref() {
            match self.execute_any(op, &p.a).await { Ok(v) => json!({"r":v}), Err(e) => json!({"e":e}) }
        } else { json!({"e":"ARG"}) };
        render(out)
    }

    #[tool(
        name = "yk.find",
        description = "Find compute opcodes lazily across built-ins, persistent user operations, and installed packs."
    )]
    async fn find(&self, Parameters(p): Parameters<FindParams>) -> String {
        let limit = p.l.unwrap_or(5).clamp(1, 20);
        let mut hits: Vec<String> = registry::search(&p.q, limit).into_iter().map(|spec| spec.opcode.to_string()).collect();
        if hits.len() < limit {
            for hit in self.user_ops.read().await.find(&p.q, limit - hits.len()) {
                if !hits.contains(&hit) { hits.push(hit); }
            }
        }
        render(json!({"r":hits}))
    }

    #[tool(
        name = "yk.spec",
        description = "Return compact argument/result/capability spec for one built-in, user, or pack opcode."
    )]
    async fn spec(&self, Parameters(p): Parameters<SpecParams>) -> String {
        if let Some(spec) = registry::resolve(&p.op) {
            return render(json!({
                "op":spec.opcode,
                "a":spec.args,
                "r":spec.returns,
                "s":source_code(spec.source),
                "c":registry::capability_code(spec.opcode),
                "k":registry::cost_code(spec.opcode)
            }));
        }
        match self.user_ops.read().await.lookup(&p.op) {
            Some(UserOp::Formula(spec)) => render(json!({"op":spec.opcode,"a":spec.params,"r":"number","s":"f","c":"d","k":"1"})),
            Some(UserOp::Composite(spec)) => render(json!({"op":spec.opcode,"a":spec.params,"r":"value","s":"c","c":"d","k":"p"})),
            None => render(json!({"e":"OP"})),
        }
    }
}

#[tool_handler(
    name = "yekaterina",
    version = "1.0.0",
    instructions = "Token-efficient compute engine. Prefer yk.compute. Use yk.find/spec only for unknown operations. Use pipe or persistent Composite UDOs to avoid MCP round trips."
)]
impl ServerHandler for Yekaterina {}

fn eval_expression(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let obj = args[0].as_object().ok_or("TYPE")?;
    let expr = obj.get("e").and_then(Value::as_str).ok_or("ARG")?;
    let vars_obj = match obj.get("v") {
        Some(v) => Some(v.as_object().ok_or("TYPE")?),
        None => None,
    };
    let mut vars = std::collections::HashMap::new();
    if let Some(values) = vars_obj {
        if values.len() > formula::MAX_PARAMS { return Err("LIMIT"); }
        for (name, value) in values {
            let mut bytes = name.bytes();
            let Some(first) = bytes.next() else { return Err("NAME"); };
            if !(first.is_ascii_alphabetic() || first == b'_') { return Err("NAME"); }
            if !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_') { return Err("NAME"); }
            vars.insert(name.clone(), value.as_f64().ok_or("TYPE")?);
        }
    }
    Ok(json!(formula::eval(expr, &vars)?))
}

fn source_code(source: registry::OperationSource) -> &'static str {
    match source {
        registry::OperationSource::BuiltIn => "b",
        registry::OperationSource::Formula => "f",
        registry::OperationSource::Composite => "c",
        registry::OperationSource::Wasm => "w",
    }
}

fn parse_batch_item(item: &Value) -> Result<(String, Vec<Value>), &'static str> { user_ops::parse_step(item) }

fn resolve_args(args: &[Value], input: Option<&Value>, results: &[Value]) -> Result<Vec<Value>, &'static str> {
    args.iter().map(|v| resolve_value(v, input, results)).collect()
}

fn resolve_value(v: &Value, input: Option<&Value>, results: &[Value]) -> Result<Value, &'static str> {
    match v {
        Value::String(s) if s == "$input" => input.cloned().ok_or("REF"),
        Value::String(s) if s.starts_with('$') && s.len() > 1 && s[1..].bytes().all(|b| b.is_ascii_digit()) => {
            let idx = s[1..].parse::<usize>().map_err(|_| "REF")?;
            results.get(idx).cloned().ok_or("REF")
        }
        Value::Array(xs) => xs.iter().map(|x| resolve_value(x, input, results)).collect::<Result<Vec<_>, _>>().map(Value::Array),
        Value::Object(obj) => {
            let mut out = serde_json::Map::with_capacity(obj.len());
            for (k, x) in obj { out.insert(k.clone(), resolve_value(x, input, results)?); }
            Ok(Value::Object(out))
        }
        Value::String(s) if s.starts_with('$') => Err("REF"),
        _ => Ok(v.clone()),
    }
}

fn request_too_large(p: &ComputeParams) -> bool {
    if p.ops.len() > MAX_BATCH || p.pipe.len() > MAX_PIPE { return true; }
    let mut nodes = 0usize;
    let mut strings = 0usize;
    for v in p.a.iter().chain(p.ops.iter()).chain(p.pipe.iter()).chain(p.input.iter()) {
        if !measure_value(v, &mut nodes, &mut strings, MAX_VALUE_NODES, MAX_STRING_BYTES) { return true; }
    }
    false
}

fn output_values_too_large(values: &[Value]) -> bool {
    let mut nodes = 1usize;
    let mut strings = 0usize;
    values.iter().any(|v| !measure_value(v, &mut nodes, &mut strings, MAX_RESULT_NODES, MAX_RESULT_BYTES))
}

fn output_value_too_large(v: &Value) -> bool {
    let mut nodes = 0usize;
    let mut strings = 0usize;
    !measure_value(v, &mut nodes, &mut strings, MAX_RESULT_NODES, MAX_RESULT_BYTES)
}

fn render(v: Value) -> String {
    if output_value_too_large(&v) { return json!({"e":"OUT_LIMIT"}).to_string(); }
    let s = v.to_string();
    if s.len() > MAX_RESULT_BYTES { json!({"e":"OUT_LIMIT"}).to_string() } else { s }
}

fn measure_value(v: &Value, nodes: &mut usize, strings: &mut usize, max_nodes: usize, max_strings: usize) -> bool {
    *nodes += 1;
    if *nodes > max_nodes { return false; }
    match v {
        Value::String(s) => {
            *strings = strings.saturating_add(s.len());
            *strings <= max_strings
        }
        Value::Array(xs) => xs.iter().all(|x| measure_value(x, nodes, strings, max_nodes, max_strings)),
        Value::Object(obj) => obj.iter().all(|(k, x)| {
            *strings = strings.saturating_add(k.len());
            *strings <= max_strings && measure_value(x, nodes, strings, max_nodes, max_strings)
        }),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn temp_dir() -> PathBuf {
        let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        env::temp_dir().join(format!("yekaterina-server-test-{}-{n}", std::process::id()))
    }

    #[tokio::test]
    async fn composite_executes_and_persists() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        yk.execute_any("udo.formula", &[json!({"op":"user.double","p":["x"],"expr":"x*2"})]).await.unwrap();
        yk.execute_any("udo.composite", &[json!({
            "op":"user.quad","p":["x"],
            "pipe":[["user.double","$a0"],["user.double","$0"]]
        })]).await.unwrap();
        assert_eq!(yk.execute_any("user.quad", &[json!(3)]).await.unwrap(), json!(12.0));

        drop(yk);
        let reloaded = Yekaterina::with_store_dir(dir.clone());
        assert_eq!(reloaded.execute_any("user.quad", &[json!(4)]).await.unwrap(), json!(16.0));
        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn expression_eval_is_stateless() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        assert_eq!(yk.execute_any("expr.eval", &[json!({"e":"a*b+1","v":{"a":2,"b":5}})]).await.unwrap(), json!(11.0));
        let _ = fs::remove_dir_all(dir);
    }
}
