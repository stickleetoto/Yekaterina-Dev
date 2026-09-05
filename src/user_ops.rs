use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{formula, registry};

pub const MAX_USER_OPS: usize = 4096;
pub const MAX_COMPOSITE_STEPS: usize = 256;
pub const MAX_PACK_OPS: usize = 1024;
pub const SNAPSHOT_VERSION: u32 = 1;
pub const PACK_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormulaOp {
    pub opcode: String,
    pub params: Vec<String>,
    pub expr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositeOp {
    pub opcode: String,
    pub params: Vec<String>,
    pub pipe: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserOp {
    Formula(FormulaOp),
    Composite(CompositeOp),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSnapshot {
    pub v: u32,
    #[serde(default)]
    pub formulas: Vec<FormulaOp>,
    #[serde(default)]
    pub composites: Vec<CompositeOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPack {
    pub v: u32,
    pub name: String,
    pub ops: Vec<PackOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "k")]
pub enum PackOp {
    #[serde(rename = "f")]
    Formula { op: String, p: Vec<String>, expr: String },
    #[serde(rename = "c")]
    Composite { op: String, p: Vec<String>, pipe: Vec<Value> },
}

#[derive(Debug, Default, Clone)]
pub struct UserRegistry {
    formulas: HashMap<String, FormulaOp>,
    composites: HashMap<String, CompositeOp>,
}

impl UserRegistry {
    pub fn define_formula(&mut self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let op = parse_formula_obj(&args[0], NamespacePolicy::UserOnly)?;
        self.ensure_capacity_for(&op.opcode)?;
        self.composites.remove(&op.opcode);
        let opcode = op.opcode.clone();
        self.formulas.insert(opcode.clone(), op);
        Ok(json!(opcode))
    }

    pub fn define_composite(&mut self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let op = parse_composite_obj(&args[0], NamespacePolicy::UserOnly)?;
        self.ensure_capacity_for(&op.opcode)?;
        let existing: HashSet<String> = self.list().into_iter().collect();
        validate_composite_dependencies(&op, Some(&existing))?;

        let opcode = op.opcode.clone();
        let mut candidate = self.clone();
        candidate.formulas.remove(&opcode);
        candidate.composites.insert(opcode.clone(), op);
        candidate.validate_no_cycles()?;
        *self = candidate;
        Ok(json!(opcode))
    }

    pub fn remove(&mut self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let op = args[0].as_str().ok_or("TYPE")?.trim().to_ascii_lowercase();
        if !matches_namespace(&op, NamespacePolicy::UserOrPack) { return Err("NAME"); }
        if self.is_referenced_outside(&HashSet::from([op.clone()]))? { return Err("IN_USE"); }
        let removed = self.formulas.remove(&op).is_some() | self.composites.remove(&op).is_some();
        Ok(json!(removed))
    }

    pub fn uninstall_pack(&mut self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let name = args[0].as_str().ok_or("TYPE")?.trim().to_ascii_lowercase();
        validate_pack_name(&name)?;
        let prefix = format!("pack.{name}.");
        let removed: HashSet<String> = self.list().into_iter().filter(|op| op.starts_with(&prefix)).collect();
        if removed.is_empty() { return Ok(json!(0)); }
        if self.is_referenced_outside(&removed)? { return Err("IN_USE"); }
        let before = self.len();
        self.formulas.retain(|op, _| !removed.contains(op));
        self.composites.retain(|op, _| !removed.contains(op));
        Ok(json!(before - self.len()))
    }

    #[allow(dead_code)]
    pub fn execute_formula(&self, opcode: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
        let spec = self.formulas.get(&opcode.trim().to_ascii_lowercase())?;
        Some(execute_formula_spec(spec, args))
    }

    pub fn lookup(&self, opcode: &str) -> Option<UserOp> {
        let op = opcode.trim().to_ascii_lowercase();
        self.formulas.get(&op).cloned().map(UserOp::Formula)
            .or_else(|| self.composites.get(&op).cloned().map(UserOp::Composite))
    }

    pub fn find(&self, q: &str, limit: usize) -> Vec<String> {
        let q = q.trim().to_ascii_lowercase();
        if q.is_empty() { return Vec::new(); }
        let mut hits: Vec<String> = self.formulas.keys()
            .chain(self.composites.keys())
            .filter(|op| op.contains(&q))
            .cloned()
            .collect();
        hits.sort();
        hits.dedup();
        hits.truncate(limit);
        hits
    }

    pub fn list(&self) -> Vec<String> {
        let mut ops: Vec<String> = self.formulas.keys().chain(self.composites.keys()).cloned().collect();
        ops.sort();
        ops.dedup();
        ops
    }

    pub fn len(&self) -> usize { self.formulas.len() + self.composites.len() }

    pub fn snapshot(&self) -> UserSnapshot {
        let mut formulas: Vec<_> = self.formulas.values().cloned().collect();
        let mut composites: Vec<_> = self.composites.values().cloned().collect();
        formulas.sort_by(|a, b| a.opcode.cmp(&b.opcode));
        composites.sort_by(|a, b| a.opcode.cmp(&b.opcode));
        UserSnapshot { v: SNAPSHOT_VERSION, formulas, composites }
    }

    pub fn from_snapshot(snapshot: UserSnapshot) -> Result<Self, &'static str> {
        if snapshot.v != SNAPSHOT_VERSION { return Err("VERSION"); }
        if snapshot.formulas.len() + snapshot.composites.len() > MAX_USER_OPS { return Err("LIMIT"); }
        let mut out = Self::default();
        for op in snapshot.formulas {
            validate_formula(&op, NamespacePolicy::UserOrPack)?;
            if out.lookup(&op.opcode).is_some() { return Err("DUP"); }
            out.formulas.insert(op.opcode.clone(), op);
        }
        let names: HashSet<String> = snapshot.composites.iter().map(|o| o.opcode.clone())
            .chain(out.formulas.keys().cloned()).collect();
        for op in snapshot.composites {
            validate_composite(&op, NamespacePolicy::UserOrPack)?;
            validate_composite_dependencies(&op, Some(&names))?;
            if out.lookup(&op.opcode).is_some() { return Err("DUP"); }
            out.composites.insert(op.opcode.clone(), op);
        }
        out.validate_no_cycles()?;
        Ok(out)
    }

    pub fn export_pack(&self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let obj = args[0].as_object().ok_or("TYPE")?;
        let name = obj.get("name").and_then(Value::as_str).ok_or("ARG")?.trim().to_ascii_lowercase();
        validate_pack_name(&name)?;

        let selected: Vec<String> = if let Some(xs) = obj.get("ops") {
            xs.as_array().ok_or("TYPE")?.iter()
                .map(|v| v.as_str().map(|s| s.trim().to_ascii_lowercase()).ok_or("TYPE"))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            self.list().into_iter().filter(|op| op.starts_with("user.")).collect()
        };
        if selected.is_empty() || selected.len() > MAX_PACK_OPS { return Err("LIMIT"); }

        let selected_set: HashSet<String> = selected.iter().cloned().collect();
        if selected_set.len() != selected.len() { return Err("DUP"); }
        let mut mapping = HashMap::new();
        for old in &selected {
            if !old.starts_with("user.") { return Err("PACK"); }
            if self.lookup(old).is_none() { return Err("OP"); }
            let tail = old.strip_prefix("user.").ok_or("PACK")?;
            mapping.insert(old.clone(), format!("pack.{name}.{tail}"));
        }

        let mut ops = Vec::with_capacity(selected.len());
        for old in &selected {
            let new_name = mapping.get(old).ok_or("PACK")?.clone();
            match self.lookup(old).ok_or("OP")? {
                UserOp::Formula(f) => ops.push(PackOp::Formula { op: new_name, p: f.params, expr: f.expr }),
                UserOp::Composite(c) => {
                    let pipe = rewrite_pipe_ops(&c.pipe, &mapping, &selected_set)?;
                    ops.push(PackOp::Composite { op: new_name, p: c.params, pipe });
                }
            }
        }
        let pack = OperationPack { v: PACK_VERSION, name, ops };
        serde_json::to_value(pack).map_err(|_| "PACK")
    }

    pub fn import_pack(&mut self, args: &[Value]) -> Result<Value, &'static str> {
        if args.len() != 1 { return Err("ARG"); }
        let pack: OperationPack = serde_json::from_value(args[0].clone()).map_err(|_| "PACK")?;
        if pack.v != PACK_VERSION { return Err("VERSION"); }
        validate_pack_name(&pack.name)?;
        if pack.ops.is_empty() || pack.ops.len() > MAX_PACK_OPS { return Err("LIMIT"); }
        if self.len().saturating_add(pack.ops.len()) > MAX_USER_OPS { return Err("LIMIT"); }

        let prefix = format!("pack.{}.", pack.name);
        let names: HashSet<String> = pack.ops.iter().map(pack_opcode).collect();
        if names.len() != pack.ops.len() { return Err("DUP"); }
        for name in &names {
            if !name.starts_with(&prefix) || !valid_opcode_chars(name) { return Err("NAME"); }
            if self.lookup(name).is_some() { return Err("DUP"); }
        }

        let mut formulas = Vec::new();
        let mut composites = Vec::new();
        for item in pack.ops {
            match item {
                PackOp::Formula { op, p, expr } => {
                    let f = FormulaOp { opcode: op, params: p, expr };
                    validate_formula(&f, NamespacePolicy::PackOnly)?;
                    formulas.push(f);
                }
                PackOp::Composite { op, p, pipe } => {
                    let c = CompositeOp { opcode: op, params: p, pipe };
                    validate_composite(&c, NamespacePolicy::PackOnly)?;
                    validate_composite_dependencies(&c, Some(&names))?;
                    composites.push(c);
                }
            }
        }

        let count = formulas.len() + composites.len();
        let mut candidate = self.clone();
        for f in formulas { candidate.formulas.insert(f.opcode.clone(), f); }
        for c in composites { candidate.composites.insert(c.opcode.clone(), c); }
        candidate.validate_no_cycles()?;
        *self = candidate;
        Ok(json!(count))
    }

    fn is_referenced_outside(&self, removed: &HashSet<String>) -> Result<bool, &'static str> {
        for (owner, composite) in &self.composites {
            if removed.contains(owner) { continue; }
            for item in &composite.pipe {
                let (child, _) = parse_step(item)?;
                if removed.contains(&child) { return Ok(true); }
            }
        }
        Ok(false)
    }

    fn validate_no_cycles(&self) -> Result<(), &'static str> {
        fn visit(
            name: &str,
            registry: &UserRegistry,
            visiting: &mut HashSet<String>,
            done: &mut HashSet<String>,
        ) -> Result<(), &'static str> {
            if done.contains(name) { return Ok(()); }
            if !visiting.insert(name.to_string()) { return Err("CYCLE"); }
            if let Some(op) = registry.composites.get(name) {
                for item in &op.pipe {
                    let (child, _) = parse_step(item)?;
                    if registry.composites.contains_key(&child) {
                        visit(&child, registry, visiting, done)?;
                    }
                }
            }
            visiting.remove(name);
            done.insert(name.to_string());
            Ok(())
        }

        let mut visiting = HashSet::new();
        let mut done = HashSet::new();
        for name in self.composites.keys() {
            visit(name, self, &mut visiting, &mut done)?;
        }
        Ok(())
    }

    fn ensure_capacity_for(&self, opcode: &str) -> Result<(), &'static str> {
        if self.lookup(opcode).is_none() && self.len() >= MAX_USER_OPS { Err("LIMIT") } else { Ok(()) }
    }
}

