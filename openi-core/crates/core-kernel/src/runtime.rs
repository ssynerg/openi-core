use tracing::info;

pub async fn start() -> anyhow::Result<()> {
    info!("Kernel runtime up. (WASM/OCI adapters pending)");
    Ok(())
}
