use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Value, json};

pub const MAX_PROGRAM_STEPS: usize = 256;
pub const MAX_PROGRAM_DEPTH: usize = 32;
pub const MAX_STEP_ID_BYTES: usize = 64;

#[derive(Debug, Clone, Deserialize)]
struct ProgramSpec {
    steps: Vec<ProgramStep>,
    #[serde(default, rename = "return")]
    return_step: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProgramStep {
    id: String,
    op: String,
    #[serde(default, rename = "a")]
    args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    pub pipe: Vec<Value>,
    pub order: Vec<String>,
    pub all: bool,
}

/// Validate a named v1.3 compute program and compile it to the already-verified
/// v1.2 numeric-reference pipeline representation.
///
/// alpha.1 intentionally reuses the sequential pipeline executor. The graph
/// contract is therefore isolated here and can be handed to a parallel DAG
/// scheduler in alpha.2 without changing the request syntax.
pub fn compile_program(
    value: &Value,
    input: Option<&Value>,
    all: bool,
) -> Result<CompiledProgram, &'static str> {
    let spec: ProgramSpec = serde_json::from_value(value.clone()).map_err(|_| "ARG")?;
    if spec.steps.is_empty() { return Err("ARG"); }
    if spec.steps.len() > MAX_PROGRAM_STEPS { return Err("LIMIT"); }

    let mut by_id = HashMap::with_capacity(spec.steps.len());
    for (index, step) in spec.steps.iter().enumerate() {
        validate_id(&step.id)?;
        if step.id == "input" { return Err("NAME"); }
        if by_id.insert(step.id.as_str(), index).is_some() { return Err("NAME"); }
        let op = step.op.trim();
        if op.is_empty() { return Err("ARG"); }
        if op.to_ascii_lowercase().starts_with("udo.") { return Err("CONTROL"); }
    }

    let target = match spec.return_step.as_deref() {
        Some(id) => *by_id.get(id).ok_or("REF")?,
        None => spec.steps.len() - 1,
    };

    let mut deps = Vec::with_capacity(spec.steps.len());
    for step in &spec.steps {
        let mut names = Vec::new();
        collect_step_refs(&step.args, &mut names);
        names.sort_unstable();
        names.dedup();
        let mut indices = Vec::with_capacity(names.len());
        for name in names {
            let index = *by_id.get(name.as_str()).ok_or("REF")?;
            indices.push(index);
        }
        deps.push(indices);
    }

    let order = stable_topological_order(&deps)?;
    enforce_depth_limit(&deps, &order)?;

    let needed = if all {
        vec![true; spec.steps.len()]
    } else {
        dependency_closure(target, &deps)
    };

    let mut pipe = Vec::with_capacity(if all { spec.steps.len() } else { needed.iter().filter(|x| **x).count() });
    let mut pipe_index_by_id: HashMap<&str, usize> = HashMap::new();
    let mut execution_order = Vec::with_capacity(pipe.capacity());

    for &step_index in &order {
        if !needed[step_index] { continue; }
        let step = &spec.steps[step_index];
        let args = step
            .args
            .iter()
            .map(|v| compile_value(v, input, &pipe_index_by_id))
            .collect::<Result<Vec<_>, _>>()?;
        let op = step.op.trim().to_ascii_lowercase();
        let out_index = pipe.len();
        pipe.push(json!({"op": op, "a": args}));
        pipe_index_by_id.insert(step.id.as_str(), out_index);
        execution_order.push(step.id.clone());
    }

    if pipe.is_empty() { return Err("ARG"); }
    if !all && execution_order.last().map(String::as_str) != Some(spec.steps[target].id.as_str()) {
        // Every selected node is an ancestor of the target, so the target must
        // be the final node after a valid topological sort. Keep this assertion
        // as a checked invariant rather than relying on reasoning alone.
        return Err("CYCLE");
    }

    Ok(CompiledProgram { pipe, order: execution_order, all })
}

fn validate_id(id: &str) -> Result<(), &'static str> {
    if id.is_empty() || id.len() > MAX_STEP_ID_BYTES { return Err("NAME"); }
    let mut bytes = id.bytes();
    let first = bytes.next().ok_or("NAME")?;
    if !(first.is_ascii_alphabetic() || first == b'_') { return Err("NAME"); }
    if !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') { return Err("NAME"); }
    Ok(())
}

fn collect_step_refs(values: &[Value], out: &mut Vec<String>) {
    for value in values { collect_refs_value(value, out); }
}