pub fn resolve_composite_args(raw: &[Value], call_args: &[Value], results: &[Value]) -> Result<Vec<Value>, &'static str> {
    raw.iter().map(|v| resolve_composite_value(v, call_args, results)).collect()
}

fn resolve_composite_value(v: &Value, call_args: &[Value], results: &[Value]) -> Result<Value, &'static str> {
    match v {
        Value::String(s) if s.starts_with("$a") && s.len() > 2 && s[2..].bytes().all(|b| b.is_ascii_digit()) => {
            let idx = s[2..].parse::<usize>().map_err(|_| "REF")?;
            call_args.get(idx).cloned().ok_or("REF")
        }
        Value::String(s) if s.starts_with('$') && s.len() > 1 && s[1..].bytes().all(|b| b.is_ascii_digit()) => {
            let idx = s[1..].parse::<usize>().map_err(|_| "REF")?;
            results.get(idx).cloned().ok_or("REF")
        }
        Value::Array(xs) => xs.iter().map(|x| resolve_composite_value(x, call_args, results)).collect::<Result<Vec<_>, _>>().map(Value::Array),
        Value::Object(obj) => {
            let mut out = serde_json::Map::with_capacity(obj.len());
            for (k, x) in obj { out.insert(k.clone(), resolve_composite_value(x, call_args, results)?); }
            Ok(Value::Object(out))
        }
        _ => Ok(v.clone()),
    }
}

