# Yekaterina v1.3 Transformer Candidate Evaluator

The transformer calculation kernel can calculate electrical, magnetic, winding,
loss, impedance and thermal quantities for one proposed design. The candidate
evaluator adds the missing decision layer needed by an optimizer or LLM agent:

- reject a design against explicit numeric constraints;
- explain every violated constraint;
- normalize unlike engineering objectives onto an interpretable 0..1 scale;
- combine objectives using caller-supplied weights;
- rank many candidates deterministically while always placing feasible designs
  ahead of infeasible ones.

This is a preliminary engineering decision helper. It is not a certification,
safety or regulatory-compliance engine. Limits and objective bands are caller
data and must come from the applicable design policy, material data, test data or
versioned standard outside this generic calculator.

## Candidate operations

The v1.3 candidate module introduces two candidate-only native operations:

- `xfmr.candidate_evaluate(metrics, constraints, objectives)`
- `xfmr.candidate_rank(candidates, constraints, objectives)`

They are intentionally not yet present in the frozen v1.2 MCP registry. Like the
other native transformer functions, they are compiled and verified through a
dedicated test target before any v1.3 registry migration is approved.

## Metrics

A candidate is represented by a JSON object whose values are finite numbers.
Metric names are deliberately caller-defined so the evaluator can consume the
outputs of current and future transformer models without hard-coding one topology.

Example:

```json
{
  "bmax_t": 1.55,
  "window_fill": 0.43,
  "winding_c": 92.0,
  "impedance_pct": 5.8,
  "total_loss_w": 1080.0,
  "mass_kg": 410.0
}
```

A referenced metric must exist. Missing, non-numeric or non-finite values fail
closed rather than being silently ignored.

## Constraints

Each constraint names one metric and defines `min`, `max`, or both.

```json
[
  {"metric":"bmax_t","max":1.65},
  {"metric":"window_fill","max":0.50},
  {"metric":"winding_c","max":105.0},
  {"metric":"impedance_pct","min":5.0,"max":6.5}
]
```

The evaluator returns an ordered result for every constraint plus a compact
`violations` list. A candidate passes only when every constraint passes.

## Objectives and 0..1 desirability

The evaluator does not add unlike physical units directly. Every objective first
maps its engineering metric to a unitless desirability from 0 to 1.

### Minimize

```json
{"metric":"total_loss_w","kind":"minimize","good":900,"bad":1500,"weight":0.6}
```

Values at or below `good` receive 1. Values at or above `bad` receive 0. Values
between them interpolate linearly.

### Maximize

```json
{"metric":"margin_pct","kind":"maximize","good":20,"bad":5,"weight":0.2}
```

Values at or above `good` receive 1. Values at or below `bad` receive 0.

### Target

```json
{"metric":"impedance_pct","kind":"target","target":5.75,"tolerance":0.75,"weight":0.2}
```

The exact target receives 1. Desirability falls linearly with absolute error and
reaches 0 at the supplied tolerance.

The final score is the weighted mean of objective desirabilities, therefore it is
always bounded to 0..1. Weights must be positive. The evaluator does not invent
weights or normalize them against engineering meaning on the caller's behalf.

## Ranking policy

`xfmr.candidate_rank` evaluates every candidate under one shared constraint and
objective policy, then sorts using three deterministic rules:

1. feasible candidates before infeasible candidates;
2. higher score first within the same feasibility class;
3. original candidate index as the final tie-breaker.

This prevents a physically or policy-invalid candidate from winning only because
it scores well on loss, mass or another optimization objective.

## Intended optimization chain

A future transformer optimizer can now use this loop:

1. generate geometry/material candidates;
2. run the transformer formula/native calculation chain;
3. assemble selected outputs into a metric object;
4. evaluate hard limits with `xfmr.candidate_evaluate`;
5. rank the surviving design space with `xfmr.candidate_rank`;
6. refine the highest-ranked candidates;
7. retain the complete constraint and objective report as design evidence.

The evaluator therefore separates three concepts that should not be conflated:
calculation, admissibility and preference.

## Resource and determinism limits

To keep the operation bounded and predictable:

- at most 128 metrics per candidate;
- at most 128 constraints;
- at most 64 objectives;
- at most 256 candidates in one ranking call;
- metric/objective values must be finite;
- malformed ranges or non-positive weights/tolerances fail closed;
- candidate ranking has a deterministic index tie-breaker.

## Verification

`tests/transformer_candidate.rs` verifies:

- feasible candidate reporting;
- explicit multi-constraint failures;
- exact 0 and 1 desirability endpoints;
- feasible-first ranking;
- score ordering;
- deterministic ties;
- fail-closed malformed specifications.

`.github/workflows/xfmr-candidate.yml` runs the candidate evaluator test target and
clippy independently of the existing transformer and full-regression workflows.

## Registry status

The evaluator is part of the v1.3 candidate design surface only. The frozen v1.2
registry remains at 1,410 built-in/control operations and the three-tool MCP
schema remains unchanged. Native transformer promotion still requires a formal
v1.3 manifest and audit migration before `xfmr.*` names become discoverable via
`yk.find` / `yk.spec` and executable through normal `yk.compute` dispatch.