fn collect_refs_value(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) if s == "$input" || s.starts_with("$input.") => {}
        Value::String(s) if s.starts_with('$') && s.len() > 1 => out.push(s[1..].to_string()),
        Value::Array(xs) => xs.iter().for_each(|v| collect_refs_value(v, out)),
        Value::Object(map) => map.values().for_each(|v| collect_refs_value(v, out)),
        _ => {}
    }
}

fn stable_topological_order(deps: &[Vec<usize>]) -> Result<Vec<usize>, &'static str> {
    let n = deps.len();
    let mut indegree: Vec<usize> = deps.iter().map(Vec::len).collect();
    let mut dependents = vec![Vec::<usize>::new(); n];
    for (node, node_deps) in deps.iter().enumerate() {
        for &dep in node_deps {
            dependents[dep].push(node);
        }
    }

    let mut done = vec![false; n];
    let mut order = Vec::with_capacity(n);
    while order.len() < n {
        let Some(next) = (0..n).find(|&i| !done[i] && indegree[i] == 0) else {
            return Err("CYCLE");
        };
        done[next] = true;
        order.push(next);
        for &child in &dependents[next] {
            indegree[child] = indegree[child].checked_sub(1).ok_or("CYCLE")?;
        }
    }
    Ok(order)
}

fn enforce_depth_limit(deps: &[Vec<usize>], order: &[usize]) -> Result<(), &'static str> {
    let mut depth = vec![1usize; deps.len()];
    for &node in order {
        let d = deps[node]
            .iter()
            .map(|&dep| depth[dep].saturating_add(1))
            .max()
            .unwrap_or(1);
        if d > MAX_PROGRAM_DEPTH { return Err("LIMIT"); }
        depth[node] = d;
    }
    Ok(())
}

fn dependency_closure(target: usize, deps: &[Vec<usize>]) -> Vec<bool> {
    let mut needed = vec![false; deps.len()];
    let mut stack = vec![target];
    while let Some(node) = stack.pop() {
        if needed[node] { continue; }
        needed[node] = true;
        stack.extend(deps[node].iter().copied());
    }
    needed
}

fn compile_value(
    value: &Value,
    input: Option<&Value>,
    pipe_index_by_id: &HashMap<&str, usize>,
) -> Result<Value, &'static str> {
    match value {
        Value::String(s) if s == "$input" => {
            input.cloned().ok_or("REF")?;
            Ok(Value::String("$input".to_string()))
        }
        Value::String(s) if let Some(path) = s.strip_prefix("$input.") => {
            resolve_input_path(input.ok_or("REF")?, path)
        }
        Value::String(s) if s.starts_with('$') && s.len() > 1 => {
            let id = &s[1..];
            let index = pipe_index_by_id.get(id).ok_or("REF")?;
            Ok(Value::String(format!("${index}")))
        }
        Value::Array(xs) => xs
            .iter()
            .map(|x| compile_value(x, input, pipe_index_by_id))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                out.insert(k.clone(), compile_value(v, input, pipe_index_by_id)?);
            }
            Ok(Value::Object(out))
        }
        Value::String(s) if s.starts_with('$') => Err("REF"),
        _ => Ok(value.clone()),
    }
}