pub fn parse_step(item: &Value) -> Result<(String, Vec<Value>), &'static str> {
    if let Some(obj) = item.as_object() {
        let op = obj.get("op").and_then(Value::as_str).ok_or("ARG")?.trim().to_ascii_lowercase();
        let args = obj.get("a").and_then(Value::as_array).cloned().unwrap_or_default();
        return Ok((op, args));
    }
    if let Some(xs) = item.as_array() {
        let (head, tail) = xs.split_first().ok_or("ARG")?;
        let op = head.as_str().ok_or("TYPE")?.trim().to_ascii_lowercase();
        return Ok((op, tail.to_vec()));
    }
    Err("TYPE")
}

pub fn execute_formula_spec(spec: &FormulaOp, args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != spec.params.len() { return Err("ARG"); }
    let mut vars = HashMap::with_capacity(args.len());
    for (name, value) in spec.params.iter().zip(args) {
        let n = value.as_f64().ok_or("TYPE")?;
        vars.insert(name.clone(), n);
    }
    let result = formula::eval(&spec.expr, &vars)?;
    Ok(json!(result))
}

#[derive(Clone, Copy)]
enum NamespacePolicy { UserOnly, PackOnly, UserOrPack }

fn parse_formula_obj(v: &Value, ns: NamespacePolicy) -> Result<FormulaOp, &'static str> {
    let obj = v.as_object().ok_or("TYPE")?;
    let op = FormulaOp {
        opcode: obj.get("op").and_then(Value::as_str).ok_or("ARG")?.trim().to_ascii_lowercase(),
        params: parse_params(obj.get("p").ok_or("ARG")?)?,
        expr: obj.get("expr").and_then(Value::as_str).ok_or("ARG")?.trim().to_string(),
    };
    validate_formula(&op, ns)?;
    Ok(op)
}

