# CURRENT_STATE.md

> Active-development snapshot for `stickleetoto/Yekaterina-Dev`.

## Current milestone

**v1.2.0 is promoted stable; v1.3 transformer-design development is in progress.**

The verified v1.2.0 source was promoted from commit
`9019194af02f7b9cbe72c5a232e753e236a84b4f` to the stable distribution repository.
The v1.2 built-in operation surface remains frozen while post-release work continues
in this repository.

| | |
|---|---|
| Stable crate line | `1.2.0` |
| Advertised MCP version | `1.0.0` (deliberately frozen and hash-gated) |
| Registered built-in/control opcodes | **1,410** |
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

The active transformer line is `v1.3-transformer-design-engine`.

It intentionally keeps the frozen v1.2 registry untouched and separates the new
work into three layers:

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

Existing `elec.transformer_voltage`, `elec.transformer_current`, and
`elec.transformer_impedance` remain authoritative for ideal turns-ratio
relationships. The v1.3 work does not duplicate those operations.

The native `xfmr.*` functions are compiled and independently tested, but they are
not yet promoted onto the MCP built-in registry. That promotion requires an
explicit v1.3 registry/audit compatibility decision rather than silently changing
the frozen v1.2 surface.

## Transformer verification gates

The transformer branch has dedicated GitHub Actions workflows for:

- the importable formula pack and its independent Python verifier;
- the native transformer material/geometry/thermal tests and clippy;
- the winding-field tests and clippy;
- the unchanged full v1.2 CI regression path.

Before the transformer branch is merged, all of those workflows must be green on
the current head.

## Compatibility invariants retained during v1.3 development

Until a v1.3 surface migration is explicitly approved, development must preserve:

1. exactly **1,410** registered built-in/control operations;
2. exactly **3** MCP tools;
3. frozen `src/model.rs` request schema;
4. **412-token / 1,725-byte** measured tool schema;
5. MCP `initialize` version **1.0.0** and its hash-gated identity;
6. **30** error codes;
7. default worker count **1**;
8. the complete v1.2 Golden and Full Capability Audit baselines.

New transformer capabilities may live in importable packs or unregistered native
candidate modules without weakening those guarantees.

## Next development decisions

The highest-value follow-up work is:

- define the formal v1.3 registry/audit migration policy before exposing native
  `xfmr.*` operations over MCP;
- add candidate-level structured pass/fail reporting and multi-objective scoring;
- extend winding geometry only with models whose assumptions and verification
  data are explicit;
- keep certification, dielectric-clearance, mechanical-withstand, and thermal
  hot-spot claims outside the generic calculator unless a versioned source and
  independent validation path are provided.

## Repository roles

- `stickleetoto/Yekaterina-Dev` — active source development, verification, and
  future release preparation.
- `stickleetoto/Yekaterina` — stable distribution, binaries, public release
  documentation, and release evidence for v1.2.0.
