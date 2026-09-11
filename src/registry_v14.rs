//! v1.4 aggregate registry and agent-friendly discovery.
//!
//! v1.3 remains the frozen 1,425-operation baseline. This layer adds a very
//! small core-math surface and improves natural-language discovery without
//! changing the MCP tool schema.

use crate::registry_v13 as legacy;

pub use legacy::{OperationSource, OperationSpec};

pub const V13_BUILTIN_COUNT: usize = 1425;
pub const V14_MATH_COUNT: usize = 2;
pub const BUILTIN_COUNT: usize = V13_BUILTIN_COUNT + V14_MATH_COUNT;

pub const V14_OPERATIONS: &[OperationSpec] = &[
    op(
        "alg.linear_root",
        &["linear_root", "solve_linear", "linear_equation"],
        &["a", "b"],
        "number",
        "solve the linear equation a*x + b = 0 for x",
    ),
    op(
        "linalg.solve",
        &["linear_solve", "solve_linear_system", "matrix_solve"],
        &["matrix", "rhs"],
        "number[]",
        "solve a square linear system A*x=b using pivoted Gaussian elimination",
    ),
];

#[derive(Debug, Clone, Copy)]
pub struct OperationsView;

pub const OPERATIONS: OperationsView = OperationsView;

type OperationsIter = std::iter::Chain<
    <legacy::OperationsView as IntoIterator>::IntoIter,
    std::slice::Iter<'static, OperationSpec>,
>;

impl OperationsView {
    pub const fn len(&self) -> usize { BUILTIN_COUNT }
    pub const fn is_empty(&self) -> bool { false }

    pub fn iter(&self) -> OperationsIter {
        legacy::OPERATIONS.into_iter().chain(V14_OPERATIONS.iter())
    }
}

impl IntoIterator for OperationsView {
    type Item = &'static OperationSpec;
    type IntoIter = OperationsIter;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

const fn op(
    opcode: &'static str,
    aliases: &'static [&'static str],
    args: &'static [&'static str],
    returns: &'static str,
    summary: &'static str,
) -> OperationSpec {
    OperationSpec {
        opcode,
        aliases,
        args,
        returns,
        summary,
        source: OperationSource::BuiltIn,
    }
}

fn v14_resolve(name: &str) -> Option<&'static OperationSpec> {
    let raw = name.trim();
    V14_OPERATIONS
        .iter()
        .find(|spec| spec.opcode == raw || spec.aliases.contains(&raw))
        .or_else(|| {
            V14_OPERATIONS.iter().find(|spec| {
                spec.opcode.eq_ignore_ascii_case(raw)
                    || spec.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(raw))
            })
        })
}

/// Preserve every v1.3 canonical/alias ownership rule. New aliases only take
/// effect when the frozen v1.3 registry has no owner for the supplied name.
pub fn resolve(name: &str) -> Option<&'static OperationSpec> {
    legacy::resolve(name).or_else(|| v14_resolve(name))
}

const SEMANTIC_HINTS: &[(&str, &str)] = &[
    ("solve linear equation", "alg.linear_root"),
    ("linear equation", "alg.linear_root"),
    ("ax plus b", "alg.linear_root"),
    ("system of equations", "linalg.solve"),
    ("simultaneous equations", "linalg.solve"),
    ("solve linear system", "linalg.solve"),
    ("matrix solve", "linalg.solve"),
    ("quadratic equation", "alg.quadratic_roots"),
    ("solve quadratic", "alg.quadratic_roots"),
    ("greatest common divisor", "alg.gcd_many"),
    ("least common multiple", "alg.lcm_many"),
    ("prime factorization", "alg.prime_factors"),
    ("matrix inverse", "mat.inverse"),
    ("invert matrix", "mat.inverse"),
    ("matrix determinant", "mat.det"),
];

fn tokens(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn contains_all(haystack: &[String], needles: &[String]) -> bool {
    !needles.is_empty() && needles.iter().all(|needle| haystack.iter().any(|token| token == needle))
}

fn searchable_tokens(spec: &OperationSpec) -> Vec<String> {
    let mut out = tokens(spec.opcode);
    for alias in spec.aliases { out.extend(tokens(alias)); }
    out.extend(tokens(spec.summary));
    out
}

fn push_unique(out: &mut Vec<&'static OperationSpec>, spec: &'static OperationSpec, limit: usize) {
    if out.len() < limit && !out.iter().any(|existing| existing.opcode == spec.opcode) {
        out.push(spec);
    }
}

fn semantic_targets(query_tokens: &[String]) -> Vec<&'static OperationSpec> {
    let mut out = Vec::new();
    for (phrase, target) in SEMANTIC_HINTS {
        let hint_tokens = tokens(phrase);
        if contains_all(query_tokens, &hint_tokens) {
            if let Some(spec) = resolve(target) {
                if !out.iter().any(|existing: &&OperationSpec| existing.opcode == spec.opcode) {
                    out.push(spec);
                }
            }
        }
    }
    out
}