fn parse_composite_obj(v: &Value, ns: NamespacePolicy) -> Result<CompositeOp, &'static str> {
    let obj = v.as_object().ok_or("TYPE")?;
    let op = CompositeOp {
        opcode: obj.get("op").and_then(Value::as_str).ok_or("ARG")?.trim().to_ascii_lowercase(),
        params: parse_params(obj.get("p").ok_or("ARG")?)?,
        pipe: obj.get("pipe").and_then(Value::as_array).ok_or("ARG")?.clone(),
    };
    validate_composite(&op, ns)?;
    Ok(op)
}

fn parse_params(v: &Value) -> Result<Vec<String>, &'static str> {
    let params = v.as_array().ok_or("TYPE")?;
    if params.len() > formula::MAX_PARAMS { return Err("LIMIT"); }
    let mut names = Vec::with_capacity(params.len());
    for p in params {
        let name = p.as_str().ok_or("TYPE")?.trim().to_string();
        validate_ident(&name)?;
        if names.iter().any(|x| x == &name) { return Err("ARG"); }
        names.push(name);
    }
    Ok(names)
}

fn validate_formula(op: &FormulaOp, ns: NamespacePolicy) -> Result<(), &'static str> {
    validate_opcode(&op.opcode, ns)?;
    if op.expr.is_empty() || op.expr.len() > formula::MAX_EXPR_LEN { return Err("LIMIT"); }
    if op.params.len() > formula::MAX_PARAMS { return Err("LIMIT"); }
    for p in &op.params { validate_ident(p)?; }
    let zeros: HashMap<String, f64> = op.params.iter().map(|n| (n.clone(), 0.0)).collect();
    match formula::eval(&op.expr, &zeros) {
        Ok(_) | Err("DIV0") | Err("NONFINITE") => Ok(()),
        Err(e) => Err(e),
    }
}

fn validate_composite(op: &CompositeOp, ns: NamespacePolicy) -> Result<(), &'static str> {
    validate_opcode(&op.opcode, ns)?;
    if op.params.len() > formula::MAX_PARAMS { return Err("LIMIT"); }
    for p in &op.params { validate_ident(p)?; }
    if op.pipe.is_empty() || op.pipe.len() > MAX_COMPOSITE_STEPS { return Err("LIMIT"); }
    for (i, item) in op.pipe.iter().enumerate() {
        let (child, raw) = parse_step(item)?;
        if child == op.opcode { return Err("CYCLE"); }
        validate_refs(&raw, op.params.len(), i)?;
    }
    Ok(())
}

