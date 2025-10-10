use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use anyhow::Result;
use tokio::time::Duration;
use async_trait::async_trait;

use openi_core_reflex::{
    monitor::{PolicyGuardReflex, RateLimitReflex},
    supervisor::{ReflexSupervisor, ReflexSubjects},
    FabricBus, Envelope, BusSubscription,
};

/// ---------------------------------------------------------------------------
/// CLI Entry
/// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "openi", version, about = "OpenI Core CLI — Fabric, Agents, and Reflex Node Controller")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold a new agent (Rust stub)
    Init { name: String },
    /// Package an Agent Manifest (validate + sign)
    Package { path: String },
    /// Deploy an Agent Manifest into the fabric
    Deploy { path: String },
    /// Start a local kernel node (dev)
    Node,
    /// Trigger curiosity (exploration) loop manually
    Curiosity { topic: Option<String> },
}

/// ---------------------------------------------------------------------------
/// Setup
/// ---------------------------------------------------------------------------

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_level(true)
        .init();
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Init { name } => init_agent(&name),
        Cmd::Package { path } => package_manifest(&path),
        Cmd::Deploy { path } => deploy_manifest(&path),
        Cmd::Node => run_node(),
        Cmd::Curiosity { topic } => run_curiosity(topic),
    }
}

/// ---------------------------------------------------------------------------
/// Agent Commands
/// ---------------------------------------------------------------------------

fn init_agent(name: &str) -> Result<()> {
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
    println!("✅ Scaffolded agent: {}", name);
    Ok(())
}

fn package_manifest(path: &str) -> Result<()> {
    use std::fs;
    use ring::signature::Ed25519KeyPair;

    let path_trimmed = path.trim_end_matches('/');
    let data = fs::read_to_string(format!("{}/AgentManifest.yaml", path_trimmed))?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&data)?;
    println!(
        "✅ Validated Agent Manifest: kind={}",
        doc.get("kind").and_then(|k| k.as_str()).unwrap_or("?")
    );

    if let Ok(priv_key_path) = std::env::var("OPENI_SIGNING_KEY") {
        if let Ok(priv_key_bytes) = fs::read(priv_key_path) {
            let key_pair = Ed25519KeyPair::from_seed_unchecked(&priv_key_bytes)
                .map_err(|_| anyhow::anyhow!("Invalid Ed25519 private key"))?;
            let sig = key_pair.sign(data.as_bytes());
            fs::write(format!("{}/AgentManifest.sig", path_trimmed), sig.as_ref())?;
            println!("🔏 Manifest signed (Ed25519)");
        }
    } else {
        println!("⚠️ No OPENI_SIGNING_KEY provided — skipping signature.");
    }

    Ok(())
}

fn deploy_manifest(path: &str) -> Result<()> {
    println!("(stub) Registered agent from {}", path);
    Ok(())
}

/// ---------------------------------------------------------------------------
/// Node Runtime — Launch Reflex Supervisor + Mock Kernel
/// ---------------------------------------------------------------------------

fn run_node() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        // 1️⃣ Initialize Fabric bus
        let bus = openi_core_kernel::get_bus().await?;

        // 2️⃣ Load policy if available
        if let Ok(policy_path) = std::env::var("OPENI_POLICY") {
            if let Err(e) = openi_core_kernel::load_policy(&policy_path) {
                tracing::warn!("⚠️ Failed to load policy config: {}", e);
            } else {
                tracing::info!("✅ Loaded policy configuration from {}", policy_path);
            }
        } else {
            tracing::info!("No OPENI_POLICY set; running with default permissive policy.");
        }

        // 3️⃣ Telemetry counters
        let events = Arc::new(AtomicU64::new(0));
        let alerts = Arc::new(AtomicU64::new(0));
        let halts = Arc::new(AtomicU64::new(0));
        let uptime = Arc::new(AtomicU64::new(0));

        // Hook the mock bus to increment event counter whenever publish() happens
        let bus_events = events.clone();

        struct TelemetryBus<B: FabricBus> {
            inner: Arc<B>,
            counter: Arc<AtomicU64>,
        }

        #[async_trait]
        impl<B> FabricBus for TelemetryBus<B>
        where
            B: FabricBus + Send + Sync + 'static,
        {
            async fn publish(&self, subject: &str, msg: &Envelope) -> Result<(), String> {
                self.counter.fetch_add(1, Ordering::Relaxed);
                self.inner.publish(subject, msg).await
            }

            async fn subscribe(&self, subject: &str) -> Result<Box<dyn BusSubscription>, String> {
                self.inner.subscribe(subject).await
            }
        }

        let tele_bus = Arc::new(TelemetryBus {
            inner: Arc::clone(&bus),
            counter: bus_events.clone(),
        });

        // 4️⃣ ReflexSupervisor with telemetry listeners
        let subjects = ReflexSubjects::default();
        let reflex_bus = tele_bus.clone();
        let alerts_ref = alerts.clone();
        let halts_ref = halts.clone();

        tokio::spawn(async move {
            if let Ok(mut sub) = reflex_bus.subscribe("fabric.control").await {
                while let Some(env) = sub.next().await {
                    if env.subject.contains("alert") {
                        alerts_ref.fetch_add(1, Ordering::Relaxed);
                    } else if env.subject.contains("halt") {
                        halts_ref.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        ReflexSupervisor::new(Arc::clone(&tele_bus), subjects)
            .with_reflex(Box::new(RateLimitReflex::new(Duration::from_secs(1), 500)))
            .with_reflex(Box::new(PolicyGuardReflex::new(vec![
                "/identity/verified",
                "/policy/allowed",
            ])))
            .spawn();

        // 5️⃣ Start kernel node (mock runtime)
        openi_core_kernel::start_node().await?;

        // 6️⃣ Heartbeat telemetry
        tokio::spawn({
            let events = events.clone();
            let alerts = alerts.clone();
            let halts = halts.clone();
            let uptime = uptime.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let up = uptime.fetch_add(5, Ordering::Relaxed) + 5;
                    tracing::info!(
                        "🩺 Node heartbeat — uptime: {:>4}s | events: {:>6} | alerts: {:>3} | halts: {:>3}",
                        up,
                        events.load(Ordering::Relaxed),
                        alerts.load(Ordering::Relaxed),
                        halts.load(Ordering::Relaxed),
                    );
                }
            }
        });

        // 7️⃣ Keep alive indefinitely
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    })
}

/// ---------------------------------------------------------------------------
/// Curiosity / Exploration Stub
/// ---------------------------------------------------------------------------

fn run_curiosity(topic: Option<String>) -> Result<()> {
    let t = topic.unwrap_or_else(|| "general".to_string());
    println!("🧠 Triggering curiosity exploration for topic: {}", t);
    println!("(stub) This will later invoke the CuriosityAgent via Fabric RPC.");
    Ok(())
}
