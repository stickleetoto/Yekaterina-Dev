//! Yekaterina MCP server entry point.
//!
//! v1.1 moved the module set into `src/lib.rs` so the engine can be benchmarked
//! and tested in-process. This binary is otherwise unchanged from v1.0.0: same
//! runtime, same transport, same service.

use rmcp::{ServiceExt, transport::stdio};
use yekaterina::server::Yekaterina;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Yekaterina::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
