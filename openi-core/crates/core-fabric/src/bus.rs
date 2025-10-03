use crate::Envelope;
use futures::StreamExt;
use parking_lot::RwLock;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

/// Simple in-process bus with prefix-wildcard topics.
/// Subscribers register a pattern like "topic://ddl/discovered/*".
/// Publishers send to a concrete topic like "topic://ddl/discovered/pg".
///
/// Matching rule:
/// - Exact match OR
/// - Pattern ending in "/*" matches any topic with that prefix.
fn matches(pattern: &str, topic: &str) -> bool {
    if let Some(pref) = pattern.strip_suffix("/*") {
        topic.starts_with(pref)
    } else {
        pattern == topic
    }
}

#[derive(Clone)]
pub struct Subscription {
    pub pattern: String,
    pub rx: mpsc::Receiver<Envelope<Value>>,
}

struct SubEntry {
    pattern: String,
    tx: mpsc::Sender<Envelope<Value>>,
}

#[derive(Default)]
pub struct Bus {
    // map is not strictly required, but keeps future sharding options open
    subs: RwLock<HashMap<usize, SubEntry>>,
    next_id: RwLock<usize>,
}

impl Bus {
    pub fn new() -> Self {
        Self { subs: RwLock::new(HashMap::new()), next_id: RwLock::new(0) }
    }

    /// Subscribe to a topic pattern. Returns a Subscription with a Receiver.
    pub fn subscribe(&self, pattern: impl Into<String>) -> Subscription {
        let pattern = pattern.into();
        let (tx, rx) = mpsc::channel(1024);
        let mut subs = self.subs.write();
        let mut id_lock = self.next_id.write();
        let id = *id_lock;
        *id_lock += 1;
        subs.insert(id, SubEntry { pattern: pattern.clone(), tx });
        Subscription { pattern, rx }
    }

    /// Publish an envelope to a concrete topic.
    pub async fn publish(&self, topic: &str, env: Envelope<Value>) {
        // Collect matches then send; avoid holding lock across awaits
        let targets: Vec<mpsc::Sender<Envelope<Value>>> = {
            let subs = self.subs.read();
            subs.values()
                .filter(|s| matches(&s.pattern, topic))
                .map(|s| s.tx.clone())
                .collect()
        };

        for tx in targets {
            // best-effort; drop on full queues
            let _ = tx.send(env.clone()).await;
        }
    }
}

/// Global in-process bus for dev/local.
/// In prod, the kernel will bridge this to QUIC peers.
pub static GLOBAL_BUS: once_cell::sync::Lazy<Arc<Bus>> =
    once_cell::sync::Lazy::new(|| Arc::new(Bus::new()));

