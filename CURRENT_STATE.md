# CURRENT_STATE.md

> Active-development snapshot for `stickleetoto/Yekaterina-Dev`.

## Current milestone

**v1.2.0 is promoted stable; v1.3 transformer-design development now has a frozen 15-op native candidate surface.**

The verified v1.2.0 source was promoted from commit
`9019194af02f7b9cbe72c5a232e753e236a84b4f` to the stable distribution repository.
The v1.2 built-in operation surface remains frozen while post-release work continues
in this repository.

| | |
|---|---|
| Stable crate line | `1.2.0` |
| Advertised MCP version | `1.0.0` (deliberately frozen and hash-gated) |
| Registered built-in/control opcodes | **1,410** |
| Staged native transformer opcodes | **15** (not yet registered) |
| Importable transformer formula-pack ops | **20** |
| MCP tools | **3** — `yk.compute`, `yk.find`, `yk.spec` |
| Schema footprint | **412 tokens / 1,725 bytes** |
| Error codes | **30** |
| Golden corpus | **527/527** |
| Full Capability Audit | **1,410/1,410** |
| Rust edition / toolchain | 2024 / pinned 1.98.0 |
| Default workers | 1 |

## Stable v1.2 verification record

The source promoted to v1.2.0 passed the full release path:

- `static_audit_v12`: **24 pass / 0 fail**;
- lexical, operation-manifest, Golden-manifest, and Full-Audit validators: PASS;
- **386** Rust test executions and clippy exit 0;
- `verify_v12_operations.py`: **164** independent checks;
- `verify_statistics.py`: **961** values against SciPy, NumPy, and mpmath;
- `verify_multiplicity.py`: **577** checks against statsmodels/SciPy plus independent definitions;
- Golden **527/527**;
- Full Capability Audit **1,410/1,410**;
- user-facing MCP demo: **DEMO PASS**;
- mutation gates: **6/6 caught**.

The stable distribution is `stickleetoto/Yekaterina`. Development changes in this
repository do not change that stable release until a later promotion is explicit.

## v1.3 transformer-design candidate

The transformer work now has four implemented layers while the v1.2 registry stays
frozen.

1. **Formula pack** — 20 importable `pack.xfmr.*` manufacturing calculations for
   core area, volts/turn, turns, flux back-checks, magnetic quantities, conductor
   sizing, hot resistance, core loss, window fill, rated current, regulation, and
   short-circuit impedance.
2. **Native material / geometry / thermal core** — B-H interpolation, magnetizing
   current from B-H data, manufacturer core-loss-grid interpolation, integer
   winding geometry, conductor skin depth, and a caller-parameterized thermal
   network.
3. **Native winding-field core** — Rogowski correction, preliminary concentric
   leakage inductance/reactance, Dowell foil/layer AC resistance factor, and
   harmonic copper-loss aggregation.
4. **Native candidate evaluator** — explicit numeric constraint reporting,
   normalized 0..1 multi-objective desirability, and deterministic feasible-first
   ranking for batches of proposed designs.

Existing `elec.transformer_voltage`, `elec.transformer_current`, and
`elec.transformer_impedance` remain authoritative for ideal turns-ratio
relationships. The v1.3 work does not duplicate those operations.

## Staged native contract

`full_audit/opcodes_v13_transformer_candidate.json` freezes the current native
candidate contract at **15** `xfmr.*` operations:

- 7 material/interpolation/thermal/basic-geometry operations;
- 6 winding-field/leakage/AC-loss operations;
- 2 candidate evaluation/ranking operations.

`scripts/audit_v13_transformer_candidate.py` proves this contract is present in
its owner modules while also proving that none of those names has leaked into the
frozen 1,410-op v1.2 registry and that the engine has no `xfmr` dispatcher branch
before the explicit migration.

The native functions are therefore implemented and independently verified but
are not yet discoverable through `yk.find`, inspectable through `yk.spec`, or
reachable by ordinary `yk.compute` dispatch.

## Candidate evaluation layer

`xfmr.candidate_evaluate` consumes one metric object plus explicit hard constraints
and objectives. It returns pass/fail evidence, every violation, per-objective
0..1 desirability, and a weighted normalized score.

`xfmr.candidate_rank` evaluates many candidates under one shared policy and sorts
with deterministic rules: feasible designs first, higher score next, original
index as the final tie-breaker.

This separates physical calculation, admissibility and design preference. Limits,
weights and objective bands remain caller data; the engine does not invent a
regulatory or manufacturing policy.

## Transformer verification gates

Dedicated GitHub Actions workflows cover:

- the importable formula pack and its independent Python verifier;
- the native transformer material/geometry/thermal tests and clippy;
- the winding-field tests and clippy;
- the candidate evaluator, staged-surface audit and clippy;
- the unchanged full v1.2 CI regression path.

## Registry migration policy

`docs/V13_REGISTRY_MIGRATION.md` now defines the explicit promotion contract.
If the current 15 native candidates are promoted without additional built-ins,
the v1.3 registry target is **1,425** operations.

The migration must append the 15 names in manifest order, expose the native
modules through the library, add `xfmr` engine dispatch, introduce a v1.3 audit,
preserve all 1,410 v1.2 names and ordering, add executable full-audit fixtures,
and keep the three-tool MCP request schema unchanged unless a separate protocol
decision explicitly approves otherwise.

## Compatibility invariants retained before migration

Until the explicit v1.3 registry migration commit, development must preserve:

1. exactly **1,410** registered built-in/control operations;
2. exactly **3** MCP tools;
3. frozen `src/model.rs` request schema;
4. **412-token / 1,725-byte** measured tool schema;
5. MCP `initialize` version **1.0.0** and its hash-gated identity;
6. **30** error codes;
7. default worker count **1**;
8. the complete v1.2 Golden and Full Capability Audit baselines;
9. all 15 staged native `xfmr.*` names absent from the built-in registry.

## Next development decisions

The highest-value follow-up work is now:

- execute the formal v1.3 registry/audit migration once the staged 15-op contract
  is accepted for MCP exposure;
- extend winding geometry with explicit unequal-height / multi-winding models;
- add CTC/strand and structural stray-loss models only with visible assumptions
  and independent validation data;
- add calibrated thermal hot-spot and mechanical short-circuit models only when
  their source/model boundaries are explicit;
- keep certification, dielectric-clearance and standard-versioned compliance
  claims outside the generic calculator until a versioned source and independent
  validation path are provided.

## Repository roles

- `stickleetoto/Yekaterina-Dev` — active source development, verification, and
  future release preparation.
- `stickleetoto/Yekaterina` — stable distribution, binaries, public release
  documentation, and release evidence for v1.2.0.
