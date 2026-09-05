#[path = "../src/registry.rs"]
mod registry;

#[test]
fn alias_resolves() {
    assert_eq!(registry::resolve("avg").unwrap().opcode, "stat.mean");
}

#[test]
fn lazy_search_is_small() {
    let hits = registry::search("mean", 5);
    assert_eq!(hits[0].opcode, "stat.mean");
    assert!(hits.len() <= 5);
}

#[test]
fn exact_alias_search_agrees_with_resolve() {
    for alias in ["mean", "sin", "power", "std", "variance"] {
        let resolved = registry::resolve(alias).expect("alias should resolve");
        let hits = registry::search(alias, 5);
        assert!(!hits.is_empty(), "search should return an exact alias hit for {alias}");
        assert_eq!(hits[0].opcode, resolved.opcode, "search/resolve disagree for {alias}");
    }
}

#[test]
fn alpha10_index_resolves_deep_tail_family_and_alias() {
    assert_eq!(registry::resolve("data.address_space").unwrap().opcode, "data.address_space");
    assert_eq!(registry::resolve("optics_lens_power_diopter").unwrap().opcode, "optics.lens_power_diopter");
}

#[test]
fn alpha10_family_search_stays_bounded() {
    let hits = registry::search("thermo.", 5);
    assert_eq!(hits.len(), 5);
    assert!(hits.iter().all(|x| x.opcode.starts_with("thermo.")));
}

#[test]
fn alpha10_registry_has_at_least_1050_operations() {
    assert!(registry::OPERATIONS.len() >= 1050);
}

#[test]
fn alpha11_registry_has_at_least_1130_operations() {
    assert!(registry::OPERATIONS.len() >= 1130);
}

#[test]
fn alpha11_trust_families_are_discoverable() {
    for family in ["verify.", "frame.", "curve.", "predicate."] {
        let hits = registry::search(family, 5);
        assert_eq!(hits.len(), 5, "family {family}");
        assert!(hits.iter().all(|x| x.opcode.starts_with(family)), "family {family}");
    }
}

#[test]
fn alpha11_high_value_aliases_resolve() {
    assert_eq!(registry::resolve("converge").unwrap().opcode, "verify.convergence");
    assert_eq!(registry::resolve("circle_in_polygon").unwrap().opcode, "predicate.circle_in_polygon");
    assert_eq!(registry::resolve("curve_audit").unwrap().opcode, "curve.audit");
}

#[test]
fn alpha12_hotfix6_case_distinct_canonical_opcodes_resolve_exactly() {
    assert_eq!(registry::resolve("data.throughput_bps").unwrap().opcode, "data.throughput_bps");
    assert_eq!(registry::resolve("data.throughput_Bps").unwrap().opcode, "data.throughput_Bps");
    assert_eq!(registry::resolve("data_throughput_bps").unwrap().opcode, "data.throughput_bps");
    assert_eq!(registry::resolve("data_throughput_Bps").unwrap().opcode, "data.throughput_Bps");
}

#[test]
fn alpha12_hotfix6_case_distinct_search_prefers_exact_identity() {
    assert_eq!(registry::search("data.throughput_bps", 2)[0].opcode, "data.throughput_bps");
    assert_eq!(registry::search("data.throughput_Bps", 2)[0].opcode, "data.throughput_Bps");
}

#[test]
fn alpha12_hotfix9_return_specs_match_runtime_shape() {
    assert_eq!(registry::resolve("prob.normalize_weights").unwrap().returns, "number[]");
    assert_eq!(registry::resolve("color.contrast_ratio").unwrap().returns, "number");
    assert_eq!(registry::resolve("color.relative_luminance").unwrap().returns, "number");
}
