# Yekaterina Roadmap — v1.4 to v2.0

## Core Principle

Yekaterina's goal is not to maximize operation count.

> **Offload the deterministic computations that humans and LLMs use most often, as compactly, quickly, and accurately as possible.**

A new Core operation should satisfy most of the following:

- high real-world usage frequency;
- broad reuse across domains;
- meaningful token or reasoning-cost reduction for LLMs;
- meaningful error reduction versus model-side arithmetic/reasoning;
- deterministic, bounded, and independently verifiable behavior.

Rare or highly industry-specific operations should not be promoted into Core merely because they can be implemented. They should remain extension candidates unless usage evidence justifies promotion.

---

## v1.4 — Core Math + Discovery

### Goal

Complete the most important gaps in common mathematical computation while improving operation discovery without expanding the MCP tool surface.

The active v1.4 line starts from the released v1.3 baseline of 1,425 operations and the current additive v1.4 layer.

### Priority areas

#### Algebra

- linear equations;
- quadratic equations;
- polynomial evaluation;
- common root solving;
- ratios and proportions;
- percentage change;
- CAGR and closely related common formulas.

#### Linear algebra

- linear-system solving;
- matrix multiplication;
- determinant;
- inverse;
- transpose;
- rank;
- dot product;
- vector norm;
- normalization;
- distance.

#### Statistics

- mean;
- median;
- mode;
- variance;
- standard deviation;
- percentile;
- quantile;
- covariance;
- correlation.

#### Common numerical operations

- clamp;
- interpolation;
- common root-finding methods;
- numerical integration;
- rounding;
- significant-figure handling.

### Agent usability

Improve `yk.find` so common natural-language intent maps reliably to canonical operations while exact canonical names and aliases remain stronger than semantic ranking.

Examples:

```text
"calculate the average"       -> stats.mean
"percent increase"            -> common percentage-change operation
"solve a system of equations" -> linalg.solve
"distance between two points" -> geometry/distance operation
```

### Exit gate

- all new operations have reference verification;
- Golden regression remains green;
- all v1.3 operations remain compatible;
- MCP tools remain exactly three;
- semantic discovery has dedicated verification;
- real-process MCP verification passes.

---

## v1.5 — Human Common Functions

### Goal

Absorb deterministic calculations that repeatedly appear in daily life, education, software development, spreadsheets, and routine professional work.

### Date and time

- date difference;
- time difference;
- timestamp conversion;
- duration calculation and normalization;
- leap-year checks;
- weekday calculation.

### Units

High-frequency conversions for:

- length;
- area;
- volume;
- mass;
- temperature;
- speed;
- pressure;
- energy;
- power;
- data size.

### Geometry

- triangle;
- rectangle;
- circle;
- sphere;
- cylinder;
- common distances;
- common angles;
- coordinate conversion.

### Finance

Only deterministic calculation primitives belong in Core, not investment advice or policy decisions.

Candidates:

- simple interest;
- compound interest;
- CAGR;
- discount;
- markup;
- margin;
- loan payment;
- present value;
- future value.

### Developer utilities

- base conversion;
- hexadecimal / decimal / binary conversion;
- checksum primitives;
- common hash operations where appropriate;
- encoding helpers;
- byte-size conversion;
- semantic-version comparison.

### Design rule

Prefer functions that users currently reach for a calculator, spreadsheet formula, or one-line Python expression to solve.

---

## v1.6 — Composition Engine

### Goal

Turn existing batch/pipeline support into a first-class deterministic composition capability so multiple calculations can be completed through one compact `yk.compute` request.

Example conceptual flow:

```text
data
  -> mean
  -> standard deviation
  -> normalization
  -> percentile
```

### Capabilities

- operation chaining;
- named intermediate values;
- dependency graphs;
- reuse of intermediate results;
- fail-fast behavior;
- bounded execution;
- deterministic execution order.

### Hard boundary

Yekaterina must not become a general-purpose scripting or remote-execution engine.

Allowed:

```text
bounded deterministic compute graph
```

Not allowed as compute operations:

```text
arbitrary shell execution
arbitrary network access
arbitrary filesystem access
general-purpose scripting
```

---

## v1.7 — Common Data Compute

