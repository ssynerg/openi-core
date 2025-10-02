use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use time::OffsetDateTime;
use ulid::Ulid;

use crate::content::ContentType;

pub type Headers = BTreeMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T = serde_json::Value> {
    pub v: u8,                 // schema version
    pub id: String,            // ULID
    pub src: String,           // agent://tenant/node/agent
    pub dest: String,          // topic://... or agent://...
    pub ts: String,            // RFC3339
    pub ctype: String,         // ContentType
    #[serde(default)]
    pub headers: Headers,      // trace_id, ttl_ms, scopes, etc.
    pub payload: T,            // typed payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,   // base64 signature
}

impl<T: Serialize + for<'de> Deserialize<'de>> Envelope<T> {
    pub fn new(src: impl Into<String>, dest: impl Into<String>, ctype: impl Into<String>, payload: T) -> Self {
        Self {
            v: 1,
            id: Ulid::new().to_string(),
            src: src.into(),
            dest: dest.into(),
            ts: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
            ctype: ctype.into(),
            headers: Headers::new(),
            payload,
            sig: None,
        }
    }

    pub fn with_header(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.headers.insert(k.into(), v.into());
        self
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        // Sign without the signature field
        let mut clone = self.clone();
        clone.sig = None;
        serde_json::to_vec(&clone).expect("serialize")
    }
}
