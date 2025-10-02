use tracing::info;

pub mod identity;
pub mod policy;
pub mod runtime;

pub async fn start_node() -> anyhow::Result<()> {
    info!("Starting OpenI kernel node (stub)…");
    runtime::start().await
}
