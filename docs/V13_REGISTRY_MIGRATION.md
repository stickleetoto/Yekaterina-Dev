# Yekaterina v1.3 Registry Migration

This document records the compatibility contract and implemented architecture for promoting the 15 native transformer operations onto normal Yekaterina MCP discovery and execution.

## Result

The migration is implemented in the development tree. The live development registry contains exactly **1,425** built-in/control operations: the frozen v1.2 set of 1,410 plus the 15 transformer-native operations from `full_audit/opcodes_v13_transformer_candidate.json`.

All 15 are now discoverable through `yk.find`, inspectable through `yk.spec`, and executable through ordinary `yk.compute`.

The 20 `pack.xfmr.*` scalar formula-pack operations remain importable pack content and are not duplicated as built-ins.

## Preservation-first architecture

The original policy proposed appending the 15 entries directly to `src/registry.rs`. During implementation, a stronger preservation strategy was adopted: the promoted v1.2 registry and engine source files remain unchanged historical baselines, while v1.3 uses explicit aggregate shims.

- `src/registry.rs` — frozen v1.2 1,410-operation registry.
- `src/registry_v13.rs` — aggregate v1.3 catalog composed from the legacy registry followed by the 15 transformer specs.
- `src/engine.rs` — frozen v1.2 execution engine.
- `src/engine_v13.rs` — transformer dispatch shim; legacy operations delegate to the v1.2 engine.
- `src/lib.rs` — exposes the historical modules as `registry_v12` / `engine_v12` and the v1.3 aggregate modules as the live `registry` / `engine` API.

This architecture satisfies the original compatibility intent without rewriting the 1,410-entry historical source snapshots.

## Built-in count and ordering

The v1.3 development registry must contain exactly **1,425** operations.

The aggregate ordering is:

1. all 1,410 v1.2 canonical names, unchanged and in their original order;
2. the 15 `xfmr.*` names in transformer manifest order.

`registry_v13::OPERATIONS` exposes a read-only aggregate view so existing safety/scheduler tests can continue iterating the full static catalog without copying the legacy registry.

## MCP compatibility invariants

The transformer migration keeps the model-facing protocol small:

- exactly 3 MCP tools: `yk.compute`, `yk.find`, `yk.spec`;
- unchanged request parameter structures;
- unchanged compatibility-gated MCP initialize identity;
- unchanged error vocabulary where existing errors (`ARG`, `TYPE`, `DOMAIN`, `LIMIT`, `NONFINITE`, etc.) already express transformer failures;
- default worker count 1 and existing deterministic scheduling rules.

Crate/release metadata is a separate promotion decision. The development migration may be verified before the stable distribution is formally released as `1.3.x`.

## Promoted transformer surface

The 15 native operations are grouped as follows.

### Material, geometry and thermal

- `xfmr.bh_field_strength`
- `xfmr.magnetizing_current_bh`
- `xfmr.core_loss_specific_interp`
- `xfmr.core_loss_from_grid`
- `xfmr.winding_geometry`
- `xfmr.skin_depth`
- `xfmr.thermal_two_node`

### Winding field and AC loss

- `xfmr.rogowski_factor`
- `xfmr.effective_leakage_height`
- `xfmr.leakage_inductance_concentric`
- `xfmr.leakage_reactance_concentric`
- `xfmr.dowell_foil_ac_factor`
- `xfmr.harmonic_copper_loss`

### Candidate decision layer

- `xfmr.candidate_evaluate`
- `xfmr.candidate_rank`

## Verification contract

`scripts/static_audit_v13.py` proves the migration structure and preservation invariants, including:

- frozen v1.2 registry/engine/model/server artifacts remain preserved;
- the v1.2 operation manifest remains 1,410 unique entries in order;
- the transformer manifest is exactly 15 unique registered entries and disjoint from v1.2;
- the v1.3 transformer specs match manifest order;
- aggregate count is exactly 1,425;
- all 15 operations are represented by the v1.3 dispatcher;
- runtime fixtures cover all 15 operations;
- the MCP tool surface remains exactly three tools.

`full_audit/fixtures_v13_transformer.json` provides executable inputs for the promoted transformer surface. `scripts/verify_v13_transformer_runtime.py` launches the release executable over real MCP stdio and verifies transformer discovery, specs and execution rather than relying only on source inspection.

## Verified migration head

Before promotion to `main`, the migration branch must pass on the same effective head:

- v1.3 static audit;
- lexical and operation-manifest checks;
- complete Rust test suite;
- complete clippy suite;
- release build;
- reviewer-facing MCP demo;
- v1.3 transformer runtime verifier;
- existing v1.2 operation/reference verifiers;
- Golden regression corpus;
- frozen v1.2 Full Capability Audit;
- benchmark invariants;
- dedicated transformer workflows.

The migration branch reached this state before PR promotion.

## Non-duplication rule

Existing built-ins remain authoritative where semantics already exist, including ideal transformer voltage/current/impedance relationships, resistivity-based resistance, current density, I-squared-R power and generic efficiency. A native `xfmr.*` operation is justified only when the transformer-specific contract requires structured material data, nonlinear curves, integer geometry, frequency-dependent effects, thermal networks or candidate-level decision reporting.

## Safety and scope boundary

Registry promotion means **available through Yekaterina**, not **certified transformer design**. These operations remain preliminary engineering calculation primitives and do not claim automatic IEC/IEEE/DOE compliance, dielectric-clearance adequacy, mechanical short-circuit withstand, guaranteed temperature rise or factory acceptance. Those require separately versioned source data and independent validation.
