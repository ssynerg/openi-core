use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name="openi", version, about="OpenI Core CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold a new agent (Rust stub)
    Init { name: String },
    /// Package an Agent Manifest (validate + sign later)
    Package { path: String },
    /// Deploy an Agent Manifest into the fabric
    Deploy { path: String },
    /// Start a local kernel node (dev)
    Node,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Init { name } => init_agent(&name),
        Cmd::Package { path } => package_manifest(&path),
        Cmd::Deploy { path } => deploy_manifest(&path),
        Cmd::Node => run_node(),
    }
}

fn init_agent(name: &str) -> anyhow::Result<()> {
    use std::fs;
    use std::path::PathBuf;
    let dir = PathBuf::from(name);
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("Agent.toml"),
        format!(
            r#"
name = "{name}"
version = "0.1.0"
runtime = "wasm"
"#,
        ),
    )?;
    println!("Scaffolded agent: {}", name);
    Ok(())
}

fn package_manifest(path: &str) -> anyhow::Result<()> {
    use std::fs;
    let p = if path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        path
    };
    let data = fs::read_to_string(format!("{}/AgentManifest.yaml", p))?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&data)?;
    println!(
        "Validated Agent Manifest: kind={}",
        doc.get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("?")
    );
    Ok(())
}

fn deploy_manifest(path: &str) -> anyhow::Result<()> {
    println!("(stub) Registered agent from {}", path);
    Ok(())
}

fn run_node() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        openi_core_kernel::start_node().await?;
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}
