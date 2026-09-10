# Yekaterina v1.3 Registry Migration Policy

This document defines the compatibility rules for promoting the staged native
transformer surface onto normal Yekaterina MCP discovery and execution.

The policy exists because v1.2.0 is already a promoted stable release. Native
`xfmr.*` code may be developed and verified in the development repository, but
it must not become discoverable through `yk.find`, inspectable through `yk.spec`
or executable through ordinary `yk.compute` until the migration is explicit and
audited.

## Current staged surface

`full_audit/opcodes_v13_transformer_candidate.json` freezes **15** native
candidate operations:

- 7 material / interpolation / thermal / basic winding-geometry operations;
- 6 winding-field / leakage / AC-loss operations;
- 2 structured candidate evaluation / ranking operations.

The 20 `pack.xfmr.*` scalar formula-pack operations remain importable pack
content. They are not part of this built-in migration and must not be duplicated
as native registry entries without a separate compatibility decision.

## Required v1.3 built-in count

The v1.2 built-in/control registry contains **1,410** operations.

If the current 15 native transformer candidates are promoted without any other
built-in additions, the v1.3 registry must contain exactly **1,425** operations.

The migration should append the 15 new canonical names in candidate-manifest
order. Existing v1.2 names must not be renamed, removed or reordered.

## MCP compatibility invariants

The transformer migration does **not** require a larger model-facing MCP tool
schema. v1.3 should retain:

- exactly 3 MCP tools: `yk.compute`, `yk.find`, `yk.spec`;
- the frozen `ComputeParams`, `FindParams` and `SpecParams` fields;
- the measured schema footprint unless an independently justified protocol
  change is approved;
- the existing error vocabulary when the transformer operations can express
  failures with current codes (`ARG`, `TYPE`, `DOMAIN`, `LIMIT`, `NONFINITE`,
  etc.);
- default worker count 1 and current deterministic scheduling rules.

The crate/release version may move to v1.3.x while the MCP initialize identity
remains deliberately compatibility-gated. Changing the advertised initialize
version is a separate protocol decision, not an automatic consequence of adding
operations behind the same three-tool surface.

## Migration commit requirements

A registry-promotion commit must make all of the following changes together:

1. expose the transformer native modules through the library target;
2. add an `xfmr` dispatcher branch to the execution engine;
3. append the 15 native operation specs to `src/registry.rs` in manifest order;
4. add a frozen v1.2 operation-list artifact if one does not already exist;
5. introduce a v1.3 static audit that proves all 1,410 v1.2 operations remain
   present, unrenamed and in order;
6. raise the expected built-in count to 1,425 only in the v1.3 audit path;
7. keep the historical v1.2 audit and release evidence unchanged as historical
   artifacts;
8. extend full-audit fixtures so every promoted `xfmr.*` operation has an
   executable acceptance case;
9. run all transformer-specific gates plus the complete existing regression
   suite on the migration commit.

A partial commit that registers names without dispatch, or dispatches code
without registry discovery, is not acceptable.

## Staging gate before migration

`scripts/audit_v13_transformer_candidate.py` enforces the inverse state while
promotion has not happened:

- the candidate manifest has the expected 15 names;
- each name exists in its designated candidate source module;
- the v1.2 registry remains exactly 1,410 operations;
- none of the 15 names has leaked into that registry;
- the engine has no `xfmr` dispatch branch yet.

This creates a clean phase boundary. Before migration, accidental exposure is a
failure. During the explicit migration commit, the gate is replaced by the v1.3
promotion audit whose expected state is the opposite.

## Acceptance requirements for promotion

Do not promote the staged native surface unless all of these are green on the
same head commit:

- Transformer Pack workflow;
- Transformer Native workflow;
- Transformer Winding workflow;
- Transformer Candidate workflow;
- full Rust test suite and clippy;
- release build and MCP demo;
- Golden regression corpus;
- Full Capability Audit;
- v1.2 reference verifiers;
- new v1.3 transformer acceptance fixtures;
- new v1.3 static audit with no pending or failed checks.

## Non-duplication rule

Existing built-ins stay authoritative where semantics already exist, including
ideal transformer voltage/current/impedance ratios, resistivity-based resistance,
current density, I-squared-R power and generic efficiency. A new `xfmr.*` native
operation is justified only when structured data, nonlinear material data,
integer geometry, frequency-dependent effects, thermal networks, candidate
reporting or another transformer-specific contract cannot be represented by the
existing canonical operation without changing its meaning.

## Safety and scope boundary

Registry promotion means "available through Yekaterina", not "certified design".
The native surface must continue to avoid claiming automatic compliance with
IEC/IEEE/DOE rules, dielectric clearances, mechanical withstand, factory thermal
rise or other standard-versioned requirements unless a separate versioned source
and validation layer is introduced.