fn v14_direct_matches(query: &str) -> Vec<(u8, &'static OperationSpec)> {
    let q = query.trim().to_ascii_lowercase();
    let q_tokens = tokens(&q);
    let mut scored = Vec::new();
    for spec in V14_OPERATIONS {
        let opcode = spec.opcode.to_ascii_lowercase();
        let aliases = spec.aliases.iter().map(|a| a.to_ascii_lowercase()).collect::<Vec<_>>();
        let summary = spec.summary.to_ascii_lowercase();
        let score = if opcode == q || aliases.iter().any(|alias| alias == &q) {
            0
        } else if opcode.starts_with(&q) || aliases.iter().any(|alias| alias.starts_with(&q)) {
            1
        } else if opcode.contains(&q) || summary.contains(&q) || aliases.iter().any(|alias| alias.contains(&q)) {
            2
        } else if contains_all(&searchable_tokens(spec), &q_tokens) {
            3
        } else {
            continue;
        };
        scored.push((score, spec));
    }
    scored.sort_by_key(|(score, spec)| (*score, spec.opcode));
    scored
}

/// v1.4 keeps exact lookup deterministic while adding a semantic fallback for
/// multi-word agent queries. The MCP request and response shape is unchanged.
pub fn search(query: &str, limit: usize) -> Vec<&'static OperationSpec> {
    if limit == 0 { return Vec::new(); }
    let q = query.trim();
    if q.is_empty() { return Vec::new(); }

    let q_tokens = tokens(q);
    let mut out: Vec<&'static OperationSpec> = Vec::new();

    // Exact canonical/alias ownership remains the strongest signal.
    if let Some(spec) = resolve(q) { push_unique(&mut out, spec, limit); }

    // Curated intent bridges cover common agent phrasing that opcode substring
    // search cannot express without adding more MCP schema or duplicate ops.
    for spec in semantic_targets(&q_tokens) { push_unique(&mut out, spec, limit); }

    // Preserve the mature v1.3 search path and ordering for ordinary queries.
    for spec in legacy::search(q, limit) { push_unique(&mut out, spec, limit); }

    // Direct matching for the two v1.4 operations.
    for (_, spec) in v14_direct_matches(q) { push_unique(&mut out, spec, limit); }

    // Last-resort token matching lets agents use short natural-language phrases
    // such as "quadratic equation" even when those words are not contiguous in
    // an opcode/summary. Require all tokens to avoid noisy one-word matches.
    if q_tokens.len() >= 2 && out.len() < limit {
        let mut fallback = OPERATIONS
            .iter()
            .filter(|spec| !out.iter().any(|existing| existing.opcode == spec.opcode))
            .filter(|spec| contains_all(&searchable_tokens(spec), &q_tokens))
            .collect::<Vec<_>>();
        fallback.sort_by_key(|spec| spec.opcode);
        for spec in fallback { push_unique(&mut out, spec, limit); }
    }

    out
}

pub fn capability_code(opcode: &str) -> &'static str {
    if matches!(opcode, "alg.linear_root" | "linalg.solve") { "d" }
    else { legacy::capability_code(opcode) }
}

pub fn cost_code(opcode: &str) -> &'static str {
    match opcode {
        "alg.linear_root" => "1",
        "linalg.solve" => "h",
        _ => legacy::cost_code(opcode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_count_is_explicit() {
        assert_eq!(legacy::OPERATIONS.len(), V13_BUILTIN_COUNT);
        assert_eq!(V14_OPERATIONS.len(), V14_MATH_COUNT);
        assert_eq!(OPERATIONS.len(), 1427);
        assert_eq!(OPERATIONS.iter().count(), 1427);
    }

    #[test]
    fn v13_alias_ownership_is_preserved() {
        assert_eq!(resolve("add").unwrap().opcode, "math.add");
        assert_eq!(search("add", 5)[0].opcode, "math.add");
        assert_eq!(resolve("xfmr.skin_depth").unwrap().opcode, "xfmr.skin_depth");
    }

    #[test]
    fn new_math_aliases_resolve() {
        assert_eq!(resolve("linear_equation").unwrap().opcode, "alg.linear_root");
        assert_eq!(resolve("matrix_solve").unwrap().opcode, "linalg.solve");
    }

    #[test]
    fn natural_language_search_maps_to_useful_operations() {
        assert_eq!(search("solve linear equation", 5)[0].opcode, "alg.linear_root");
        assert_eq!(search("system of equations", 5)[0].opcode, "linalg.solve");
        assert_eq!(search("quadratic equation", 5)[0].opcode, "alg.quadratic_roots");
        assert_eq!(search("greatest common divisor", 5)[0].opcode, "alg.gcd_many");
        assert_eq!(search("matrix inverse", 5)[0].opcode, "mat.inverse");
    }
}
