//! Yekaterina MCP server entry point.
//!
//! v1.1 moved the module set into `src/lib.rs` so the engine can be benchmarked
//! and tested in-process. This binary is otherwise unchanged from v1.0.0: same
//! runtime, same transport, same service.

use rmcp::{ServiceExt, transport::stdio};
use yekaterina::{pool, server::Yekaterina};

/// Workers used when nothing is configured.
///
/// v1.1 ships single-worker by default. Compatibility and predictability come
/// first: parallel execution stays opt-in until the 1/2/4/8 scaling data,
/// determinism stress runs and RSS measurements are all complete. Raising this
/// is a deliberate release decision, not a default to drift into.
const DEFAULT_WORKERS: usize = 1;

/// Resolve the worker count from `--workers N|auto`, else `YEKATERINA_WORKERS`,
/// else [`DEFAULT_WORKERS`].
///
/// Unparseable input is a hard error rather than a silent fallback: a client
/// that asked for four workers and got one should be told.
fn configured_workers() -> Result<usize, String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--workers") {
        let value = args
            .get(pos + 1)
            .ok_or_else(|| "--workers requires a value (N or 'auto')".to_string())?;
        return pool::resolve_workers(value);
    }
    if let Some(rest) = args.iter().find_map(|a| a.strip_prefix("--workers=")) {
        return pool::resolve_workers(rest);
    }
    match std::env::var("YEKATERINA_WORKERS") {
        Ok(value) => pool::resolve_workers(&value),
        Err(_) => Ok(DEFAULT_WORKERS),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workers = match configured_workers() {
        Ok(n) => n,
        Err(message) => {
            // stdout carries the MCP protocol; diagnostics go to stderr.
            eprintln!("yekaterina: {message}");
            std::process::exit(2);
        }
    };
    let service = Yekaterina::with_workers(workers).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
