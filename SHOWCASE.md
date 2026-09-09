# Yekaterina Showcase

**Pure computation. Minimal tokens.**

Yekaterina is a deterministic computation offloader for AI agents over MCP. The
core idea is to keep the model-facing surface tiny while the internal compute
registry grows.

## At a glance

```text
AI agent / Codex
      |
      |  MCP over stdio
      v
+------------------------------+
| Yekaterina                   |
|                              |
|  yk.find    discover         |
|  yk.spec    inspect          |
|  yk.compute execute          |
+---------------+--------------+
                |
                v
       1,410 registered ops
   arithmetic / exact numbers /
 statistics / probability / LA /
 numerical methods / batches...
```

RC1 keeps the externally visible request schema frozen:

- **1,410** registered built-in/control operations;
- **3** MCP tools;
- **412 tokens / 1,725 bytes** measured MCP schema footprint;
- Golden **527/527**;
- Full Capability Audit **1,410/1,410**.

The `initialize` response deliberately continues to advertise MCP server version
`1.0.0`. The crate is `1.2.0`; the wire identity is separately frozen and
hash-gated for compatibility.

## 60-second local demo

### Windows

```powershell
.\DEMO_WINDOWS.bat
```

### Linux / macOS

```bash
./DEMO_UNIX.sh
```

If `target/release/yekaterina` is missing, the wrapper builds it with the pinned
lockfile first.

The demo uses the same STDIO MCP path an agent uses. It does not call Rust
functions directly.

Expected shape:

```text
Yekaterina v1.2 RC demo
MCP tools: yk.compute, yk.find, yk.spec
math.add: 20 + 22 -> 42
dec.add: 0.1 + 0.2 -> 0.3
batch: [3, 42, 10]
discovery "welch" -> ...
spec ... -> ...
DEMO PASS
```

The script fails non-zero if the tool catalog drifts, the exact decimal example
is no longer exact, the batch result changes, discovery fails, or the MCP server
does not answer.

## What the three tools do

### `yk.find`

Lazy discovery. The model searches only when it does not already know the
operation name.

### `yk.spec`

Returns a compact operation contract: arguments, return type, source/capability,
and cost hints.

### `yk.compute`

Runs a single operation, compact batch, sequential pipeline, expression, or
supported persistent user-operation control.

This split avoids placing all operation definitions into `tools/list`.

## Why offload computation?

Language models are useful for deciding *what* computation to perform. They are
not the ideal place to repeatedly execute deterministic arithmetic, exact
decimal work, statistical tests, or numerical kernels token by token.

Yekaterina lets the agent keep planning in language while delegating the
deterministic part to a bounded native engine.

## Verification model

The v1.2 baseline at `8361beae` passed:

- static audit: **24 pass / 0 fail**;
- **386** Rust test executions;
- Golden **527/527**;
- Full Capability Audit **1,410/1,410**;
- 164 independent applied/exact checks;
- 961 statistical reference values against SciPy, NumPy, and mpmath;
- 577 multiplicity/effect-size checks against statsmodels/SciPy and independent
  procedure definitions.

RC1 adds another layer: the exact reviewer-facing demo runs in CI against the
release binary.

## Codex

Yekaterina is a local STDIO MCP server. See
[`docs/CODEX_SETUP.md`](docs/CODEX_SETUP.md) for current Codex CLI and
`config.toml` setup.

A typical agent flow is:

```text
Need Welch-related computation
        |
        v
yk.find("welch")
        |
        v
yk.spec(<selected opcode>)
        |
        v
yk.compute(...)
```

## Runtime independence and development workflow

Yekaterina does not require an OpenAI or Anthropic model/API at runtime. It is an
MCP compute server and can be used by compatible hosts.

The project used an AI-assisted engineering workflow during development:
ChatGPT/Codex contributed to early architecture and implementation, and Claude
Code was also used in later implementation, review, debugging, and verification.
Runtime correctness is gated by executable tests and independent reference
checks rather than by trusting any coding assistant's output.

## Security boundary

Compute operations expose no arbitrary shell, arbitrary network, or arbitrary
filesystem execution. Resource guards bound expression, batch, pipeline, user
operation, and numerical workloads.

The worker pool is opt-in and keeps result order structural. The default remains
one worker.