### Goal

Offload small and medium deterministic structured-data operations that are wasteful for an LLM to perform token by token.

Priority candidates:

- sort;
- unique;
- deduplicate;
- frequency counting;
- histogram;
- grouping;
- aggregation;
- min/max;
- top-k;
- rank;
- cumulative sum;
- moving average;
- rolling windows;
- normalization;
- standardization;
- correlation.

Yekaterina is not intended to become a dataframe framework. Only common deterministic data work with clear agent value belongs in Core.

---

## v1.8 — Probability + Scientific Fundamentals

### Goal

Cover the common scientific and engineering calculations that recur across education, research, programming, and general technical work.

### Probability

- permutations;
- combinations;
- binomial calculations;
- normal-distribution calculations;
- expected value;
- common probability primitives;
- confidence intervals where deterministic inputs and assumptions are explicit.

### Physics

Prioritize high-frequency fundamentals such as:

- velocity;
- acceleration;
- force;
- momentum;
- kinetic energy;
- potential energy;
- power;
- Ohm's law;
- electrical power;
- basic thermal equations.

### Signal and engineering basics

- frequency / period conversion;
- dB conversion;
- RMS;
- common filter mathematics;
- common vector calculations.

Specialized industrial design procedures remain outside Core unless usage evidence proves they are broadly useful.

---

## v1.9 — Usage Intelligence

### Goal

Stop guessing which operations should be added next. Use measured demand and benchmark evidence to guide Core promotion.

MCPMeter or equivalent benchmark tooling should measure:

- most frequently called operations;
- `yk.find` query frequency;
- searches that failed because no suitable operation existed;
- average token saving;
- latency;
- tool-call overhead;
- computation failure rate;
- correctness versus direct LLM calculation.

### Function Promotion Score

A candidate scoring model:

```text
Promotion Score =
    Usage Frequency
  + Generality
  + Token Saving
  + Error Reduction
  - Maintenance Cost
```

The exact weighting may evolve, but the principle is permanent: Core promotion should be evidence-driven.

---

## v2.0 — Human Compute Core

### Goal

Establish Yekaterina as a compact, deterministic compute layer for LLM agents rather than merely a large collection of calculator functions.

Target conceptual structure:

```text
LLM
 |
 +-- yk.find
 +-- yk.spec
 +-- yk.compute
        |
        v
+---------------------------+
|      Yekaterina Core      |
+---------------------------+
| Math                      |
| Statistics                |
| Linear Algebra            |
| Units                     |
| Date / Time               |
| Geometry                  |
| Finance primitives        |
| Data                      |
| Probability               |
| Scientific fundamentals   |
| Developer utilities       |
+---------------------------+
```

The MCP surface should remain exactly three tools unless there is overwhelming evidence that a schema change is necessary.

Operation count is not the target. A smaller high-density Core that covers most recurring human/agent computation is preferable to a much larger registry dominated by rare functions.

---

## Permanent Rules

### 1. No opcode race

Operation count is not a KPI.

### 2. Frequency first

Implement frequently used calculations before rare ones.

### 3. Core stays general

A function needed by only one project or one narrow industry should not enter Core by default.

### 4. Verification is mandatory

Every Core operation requires deterministic tests and appropriate independent/reference verification.

### 5. Freeze the MCP surface where possible

Prefer to keep:

```text
yk.find
yk.spec
yk.compute
```

as the complete model-facing tool surface.

### 6. Measure every release

Track at least:

- correctness;
- token saving;
- latency;
- tool overhead;
- discovery success rate.

### 7. Rare functions become extensions

Possible future separation:

```text
Yekaterina Core
Yekaterina Engineering Pack
Yekaterina Science Pack
Yekaterina Finance Pack
```

Core remains compact and broadly useful.

---

## Development Priority Order

```text
1. High-frequency missing functions
2. Core-math gaps
3. yk.find discovery quality
4. Pipeline / composition
5. Common data computation
6. Scientific fundamentals
7. Usage measurement and MCPMeter evidence
8. v2.0 stabilization
```

The long-term decision rule is simple:

> **If there is no good reason for an LLM to perform a deterministic calculation itself, Yekaterina should be able to offload it.**
