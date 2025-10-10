//! Built-in Reflex monitors.
//!
//! These are reference implementations. They are cheap to run and safe to keep always-on.

use super::*;
use std::collections::VecDeque;
use tokio::time::{Instant, Duration};
use tracing::{warn, error};

/// Detects abnormal publish rates by counting events within a sliding window.
/// Useful to catch storms, loops, or floods.
pub struct RateLimitReflex {
    window: Duration,
    max_events: usize,
    deque: VecDeque<Instant>,
}

impl RateLimitReflex {
    pub fn new(window: Duration, max_events: usize) -> Self {
        Self {
            window,
            max_events,
            deque: VecDeque::with_capacity(max_events + 8),
        }
    }

    fn prune_old(&mut self, now: Instant) {
        while let Some(ts) = self.deque.front().copied() {
            if now.duration_since(ts) > self.window {
                self.deque.pop_front();
            } else {
                break;
            }
        }
    }
}

#[async_trait]
impl Reflex for RateLimitReflex {
    fn name(&self) -> &'static str {
        "rate_limit"
    }

    async fn on_event(&mut self, _evt: &Envelope) -> Result<ReflexAction, ReflexError> {
        let now = Instant::now();
        self.prune_old(now);
        self.deque.push_back(now);

        if self.deque.len() > self.max_events {
            let msg = format!(
                "RateLimitReflex: {} events in {:?} (limit {})",
                self.deque.len(),
                self.window,
                self.max_events
            );
            warn!("{}", msg);
            return Ok(ReflexAction::Alert(msg));
        }

        Ok(ReflexAction::Continue)
    }

    async fn on_tick(&mut self, _now: Instant) -> Result<ReflexAction, ReflexError> {
        // No periodic action required; pruning happens per-event.
        Ok(ReflexAction::Continue)
    }
}

/// Detects repeated error patterns (e.g., panic loops) on a subject or header key.
pub struct PanicLoopReflex {
    /// JSON pointer into headers/body to check (e.g. "/error/code").
    field_pointer: &'static str,
    /// Sliding window size.
    window: usize,
    /// Minimum count within window to trigger.
    min_repeats: usize,
    ring: VecDeque<bool>,
}

impl PanicLoopReflex {
    pub fn new(field_pointer: &'static str, window: usize, min_repeats: usize) -> Self {
        Self {
            field_pointer,
            window,
            min_repeats,
            ring: VecDeque::with_capacity(window),
        }
    }

    fn push(&mut self, is_error: bool) -> usize {
        if self.ring.len() == self.window {
            self.ring.pop_front();
        }
        self.ring.push_back(is_error);
        self.ring.iter().filter(|x| **x).count()
    }

    fn extract_flag(&self, evt: &Envelope) -> bool {
        let get_bool = |json: &serde_json::Value, path: &str| -> bool {
            let mut cur = json;
            for seg in path.trim_start_matches('/').split('/') {
                match cur.get(seg) {
                    Some(next) => cur = next,
                    None => return false,
                }
            }
            cur.as_bool().unwrap_or(false)
        };

        let header_hit = get_bool(&evt.headers, self.field_pointer);
        let body_hit = get_bool(&evt.body, self.field_pointer);
        header_hit || body_hit
    }
}

#[async_trait]
impl Reflex for PanicLoopReflex {
    fn name(&self) -> &'static str {
        "panic_loop"
    }

    async fn on_event(&mut self, evt: &Envelope) -> Result<ReflexAction, ReflexError> {
        let is_error = self.extract_flag(evt);
        let cnt = self.push(is_error);

        if cnt >= self.min_repeats {
            let msg = format!(
                "PanicLoopReflex: {} error flags in last {} events (pointer: {})",
                cnt, self.window, self.field_pointer
            );
            error!("{}", msg);
            return Ok(ReflexAction::Halt(msg));
        }
        Ok(ReflexAction::Continue)
    }
}

/// Validates a set of cheap policy conditions embedded in headers.
/// Example: `headers.policy.allowed == true` OR `headers.identity.verified == true`.
pub struct PolicyGuardReflex {
    /// List of boolean header JSON pointers that must evaluate to `true`.
    required_true: Vec<&'static str>,
}

impl PolicyGuardReflex {
    pub fn new(required_true: Vec<&'static str>) -> Self {
        Self { required_true }
    }

    fn header_bool(ptr: &str, json: &serde_json::Value) -> bool {
        let mut cur = json;
        for seg in ptr.trim_start_matches('/').split('/') {
            match cur.get(seg) {
                Some(next) => cur = next,
                None => return false,
            }
        }
        cur.as_bool().unwrap_or(false)
    }
}

#[async_trait]
impl Reflex for PolicyGuardReflex {
    fn name(&self) -> &'static str {
        "policy_guard"
    }

    async fn on_event(&mut self, evt: &Envelope) -> Result<ReflexAction, ReflexError> {
        for ptr in &self.required_true {
            if !Self::header_bool(ptr, &evt.headers) {
                let msg = format!("PolicyGuardReflex: required header {} != true", ptr);
                warn!("{}", msg);
                return Ok(ReflexAction::Halt(msg));
            }
        }
        Ok(ReflexAction::Continue)
    }
}