fn resolve_input_path(input: &Value, path: &str) -> Result<Value, &'static str> {
    if path.is_empty() { return Err("REF"); }
    let mut current = input;
    for part in path.split('.') {
        if part.is_empty() { return Err("REF"); }
        current = match current {
            Value::Object(map) => map.get(part).ok_or("REF")?,
            Value::Array(xs) => {
                let index = part.parse::<usize>().map_err(|_| "REF")?;
                xs.get(index).ok_or("REF")?
            }
            _ => return Err("REF"),
        };
    }
    Ok(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_named_dependency_to_numeric_pipe_reference() {
        let p = json!({
            "steps": [
                {"id":"a","op":"math.add","a":[1,2]},
                {"id":"b","op":"math.mul","a":["$a",10]}
            ],
            "return":"b"
        });
        let c = compile_program(&p, None, false).unwrap();
        assert_eq!(c.order, vec!["a", "b"]);
        assert_eq!(c.pipe[0], json!({"op":"math.add","a":[1,2]}));
        assert_eq!(c.pipe[1], json!({"op":"math.mul","a":["$0",10]}));
    }

    #[test]
    fn forward_reference_is_reordered_deterministically() {
        let p = json!({"steps":[
            {"id":"c","op":"math.add","a":["$a","$b"]},
            {"id":"a","op":"math.add","a":[1,2]},
            {"id":"b","op":"math.add","a":[3,4]}
        ],"return":"c"});
        let c = compile_program(&p, None, false).unwrap();
        assert_eq!(c.order, vec!["a", "b", "c"]);
        assert_eq!(c.pipe[2], json!({"op":"math.add","a":["$0","$1"]}));
    }

    #[test]
    fn unused_branch_is_pruned_for_single_return() {
        let p = json!({"steps":[
            {"id":"wanted","op":"math.add","a":[1,2]},
            {"id":"unused","op":"math.mul","a":[9,9]},
            {"id":"final","op":"math.mul","a":["$wanted",10]}
        ],"return":"final"});
        let c = compile_program(&p, None, false).unwrap();
        assert_eq!(c.order, vec!["wanted", "final"]);
        assert_eq!(c.pipe.len(), 2);
    }

    #[test]
    fn all_keeps_every_step() {
        let p = json!({"steps":[
            {"id":"a","op":"math.add","a":[1,2]},
            {"id":"unused","op":"math.mul","a":[9,9]},
            {"id":"b","op":"math.mul","a":["$a",10]}
        ],"return":"b"});
        let c = compile_program(&p, None, true).unwrap();
        assert_eq!(c.order, vec!["a", "unused", "b"]);
        assert_eq!(c.pipe.len(), 3);
    }

    #[test]
    fn input_paths_are_inlined_and_whole_input_stays_pipeline_reference() {
        let input = json!({"x":[10,20],"nested":{"v":7}});
        let p = json!({"steps":[
            {"id":"a","op":"math.add","a":["$input.x.0","$input.nested.v"]},
            {"id":"b","op":"debug.echo","a":["$input"]}
        ],"return":"a"});
        let c = compile_program(&p, Some(&input), false).unwrap();
        assert_eq!(c.pipe[0], json!({"op":"math.add","a":[10,7]}));

        let all = compile_program(&p, Some(&input), true).unwrap();
        assert_eq!(all.pipe[1], json!({"op":"debug.echo","a":["$input"]}));
    }

    #[test]
    fn rejects_unknown_reference_duplicate_id_cycle_and_control() {
        let unknown = json!({"steps":[{"id":"a","op":"math.add","a":["$missing",1]}]});
        assert_eq!(compile_program(&unknown, None, false), Err("REF"));

        let duplicate = json!({"steps":[
            {"id":"a","op":"math.add","a":[1,2]},
            {"id":"a","op":"math.mul","a":[1,2]}
        ]});
        assert_eq!(compile_program(&duplicate, None, false), Err("NAME"));

        let cycle = json!({"steps":[
            {"id":"a","op":"math.add","a":["$b",1]},
            {"id":"b","op":"math.add","a":["$a",1]}
        ]});
        assert_eq!(compile_program(&cycle, None, false), Err("CYCLE"));

        let control = json!({"steps":[{"id":"a","op":"udo.list","a":[]}]});
        assert_eq!(compile_program(&control, None, false), Err("CONTROL"));
    }

    #[test]
    fn rejects_missing_input_path() {
        let p = json!({"steps":[{"id":"a","op":"math.add","a":["$input.nope",1]}]});
        assert_eq!(compile_program(&p, Some(&json!({"x":1})), false), Err("REF"));
        assert_eq!(compile_program(&p, None, false), Err("REF"));
    }

    #[test]
    fn enforces_graph_depth_limit() {
        let mut steps = Vec::new();
        for i in 0..=MAX_PROGRAM_DEPTH {
            let args = if i == 0 { json!([1, 1]) } else { json!([format!("$s{}", i - 1), 1]) };
            steps.push(json!({"id":format!("s{i}"),"op":"math.add","a":args}));
        }
        let p = json!({"steps":steps});
        assert_eq!(compile_program(&p, None, false), Err("LIMIT"));
    }

    #[test]
    fn validates_ids_and_step_limit() {
        let bad = json!({"steps":[{"id":"9bad","op":"math.add","a":[1,2]}]});
        assert_eq!(compile_program(&bad, None, false), Err("NAME"));

        let mut steps = Vec::new();
        for i in 0..=MAX_PROGRAM_STEPS {
            steps.push(json!({"id":format!("s{i}"),"op":"math.add","a":[1,2]}));
        }
        assert_eq!(compile_program(&json!({"steps":steps}), None, false), Err("LIMIT"));
    }
}
