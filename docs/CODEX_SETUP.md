# Codex setup — Yekaterina over STDIO MCP

Yekaterina is a local STDIO MCP server. Current Codex local clients support
STDIO MCP servers and share MCP configuration across the ChatGPT desktop app,
Codex CLI, and IDE extension.

Official MCP documentation:
<https://learn.chatgpt.com/docs/extend/mcp>

## 1. Build Yekaterina

From this repository:

```powershell
cargo build --locked --release
```

On Windows the executable is:

```text
target\release\yekaterina.exe
```

Run the project demo before adding it to Codex:

```powershell
.\DEMO_WINDOWS.bat
```

The last line must be:

```text
DEMO PASS
```

## 2. Add it with the Codex CLI

The Codex CLI syntax for a local STDIO server is:

```text
codex mcp add <server-name> -- <stdio-server-command>
```

Windows example:

```powershell
codex mcp add yekaterina -- "C:\Tools\Yekaterina\yekaterina.exe"
```

Or point it at a source build:

```powershell
codex mcp add yekaterina -- "D:\path\to\Yekaterina-Dev\target\release\yekaterina.exe"
```

Verify the registration:

```powershell
codex mcp list
```

Inside the Codex TUI, `/mcp` shows active MCP servers.

## 3. `config.toml` alternative

Codex stores MCP configuration in `~/.codex/config.toml`. A trusted project can
also use a project-scoped `.codex/config.toml`.

```toml
[mcp_servers.yekaterina]
command = "C:\\Tools\\Yekaterina\\yekaterina.exe"
args = []
startup_timeout_sec = 10
```

Yekaterina does not need network access for this connection. Codex launches the
process locally and communicates over stdin/stdout.

## 4. What Codex sees

Yekaterina exposes exactly:

```text
yk.compute
yk.find
yk.spec
```

Recommended flow:

```text
known opcode    -> yk.compute
unknown concept -> yk.find -> yk.spec -> yk.compute
```

Example request to Codex:

```text
Use Yekaterina for the deterministic calculations in this task.
If you do not know the operation name, discover it with yk.find and inspect it
with yk.spec before calling yk.compute.
```

## Version note

The Rust crate is `1.2.0`, but the MCP `initialize` response deliberately
continues to advertise server version `1.0.0`. That wire identity is frozen and
hash-gated so the v1.1/v1.2 internal evolution does not silently change what an
existing MCP client observes.

This is intentional and should not be treated as an installation mismatch.
