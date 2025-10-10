//! Supervisor: wires the Reflex set to the Fabric bus, schedules ticks, and executes actions.

use super::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration, Instant};

/// Trait alias so we can hold a heterogenous set of boxed reflexes.
type BoxedReflex = Box<dyn Reflex>;

/// Minimal ReflexSubjects for simulation or real deployment.
#[derive(Clone, Debug)]
pub struct ReflexSubjects {
    pub all_events_subject: String,
    pub control_subject: String,
}

impl Default for ReflexSubjects {
    fn default() -> Self {
        Self {
            all_events_subject: "fabric.events.*".into(),
            control_subject: "fabric.control".into(),
        }
    }
}

/// Encapsulates lifecycle of always-on reflex agents.
pub struct ReflexSupervisor<BUS> {
    bus: Arc<BUS>,
    subjects: ReflexSubjects,
    reflexes: Vec<BoxedReflex>,
    tick_interval: Duration,
}

impl<BUS> ReflexSupervisor<BUS>
where
    BUS: FabricBus + Send + Sync + 'static,
{
    pub fn new(bus: Arc<BUS>, subjects: ReflexSubjects) -> Self {
        Self {
            bus,
            subjects,
            reflexes: Vec::new(),
            tick_interval: Duration::from_millis(500),
        }
    }

    pub fn with_reflex(mut self, reflex: BoxedReflex) -> Self {
        self.reflexes.push(reflex);
        self
    }

    pub fn with_tick_interval(mut self, every: Duration) -> Self {
        self.tick_interval = every;
        self
    }

    /// Start the Reflex event + tick loops.
    pub fn spawn(self) {
        let bus = self.bus.clone();
        let subjects = self.subjects.clone();
        let reflexes = Arc::new(Mutex::new(self.reflexes));
        let tick_every = self.tick_interval;

        // Event loop
        tokio::spawn({
            let bus = bus.clone();
            let reflexes = reflexes.clone();
            async move {
                println!("🧠 ReflexSupervisor: subscribing to {}", subjects.all_events_subject);
                let mut sub = match bus.subscribe(&subjects.all_events_subject).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("❌ ReflexSupervisor: failed to subscribe: {}", e);
                        return;
                    }
                };

                while let Some(evt) = sub.next().await {
                    let mut g = reflexes.lock().await;
                    for r in g.iter_mut() {
                        match r.on_event(&evt).await {
                            Ok(ReflexAction::Continue) => {}
                            Ok(ReflexAction::Alert(reason)) => {
                                println!("⚠️  ALERT from {} → {}", r.name(), reason);
                                let _ = publish_alert(&*bus, &subjects.control_subject, r.name(), &reason, &evt).await;
                            }
                            Ok(ReflexAction::Halt(reason)) => {
                                println!("🛑 HALT from {} → {}", r.name(), reason);
                                let _ = publish_halt(&*bus, &subjects.control_subject, r.name(), &reason, &evt).await;
                            }
                            Err(err) => {
                                eprintln!("❗ Reflex error in {} → {}", r.name(), err);
                            }
                        }
                    }
                }
            }
        });

        // Tick loop
        tokio::spawn(async move {
            let mut ticker = interval(tick_every);
            loop {
                ticker.tick().await;
                let now = Instant::now();
                let mut g = reflexes.lock().await;
                for r in g.iter_mut() {
                    if let Err(e) = r.on_tick(now).await {
                        eprintln!("⏱️  Tick error in {} → {}", r.name(), e);
                    }
                }
            }
        });
    }
}

/// ---------------------------------------------------------------------------
/// Helper functions for reflex control envelopes
/// ---------------------------------------------------------------------------

async fn publish_alert<B: FabricBus>(
    bus: &B,
    control_subject: &str,
    reflex: &str,
    reason: &str,
    evt: &Envelope,
) -> Result<(), String> {
    let env = control_envelope("alert", reflex, reason, evt);
    bus.publish(control_subject, &env).await
}

async fn publish_halt<B: FabricBus>(
    bus: &B,
    control_subject: &str,
    reflex: &str,
    reason: &str,
    evt: &Envelope,
) -> Result<(), String> {
    let env = control_envelope("halt", reflex, reason, evt);
    bus.publish(control_subject, &env).await
}

fn control_envelope(kind: &str, reflex: &str, reason: &str, evt: &Envelope) -> Envelope {
    Envelope {
        id: format!("reflex:{}:{}", kind, uuid()),
        subject: format!("reflex.{}", kind),
        ts_ms: epoch_ms(),
        headers: serde_json::json!({
            "reflex": reflex,
            "reason": reason,
            "source_event": evt.id,
        }),
        body: serde_json::json!({}),
    }
}

fn epoch_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", n)
}