fn validate_composite_dependencies(op: &CompositeOp, allowed_dynamic: Option<&HashSet<String>>) -> Result<(), &'static str> {
    for item in &op.pipe {
        let (child, _) = parse_step(item)?;
        if child.starts_with("udo.") { return Err("CONTROL"); }
        if registry::resolve(&child).is_some() { continue; }
        if let Some(names) = allowed_dynamic {
            if names.contains(&child) { continue; }
        } else if matches_namespace(&child, NamespacePolicy::UserOrPack) {
            // Runtime registry can resolve existing or future user operations.
            continue;
        }
        return Err("OP");
    }
    Ok(())
}

fn validate_refs(values: &[Value], param_count: usize, prior_count: usize) -> Result<(), &'static str> {
    for v in values { validate_ref_value(v, param_count, prior_count)?; }
    Ok(())
}

fn validate_ref_value(v: &Value, param_count: usize, prior_count: usize) -> Result<(), &'static str> {
    match v {
        Value::String(s) if s.starts_with("$a") && s.len() > 2 && s[2..].bytes().all(|b| b.is_ascii_digit()) => {
            let idx = s[2..].parse::<usize>().map_err(|_| "REF")?;
            if idx >= param_count { return Err("REF"); }
        }
        Value::String(s) if s.starts_with('$') && s.len() > 1 && s[1..].bytes().all(|b| b.is_ascii_digit()) => {
            let idx = s[1..].parse::<usize>().map_err(|_| "REF")?;
            if idx >= prior_count { return Err("REF"); }
        }
        Value::Array(xs) => for x in xs { validate_ref_value(x, param_count, prior_count)?; },
        Value::Object(obj) => for x in obj.values() { validate_ref_value(x, param_count, prior_count)?; },
        Value::String(s) if s.starts_with('$') => return Err("REF"),
        _ => {}
    }
    Ok(())
}

fn rewrite_pipe_ops(pipe: &[Value], mapping: &HashMap<String, String>, selected: &HashSet<String>) -> Result<Vec<Value>, &'static str> {
    let mut out = Vec::with_capacity(pipe.len());
    for item in pipe {
        let (op, args) = parse_step(item)?;
        let rewritten = if let Some(new) = mapping.get(&op) {
            new.clone()
        } else if op.starts_with("user.") {
            if selected.contains(&op) { return Err("PACK"); }
            return Err("PACK");
        } else if op.starts_with("pack.") {
            return Err("PACK");
        } else {
            op
        };
        let mut row = Vec::with_capacity(args.len() + 1);
        row.push(json!(rewritten));
        row.extend(args);
        out.push(Value::Array(row));
    }
    Ok(out)
}

fn pack_opcode(item: &PackOp) -> String {
    match item {
        PackOp::Formula { op, .. } | PackOp::Composite { op, .. } => op.trim().to_ascii_lowercase(),
    }
}

fn validate_opcode(op: &str, policy: NamespacePolicy) -> Result<(), &'static str> {
    if op.len() > 128 || !matches_namespace(op, policy) || !valid_opcode_chars(op) { return Err("NAME"); }
    Ok(())
}

fn matches_namespace(op: &str, policy: NamespacePolicy) -> bool {
    match policy {
        NamespacePolicy::UserOnly => op.starts_with("user."),
        NamespacePolicy::PackOnly => op.starts_with("pack."),
        NamespacePolicy::UserOrPack => op.starts_with("user.") || op.starts_with("pack."),
    }
}

fn valid_opcode_chars(op: &str) -> bool {
    !op.ends_with('.') && !op.contains("..") && op.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_'))
}

fn validate_ident(name: &str) -> Result<(), &'static str> {
    let mut it = name.bytes();
    let Some(first) = it.next() else { return Err("NAME"); };
    if !(first.is_ascii_alphabetic() || first == b'_') { return Err("NAME"); }
    if !it.all(|b| b.is_ascii_alphanumeric() || b == b'_') { return Err("NAME"); }
    Ok(())
}

fn validate_pack_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() || name.len() > 48 { return Err("NAME"); }
    if !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') { return Err("NAME"); }
    Ok(())
}
