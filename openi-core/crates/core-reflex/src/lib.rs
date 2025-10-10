//! OpenI Core Reflex — autonomic event supervision layer.
//!
//! This crate defines the Reflex framework: an autonomic safety and health
//! monitoring subsystem for the OpenI Fabric. Reflex monitors observe message
//! envelopes and can raise alerts or halts based on behavior or policy violations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

pub mod monitor;
pub mod supervisor;

pub use monitor::*;
pub use supervisor::*;

/// ---------------------------------------------------------------------------
/// Core Data Types
/// ---------------------------------------------------------------------------

/// Basic event structure observed by Reflex monitors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique identifier for the event.
    pub id: String,
    /// Subject or topic this event pertains to.
    pub subject: String,
    /// Millisecond timestamp (epoch).
    pub ts_ms: u64,
    /// Structured headers (metadata).
    #[serde(default)]
    pub headers: serde_json::Value,
    /// Structured body (payload).
    #[serde(default)]
    pub body: serde_json::Value,
}

/// Describes the subjects Reflex monitors should subscribe to or publish to.
#[derive(Debug, Clone)]
pub struct ReflexSubjects {
    /// Main event stream subject that all reflexes observe.
    pub all_events_subject: String,
    /// Control subject for publishing alerts/halts.
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

/// ---------------------------------------------------------------------------
/// Bus Abstractions
/// ---------------------------------------------------------------------------

/// Trait representing a publish/subscribe transport layer.
///
/// The bus is expected to support async publish and subscribe operations.
/// Each reflex monitor may subscribe to events or publish control actions.
#[async_trait]
pub trait FabricBus: Send + Sync + 'static {
    async fn publish(&self, subject: &str, msg: &Envelope) -> Result<(), String>;
    async fn subscribe(&self, subject: &str) -> Result<Box<dyn BusSubscription>, String>;
}

/// Subscription trait — represents an async iterator of envelopes.
#[async_trait]
pub trait BusSubscription: Send + Sync {
    async fn next(&mut self) -> Option<Envelope>;
}

/// ---------------------------------------------------------------------------
/// Reflex Evaluation Model
/// ---------------------------------------------------------------------------

/// Reflex evaluation outcomes. A Reflex can:
/// - Continue: normal operation
/// - Alert: signal non-fatal anomaly
/// - Halt: signal critical failure (requires runtime intervention)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReflexAction {
    Continue,
    Alert(String),
    Halt(String),
}

/// Errors that can arise during reflex evaluation.
#[derive(Debug, Error)]
pub enum ReflexError {
    #[error("subscription error: {0}")]
    Subscription(String),
    #[error("bus publish error: {0}")]
    Bus(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Core trait implemented by all Reflex monitors.
///
/// Each Reflex can react to events (`on_event`) and periodic ticks (`on_tick`).
#[async_trait]
pub trait Reflex: Send + Sync {
    /// The canonical name of the reflex (used in logs and alerts).
    fn name(&self) -> &'static str;

    /// Invoked whenever a new Envelope is observed.
    async fn on_event(&mut self, evt: &Envelope) -> Result<ReflexAction, ReflexError>;

    /// Invoked on a periodic timer (default: every 500ms).
    async fn on_tick(&mut self, _now: tokio::time::Instant) -> Result<ReflexAction, ReflexError> {
        Ok(ReflexAction::Continue)
    }
}
