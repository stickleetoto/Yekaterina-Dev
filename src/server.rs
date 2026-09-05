use std::{borrow::Cow, future::Future, path::PathBuf, pin::Pin, sync::Arc};

use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};

use crate::{
    engine, formula,
    limits::{self, ResultBudget},
    model::{ComputeParams, FindParams, SpecParams},
    registry, storage,
    user_ops::{self, UserOp, UserRegistry},
};

const MAX_BATCH: usize = 1024;
const MAX_PIPE: usize = 256;
const MAX_UDO_DEPTH: usize = 32;

#[derive(Clone)]
pub struct Yekaterina {
    /// Copy-on-write user-operation registry.
    ///
    /// v1.0.0 stored the registry directly and took a fresh read lock for
    /// *every* lookup, including once per step inside a composite operation.
    /// A concurrent `udo.remove` or `udo.import` landing between two steps of
    /// the same composite therefore let that composite observe two different
    /// registry versions -- a torn read. `rmcp` already dispatches every
    /// `tools/call` as an independent task on a multi-thread runtime, so this
    /// was reachable in v1.0.0; only the serial request pattern of stdio
    /// clients kept it hidden.
    ///
    /// Holding one read guard across a whole composite is not a fix: tokio's
    /// `RwLock` is write-preferring, so a second read acquisition from the same
    /// task while a writer waits would deadlock. Instead the registry is
    /// immutable behind an `Arc`; a reader clones the `Arc` once and the whole
    /// operation runs against that snapshot.
    user_ops: Arc<RwLock<Arc<UserRegistry>>>,
    /// Serializes mutations *and* their on-disk snapshot writes.
    ///
    /// This gate, not the registry lock, is what orders snapshot generations on
    /// disk. It lets the registry write lock be held only long enough to swap an
    /// `Arc`, so readers are never blocked behind an `fsync`.
    store_gate: Arc<Mutex<()>>,
    store_dir: Arc<PathBuf>,
}

impl Default for Yekaterina {
    fn default() -> Self { Self::new() }
}

impl Yekaterina {
    pub fn new() -> Self { Self::with_store_dir(storage::default_store_dir()) }

    pub fn with_store_dir(store_dir: PathBuf) -> Self {
        let registry = storage::load(&store_dir).unwrap_or_default();
        Self {
            user_ops: Arc::new(RwLock::new(Arc::new(registry))),
            store_gate: Arc::new(Mutex::new(())),
            store_dir: Arc::new(store_dir),
        }
    }

    /// Take one immutable view of the user registry.
    ///
    /// The read lock is held only for an `Arc` clone.
    async fn registry_snapshot(&self) -> Arc<UserRegistry> {
        Arc::clone(&*self.user_ops.read().await)
    }

    /// Reuse the caller's snapshot when it has one, otherwise take a fresh one.
    ///
    /// Reusing is what keeps a composite coherent; taking one lazily is what
    /// keeps built-in dispatch lock-free.
    async fn snapshot_or(&self, existing: Option<&Arc<UserRegistry>>) -> Arc<UserRegistry> {
        match existing {
            Some(snap) => Arc::clone(snap),
            None => self.registry_snapshot().await,
        }
    }

