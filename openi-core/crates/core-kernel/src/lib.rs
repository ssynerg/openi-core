use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use serde_json::json;

use openi_core_reflex::{FabricBus, BusSubscription, Envelope};

/// ---------------------------------------------------------------------------
/// Kernel Runtime Entrypoint
/// ---------------------------------------------------------------------------

pub mod runtime;
pub mod identity;
pub mod policy;

/// Starts the OpenI kernel node (stubbed for now).
pub async fn start_node() -> Result<()> {
    info!("Starting OpenI kernel node (stub)...");
    runtime::start().await
}

/// ---------------------------------------------------------------------------
/// Mock Fabric Bus — with Reflex Simulation Mode
/// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct MockBus {
    tx: broadcast::Sender<Envelope>,
}

#[async_trait::async_trait]
impl FabricBus for MockBus {
    async fn publish(&self, subject: &str, msg: &Envelope) -> Result<(), String> {
        println!("[MOCK BUS] publish → {}", subject);
        let _ = self.tx.send(msg.clone());
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> Result<Box<dyn BusSubscription>, String> {
        println!("[MOCK BUS] subscribe → {}", subject);
        let rx = self.tx.subscribe();
        Ok(Box::new(MockSub { rx }))
    }
}

/// ---------------------------------------------------------------------------
/// Subscription Wrapper
/// ---------------------------------------------------------------------------

pub struct MockSub {
    rx: broadcast::Receiver<Envelope>,
}

#[async_trait::async_trait]
impl BusSubscription for MockSub {
    async fn next(&mut self) -> Option<Envelope> {
        match self.rx.recv().await {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
}

/// ---------------------------------------------------------------------------
/// Bus Factory + Reflex Simulation
/// ---------------------------------------------------------------------------

/// Returns a mock Fabric bus handle (Arc<MockBus>) and starts the reflex simulator.
pub async fn get_bus() -> Result<Arc<MockBus>> {
    let (tx, _rx) = broadcast::channel(1000);
    let bus = Arc::new(MockBus { tx });

    // Spawn reflex simulation
    let sim_bus = bus.clone();
    tokio::spawn(async move {
        simulate_reflex_events(sim_bus).await;
    });

    Ok(bus)
}

/// Simulates envelope events periodically to trigger Reflex activity.
async fn simulate_reflex_events(bus: Arc<MockBus>) {
    let mut counter = 0u64;

    loop {
        counter += 1;

        // Alternate between valid and invalid policy headers every 10th event
        let headers = if counter % 10 == 0 {
            json!({
                "identity": { "verified": false },
                "policy": { "allowed": false }
            })
        } else {
            json!({
                "identity": { "verified": true },
                "policy": { "allowed": true }
            })
        };

        let evt = Envelope {
            id: format!("evt-{}", counter),
            subject: "fabric.events.mock".to_string(),
            ts_ms: epoch_ms(),
            headers,
            body: json!({ "seq": counter, "payload": "synthetic test data" }),
        };

        let _ = bus.publish("fabric.events.*", &evt).await;

        // Every 100 cycles, trigger a flood to test RateLimitReflex
        if counter % 100 == 0 {
            println!("⚡ Generating burst load to trip RateLimitReflex");
            for _ in 0..550 {
                let evt2 = Envelope {
                    id: format!("burst-{}", counter),
                    subject: "fabric.events.burst".to_string(),
                    ts_ms: epoch_ms(),
                    headers: json!({
                        "identity": { "verified": true },
                        "policy": { "allowed": true }
                    }),
                    body: json!({ "burst": counter }),
                };
                let _ = bus.publish("fabric.events.*", &evt2).await;
            }
        }

        sleep(Duration::from_millis(200)).await;
    }
}

/// ---------------------------------------------------------------------------
/// Mock Policy Loader
/// ---------------------------------------------------------------------------

pub fn load_policy(path: &str) -> Result<()> {
    println!("[MOCK POLICY] loaded from {}", path);
    Ok(())
}

/// ---------------------------------------------------------------------------
/// Utility
/// ---------------------------------------------------------------------------

fn epoch_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
