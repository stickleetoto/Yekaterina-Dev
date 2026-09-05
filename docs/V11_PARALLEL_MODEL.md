# Yekaterina v1.1 — parallel execution model

Phases 5 to 7 and 9. Internal only: no new MCP tool, no schema change, no new
error code. The LLM cannot tell whether a batch ran on one thread or eight.

## Default: one worker

v1.1 ships `DEFAULT_WORKERS = 1`. Parallelism is opt-in via `--workers N|auto`
or `YEKATERINA_WORKERS`. `auto` uses `available_parallelism()` clamped to 8.

This is deliberate, and the measurements below support keeping it. Parallelism
helps a **narrow class** of workloads and is neutral on the rest; shipping it on
by default would trade predictability for a benefit most requests never see.

## What parallelism actually bought

Paired A/B on the same binary, worker count as the only variable, 11 rounds,
alternating order. Every row is a unanimous sign test (0/11 rounds slower).

| workers | `heavy.integrate1000_x64` | speedup | `heavy.mixed_skew_16` | speedup | efficiency |
|---:|---:|---:|---:|---:|---:|
| 1 | 3.03 ms | 1.00x | 6.93 ms | 1.00x | — |
| 2 | 1.73 ms | **1.75x** | 4.10 ms | **1.69x** | 85–87% |
| 4 | 1.03 ms | **2.98x** | 2.58 ms | **2.72x** | 68–74% |
| 8 | 0.81 ms | **3.69x** | 2.34 ms | **2.98x** | 37–46% |

Efficiency falls off past four workers on this 10-physical-core machine, which
is expected: the serial part of a request — JSON parse, result serialization,
protocol framing — is untouched by workers, so Amdahl's law caps the gain.

**No speedup is asserted as a requirement.** These are the measured numbers on
one machine; other hardware will differ.

Everything else in the suite measures neutral, within ±2% with coin-flip sign
tests. That is the intended outcome, not a disappointment: those workloads are
correctly kept on the sequential path.

## Why the first cost model was wrong

The adaptive policy decides whether a run of independent items is worth
distributing. The first model scored an item as `cost_class × argument_size`.
The worker sweep showed it was **ordered wrongly**:

| workload | first model | measured at 4 workers |
|---|---:|---|
| 16 × mixed `signal.dft(256)` | 131,592 | **0.40x — scales** |
| 8 × `signal.fft(512)` | 262,656 | 1.04x — no gain |
| 4 × `stat.sum(10000)` | 320,032 | 1.07x — slightly worse |

The only workload that benefited scored *lowest of the three*. No threshold can
separate that, so the threshold was not the problem — the model was.

Argument size conflates two different costs:

* `stat.sum` over 10,000 numbers is O(n) trivial arithmetic. The request's time
  goes into **parsing** 10,000 numbers, serially, before any wave exists.
* `signal.dft` over 256 samples is O(n²) trigonometry on a small payload.

Distributing the first cannot help, and costs a handoff. That was measured:
`4 × stat.sum(10000)` ran 1.07x *slower* with four workers under the old model.

## The corrected model

`scheduler::estimated_cost` returns two numbers:

* **compute** — algorithmic work, what a worker removes from the critical path;
* **payload** — argument and result volume, parsed and serialized serially no
  matter how many workers exist.

Complexity comes from the operation's cost class, applied to the right
magnitude:

| class | compute | note |
|---|---|---|
| `1` | 1 | O(1) |
| `n` | payload | O(n) over the argument |
| `h` | payload² | quadratic, e.g. `signal.dft` |
| `h`, log-linear | payload·log₂(payload) | `signal.fft`/`ifft`/`rfft` |
| `h`, iteration-driven | largest scalar argument | `num.`/`ode.`/`optimize.`/`series.`, where the step count is an argument, not a container |
| `n3` | payload³ | `mat.mul` |

A run is distributed only when **both** hold:

```
compute >= PARALLEL_COMPUTE_FLOOR          (50,000)
compute >= payload * PARALLEL_PAYLOAD_RATIO (20)
```

Both constants come from the sweep, not from taste. The ratio test is what keeps
large-argument, cheap-arithmetic batches sequential.

Correcting the model also **found a workload the first one missed**:
`64 × num.integrate(1000 steps)` carries a tiny payload, so container size
scored it at 8,192 and it never parallelised. Scored by its step count it clears
the floor, and it now runs **2.8x faster at four workers**.

A wrong cost estimate can only pick a worse schedule; it can never change a
result. That is why this file is allowed to use `registry::cost_code`, which
classifies by opcode prefix — a heuristic explicitly forbidden in `safety.rs`,
where a wrong answer would be a race.

## Ordering

Results are written into slots keyed by input index. Completion order is
structurally unable to leak into the response: there is no sort, and no path by
which a completion order could become a result order.

Verified at the MCP level: **all 55 workload response fingerprints are
byte-identical across workers 1, 2, 4 and 8, and identical to the frozen v1.0.0
baseline.**

## What may run concurrently

`scheduler::plan_batch` marks an item `Concurrent` only when it parses,
references no earlier result, and resolves to a built-in classified `Pure`.
Everything else is `Ordered` and acts as a barrier: dependent items, user
formulas and composites, and the `udo.*` control operations.

`$input` is deliberately not treated as a dependency. It comes from the request,
not from `results`, so a batch of `["stat.sum", "$input"]` items still
parallelises.

Batch semantics measured against v1.0.0 *before* any of this was written, and
pinned by tests at every worker count:

```text
[["math.add",1,2],["math.mul","$0",10]]   -> {"r":[3.0,30.0]}
[["math.div",1,0],["math.mul","$0",10]]   -> {"r":[{"e":"DIV0"},{"e":"TYPE"}]}
[["math.mul","$1",10],["math.add",1,2]]   -> {"r":[{"e":"REF"},3.0]}
```

Errors are values, not aborts; a later item may reference one; forward
references are `REF` errors, which is what makes the dependency graph strictly
backward-pointing and safe to schedule.

## Panics

A panicking operation must not take down a worker and must not change what a
client observes. Jobs run under `catch_unwind`; the payload is carried back and
resumed on the request task.

That reproduces v1.0.0 exactly. There, a panic in `engine::execute` unwound
through the request's tokio task: the task died, no response was sent, the
server survived. Here the same panic reaches the same place with the same
payload; only the worker thread is spared.

**No new error code was introduced.** The existing vocabulary — 30 codes — has
nothing meaning "internal panic", and inventing one would have changed
observable behaviour, which requires approval. Relocating the panic avoids the
question entirely.

## Known limitation

When a batch aborts with `OUT_LIMIT`, sequential v1.0.0 never executed the items
after the one that crossed the limit. A wave executes them and discards the
results. This is unobservable except in one corner: a batch that both exceeds
`OUT_LIMIT` *and* contains a panicking operation after that point would panic
under parallel execution where v1.0.0 returned `OUT_LIMIT`. Both conditions are
pathological, and the wasted work is bounded by the wave. Recorded rather than
fixed, because fixing it would mean not starting work that is almost always
needed.