    /// Entry point for one top-level operation.
    ///
    /// No snapshot is taken here. Built-in opcodes resolve through the static
    /// registry and never touch the user registry at all, exactly as in v1.0.0;
    /// taking a snapshot eagerly cost a lock acquisition per *batch item* and
    /// measured 5% slower on a 1,000-item batch.
    ///
    /// The snapshot is acquired lazily at the one place that needs it -- the
    /// dynamic lookup -- and from then on is passed down unchanged. A composite
    /// is only ever reached *through* that lookup, so by the time one executes a
    /// snapshot is always held, and all of its steps share it.
    ///
    /// Granularity is per top-level operation, not per request: a batch whose
    /// first item defines a UDO and whose second item calls it still works, as
    /// in v1.0.0. No mutation can originate inside a composite -- `udo.*` is
    /// rejected both at composite definition time and at any depth above zero.
    async fn execute_any(&self, opcode: &str, args: &[Value]) -> Result<Value, &'static str> {
        self.execute_any_depth(opcode, args, 0, None).await
    }

    fn execute_any_depth<'a>(
        &'a self,
        opcode: &'a str,
        args: &'a [Value],
        depth: usize,
        snapshot: Option<&'a Arc<UserRegistry>>,
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
                        let snap = self.snapshot_or(snapshot).await;
                        return Ok(json!(snap.list()));
                    }
                    "udo.export" => {
                        let snap = self.snapshot_or(snapshot).await;
                        return snap.export_pack(args);
                    }
                    "expr.eval" => return eval_expression(args),
                    canonical => return engine::execute(canonical, args),
                }
            }

            // First point that needs the user registry. Everything above this
            // resolved through the static built-in registry without a lock.
            let snap = self.snapshot_or(snapshot).await;
            let dynamic = snap.lookup(opcode);
            match dynamic {
                Some(UserOp::Formula(spec)) => user_ops::execute_formula_spec(&spec, args),
                Some(UserOp::Composite(spec)) => {
                    if args.len() != spec.params.len() { return Err("ARG"); }
                    let mut results = Vec::with_capacity(spec.pipe.len());
                    let mut budget = ResultBudget::new();
                    for item in &spec.pipe {
                        let (child, raw_args) = user_ops::parse_step_ref(item)?;
                        let resolved = user_ops::resolve_composite_args(raw_args, args, &results)?;
                        let value = self
                            .execute_any_depth(&child, &resolved, depth + 1, Some(&snap))
                            .await?;
                        if limits::value_too_large(&value) { return Err("OUT_LIMIT"); }
                        if !budget.admit(&value) { return Err("OUT_LIMIT"); }
                        results.push(value);
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
        // One mutation at a time. Every snapshot generation on disk is written
        // while this gate is held, so generations are published in the same
        // order the mutations were applied.
        let _gate = self.store_gate.lock().await;

        let mut candidate = (*self.registry_snapshot().await).clone();
        let result = f(&mut candidate)?;

        // Persist before publishing. v1.0.0 mutated the live registry, wrote,
        // and rolled back on write failure, which held the registry write lock
        // across the fsync and left a window where memory and disk disagreed.
        // Committing to disk first reaches the same observable outcome -- the
        // same error returned, the in-memory registry unchanged -- with the
        // write lock held only for the swap below.
        storage::save(&self.store_dir, &candidate)?;

        *self.user_ops.write().await = Arc::new(candidate);
        Ok(result)
    }

    async fn run_batch(&self, items: &[Value], input: Option<&Value>) -> Value {
        if items.len() > MAX_BATCH { return json!({"e":"LIMIT"}); }
        let mut results = Vec::with_capacity(items.len());
        let mut budget = ResultBudget::new();
        for item in items {
            let parsed = parse_batch_item(item);
            let value = match parsed {
                Ok((op, raw_args)) => match resolve_args(raw_args, input, &results) {
                    Ok(args) => match self.execute_any(&op, &args).await { Ok(v) => v, Err(e) => json!({"e":e}) },
                    Err(e) => json!({"e":e}),
                },
                Err(e) => json!({"e":e}),
            };
            // The stored value, not the computed one, is what the accumulated
            // budget is charged for: an oversized item becomes an OUT_LIMIT
            // marker in the result vector, exactly as in v1.0.0.
            let stored = if limits::value_too_large(&value) { json!({"e":"OUT_LIMIT"}) } else { value };
            if !budget.admit(&stored) { return json!({"e":"OUT_LIMIT"}); }
            results.push(stored);
        }
        json!({"r":results})
    }

    async fn run_pipeline(&self, items: &[Value], input: Option<&Value>, all: bool) -> Value {
        if items.is_empty() { return json!({"e":"ARG"}); }
        if items.len() > MAX_PIPE { return json!({"e":"LIMIT"}); }
        let mut results = Vec::with_capacity(items.len());
        let mut budget = ResultBudget::new();
        for (i, item) in items.iter().enumerate() {
            let (op, raw_args) = match parse_batch_item(item) { Ok(v) => v, Err(e) => return json!({"e":e,"i":i}) };
            let args = match resolve_args(raw_args, input, &results) { Ok(v) => v, Err(e) => return json!({"e":e,"i":i}) };
            match self.execute_any(&op, &args).await {
                Ok(v) => {
                    if limits::value_too_large(&v) { return json!({"e":"OUT_LIMIT","i":i}); }
                    if !budget.admit(&v) { return json!({"e":"OUT_LIMIT","i":i}); }
                    results.push(v);
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
            for hit in self.registry_snapshot().await.find(&p.q, limit - hits.len()) {
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
        match self.registry_snapshot().await.lookup(&p.op) {
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

fn parse_batch_item(item: &Value) -> Result<(Cow<'_, str>, &[Value]), &'static str> {
    user_ops::parse_step_ref(item)
}

/// Resolve `$input` / `$N` references in batch and pipeline arguments.
///
/// Returns a borrow when the argument tree contains no `$`-prefixed string.
/// v1.0.0 rebuilt the tree unconditionally, so a single 10,000-element array
/// argument was deep-copied on every item even with no references present --
/// and `parse_step` had already copied it once before that.
///
/// The borrowed path is only taken when [`user_ops::contains_reference`] is
/// false, and in that case every arm of [`resolve_value`] reduces to
/// `v.clone()`: no substitution can occur and no error can be raised, because
/// every error arm requires a `$`-prefixed string.
fn resolve_args<'a>(
    args: &'a [Value],
    input: Option<&Value>,
    results: &[Value],
) -> Result<Cow<'a, [Value]>, &'static str> {
    if !args.iter().any(user_ops::contains_reference) {
        return Ok(Cow::Borrowed(args));
    }
    args.iter()
        .map(|v| resolve_value(v, input, results))
        .collect::<Result<Vec<_>, _>>()
        .map(Cow::Owned)
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
        if !limits::measure_value(v, &mut nodes, &mut strings, limits::MAX_VALUE_NODES, limits::MAX_STRING_BYTES) { return true; }
    }
    false
}

fn render(v: Value) -> String {
    if limits::value_too_large(&v) { return json!({"e":"OUT_LIMIT"}).to_string(); }
    let s = v.to_string();
    if s.len() > limits::MAX_RESULT_BYTES { json!({"e":"OUT_LIMIT"}).to_string() } else { s }
}

#[cfg(test)]
mod resolution_tests {
    use super::*;

    /// The v1.0.0 resolver: always rebuilds, never borrows. Retained as the
    /// oracle for the Phase 2B borrow optimization.
    fn resolve_args_always_owned(
        args: &[Value],
        input: Option<&Value>,
        results: &[Value],
    ) -> Result<Vec<Value>, &'static str> {
        args.iter().map(|v| resolve_value(v, input, results)).collect()
    }

    struct Gen(u64);

    impl Gen {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }

        fn value(&mut self, depth: usize) -> Value {
            match self.next() % if depth >= 3 { 6 } else { 8 } {
                0 => Value::Null,
                1 => json!(self.next() % 100),
                2 => json!(self.next() % 2 == 0),
                3 => Value::String("plain".into()),
                // Reference-shaped strings, valid and invalid, at the same rate
                // as ordinary values so both resolver paths are exercised.
                4 => Value::String(match self.next() % 4 {
                    0 => "$input".to_string(),
                    1 => format!("${}", self.next() % 4),
                    2 => "$nope".to_string(),
                    _ => format!("$a{}", self.next() % 3),
                }),
                5 => json!((self.next() % 100) as f64 / 3.0),
                6 => {
                    let n = (self.next() % 4) as usize;
                    Value::Array((0..n).map(|_| self.value(depth + 1)).collect())
                }
                _ => {
                    let n = (self.next() % 3) as usize;
                    let mut m = serde_json::Map::new();
                    for i in 0..n {
                        m.insert(format!("k{i}"), self.value(depth + 1));
                    }
                    Value::Object(m)
                }
            }
        }
    }

    #[test]
    fn borrowing_resolver_matches_always_owned_resolver() {
        for seed in 0..600u64 {
            let mut g = Gen(seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(7));
            let args: Vec<Value> = (0..4).map(|_| g.value(0)).collect();
            let results: Vec<Value> = (0..3).map(|_| g.value(2)).collect();
            let input = if seed % 3 == 0 { None } else { Some(json!({"in": seed})) };

            let expected = resolve_args_always_owned(&args, input.as_ref(), &results);
            let actual = resolve_args(&args, input.as_ref(), &results);

            match (expected, actual) {
                (Ok(e), Ok(a)) => assert_eq!(e.as_slice(), a.as_ref(), "seed {seed}"),
                (Err(e), Err(a)) => assert_eq!(e, a, "seed {seed}: error identity differs"),
                (e, a) => panic!("seed {seed}: outcome differs: {e:?} vs {a:?}"),
            }
        }
    }

    #[test]
    fn reference_free_args_take_the_borrowed_path() {
        let args = vec![json!([1, 2, 3]), json!({"k": "plain"}), json!(7)];
        let out = resolve_args(&args, None, &[]).unwrap();
        assert!(matches!(out, Cow::Borrowed(_)), "no-reference args must not be cloned");
        assert_eq!(out.as_ref(), args.as_slice());
    }

    #[test]
    fn any_dollar_string_forces_the_owned_path() {
        // Even an invalid reference must leave the borrowed path, because the
        // owned path is where v1.0.0 raises REF.
        let args = vec![json!(["ok", "$nope"])];
        assert!(matches!(resolve_args(&args, None, &[]), Err("REF")));
        let nested = vec![json!({"k": {"deep": "$0"}})];
        let out = resolve_args(&nested, None, &[json!(42)]).unwrap();
        assert!(matches!(out, Cow::Owned(_)));
        assert_eq!(out.as_ref(), &[json!({"k": {"deep": 42}})]);
    }

    #[test]
    fn parse_step_ref_matches_parse_step() {
        let items = vec![
            json!({"op": "math.add", "a": [1, 2]}),
            json!({"op": "  Math.Add  ", "a": []}),
            json!({"op": "stat.sum"}),
            json!({"op": "stat.sum", "a": "not-an-array"}),
            json!(["math.mul", 3, 4]),
            json!(["  MATH.SUB  "]),
            json!({"nope": 1}),
            json!([]),
            json!([5, 6]),
            json!("scalar"),
            json!(42),
        ];
        for item in &items {
            let owned = user_ops::parse_step(item);
            let borrowed = user_ops::parse_step_ref(item);
            match (owned, borrowed) {
                (Ok((o_op, o_args)), Ok((b_op, b_args))) => {
                    assert_eq!(o_op, b_op.as_ref(), "opcode differs for {item}");
                    assert_eq!(o_args.as_slice(), b_args, "args differ for {item}");
                }
                (Err(o), Err(b)) => assert_eq!(o, b, "error identity differs for {item}"),
                (o, b) => panic!("outcome differs for {item}: {o:?} vs {b:?}"),
            }
        }
    }

    #[test]
    fn already_lowercase_opcode_is_not_reallocated() {
        let item = json!(["math.add", 1]);
        let (op, _) = user_ops::parse_step_ref(&item).unwrap();
        assert!(matches!(op, Cow::Borrowed(_)));
        let upper = json!(["Math.Add", 1]);
        let (op, _) = user_ops::parse_step_ref(&upper).unwrap();
        assert!(matches!(op, Cow::Owned(_)));
        assert_eq!(op.as_ref(), "math.add");
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

    /// R1. A snapshot taken before a mutation must keep resolving the
    /// definitions it was taken with. This is the property that makes a
    /// composite's steps coherent: the whole composite runs against one
    /// snapshot, so a concurrent redefinition cannot land between two steps.
    #[tokio::test]
    async fn registry_snapshot_is_immune_to_later_mutation() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        yk.execute_any("udo.formula", &[json!({"op":"user.k","p":["x"],"expr":"x*2"})])
            .await
            .unwrap();

        let snapshot = yk.registry_snapshot().await;

        // Redefine the operation the snapshot already captured.
        yk.execute_any("udo.formula", &[json!({"op":"user.k","p":["x"],"expr":"x*100"})])
            .await
            .unwrap();

        // The snapshot still sees the original definition ...
        let old = snapshot.lookup("user.k").expect("snapshot retains the op");
        match old {
            UserOp::Formula(f) => assert_eq!(f.expr, "x*2", "snapshot was mutated in place"),
            _ => panic!("expected a formula"),
        }
        // ... while a fresh execution sees the new one.
        assert_eq!(yk.execute_any("user.k", &[json!(3)]).await.unwrap(), json!(300.0));
        let _ = fs::remove_dir_all(dir);
    }

    /// A composite must run entirely against one registry version. Every step
    /// resolves through the snapshot handed to `execute_any_depth`, so removing
    /// a child mid-flight cannot be observed by an in-flight composite.
    #[tokio::test]
    async fn composite_runs_against_one_registry_version() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        yk.execute_any("udo.formula", &[json!({"op":"user.a","p":["x"],"expr":"x+1"})])
            .await
            .unwrap();
        yk.execute_any("udo.composite", &[json!({
            "op":"user.chain","p":["x"],
            "pipe":[["user.a","$a0"],["user.a","$0"],["user.a","$1"]]
        })]).await.unwrap();

        let snapshot = yk.registry_snapshot().await;
        // Redefining the child after the snapshot must not affect a run that
        // started from that snapshot.
        yk.execute_any("udo.formula", &[json!({"op":"user.a","p":["x"],"expr":"x+1000"})])
            .await
            .unwrap();

        let via_snapshot = yk
            .execute_any_depth("user.chain", &[json!(0)], 0, Some(&snapshot))
            .await
            .unwrap();
        assert_eq!(via_snapshot, json!(3.0), "composite saw a torn registry");

        // A run started after the mutation sees the new child, consistently for
        // all three of its steps.
        assert_eq!(yk.execute_any("user.chain", &[json!(0)]).await.unwrap(), json!(3000.0));
        let _ = fs::remove_dir_all(dir);
    }

    /// The R1 regression test proper.
    ///
    /// The two tests above pin the new invariant but use APIs that did not
    /// exist in v1.0.0, so neither could have failed against it. This one runs
    /// only through the public execution path, and it *would* fail on v1.0.0:
    /// there, each composite step took its own read lock, so a redefinition of
    /// the child landing between steps produced a mixed result.
    ///
    /// `user.chain` applies `user.a` three times. A coherent run therefore
    /// yields 3 (child `x+1`) or 3000 (child `x+1000`). Anything else -- 1002,
    /// 2001, 2000, 1001 -- is a torn read, and those are exactly the values a
    /// per-step lookup can produce.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn composite_never_observes_a_torn_registry_under_churn() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        yk.execute_any("udo.formula", &[json!({"op":"user.a","p":["x"],"expr":"x+1"})])
            .await
            .unwrap();
        yk.execute_any("udo.composite", &[json!({
            "op":"user.chain","p":["x"],
            "pipe":[["user.a","$a0"],["user.a","$0"],["user.a","$1"]]
        })]).await.unwrap();

        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let churn = {
            let yk = yk.clone();
            let stop = Arc::clone(&stop);
            tokio::spawn(async move {
                let mut big = false;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let expr = if big { "x+1000" } else { "x+1" };
                    let _ = yk
                        .execute_any(
                            "udo.formula",
                            &[json!({"op":"user.a","p":["x"],"expr":expr})],
                        )
                        .await;
                    big = !big;
                }
            })
        };

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut runs = 0usize;
        while std::time::Instant::now() < deadline {
            let got = yk.execute_any("user.chain", &[json!(0)]).await.unwrap();
            let n = got.as_f64().expect("numeric result");
            assert!(
                n == 3.0 || n == 3000.0,
                "torn registry observed after {runs} runs: composite returned {n}, \
                 which mixes two definitions of user.a"
            );
            runs += 1;
        }

        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        churn.await.unwrap();
        assert!(runs > 100, "stress loop did not run enough iterations: {runs}");
        let _ = fs::remove_dir_all(dir);
    }

    /// R2. Concurrent mutations must all survive: the store gate has to order
    /// both the in-memory publish and the on-disk generation, or writes are
    /// lost. Also asserts the persisted snapshot agrees with memory.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_mutations_do_not_lose_updates() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());

        const N: usize = 24;
        let mut handles = Vec::with_capacity(N);
        for i in 0..N {
            let yk = yk.clone();
            handles.push(tokio::spawn(async move {
                yk.execute_any(
                    "udo.formula",
                    &[json!({"op": format!("user.c{i}"), "p":["x"], "expr":"x*2"})],
                )
                .await
            }));
        }
        for h in handles {
            h.await.unwrap().unwrap();
        }

        let listed = yk.registry_snapshot().await.list();
        assert_eq!(listed.len(), N, "in-memory registry lost concurrent writes: {listed:?}");

        // The newest on-disk generation must contain every operation too.
        let reloaded = Yekaterina::with_store_dir(dir.clone());
        let persisted = reloaded.registry_snapshot().await.list();
        assert_eq!(persisted, listed, "persisted snapshot disagrees with memory");
        let _ = fs::remove_dir_all(dir);
    }

    /// Readers must keep making progress while a writer is persisting, and no
    /// combination of the registry lock and the store gate may deadlock.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn readers_and_writers_make_progress_together() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        yk.execute_any("udo.formula", &[json!({"op":"user.r","p":["x"],"expr":"x*2"})])
            .await
            .unwrap();

        let mut handles = Vec::new();
        for i in 0..8 {
            let yk = yk.clone();
            handles.push(tokio::spawn(async move {
                yk.execute_any(
                    "udo.formula",
                    &[json!({"op": format!("user.w{i}"), "p":["x"], "expr":"x+1"})],
                )
                .await
                .map(|_| ())
            }));
        }
        for _ in 0..64 {
            let yk = yk.clone();
            handles.push(tokio::spawn(async move {
                assert_eq!(yk.execute_any("user.r", &[json!(21)]).await.unwrap(), json!(42.0));
                yk.execute_any("udo.list", &[]).await.map(|_| ())
            }));
        }

        let all = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            futures_join(handles),
        )
        .await
        .expect("readers and writers deadlocked");
        for r in all {
            r.unwrap();
        }
        let _ = fs::remove_dir_all(dir);
    }

    async fn futures_join(
        handles: Vec<tokio::task::JoinHandle<Result<(), &'static str>>>,
    ) -> Vec<Result<(), &'static str>> {
        let mut out = Vec::with_capacity(handles.len());
        for h in handles {
            out.push(h.await.unwrap());
        }
        out
    }

    #[tokio::test]
    async fn expression_eval_is_stateless() {
        let dir = temp_dir();
        let yk = Yekaterina::with_store_dir(dir.clone());
        assert_eq!(yk.execute_any("expr.eval", &[json!({"e":"a*b+1","v":{"a":2,"b":5}})]).await.unwrap(), json!(11.0));
        let _ = fs::remove_dir_all(dir);
    }
}
