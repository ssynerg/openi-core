use anyhow::Result;
use openi_core_fabric::Envelope;
use serde::{de::DeserializeOwned, Serialize};

pub struct Agent {
    pub name: String,
    pub version: String,
}

impl Agent {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into() }
    }

    pub async fn publish<T: Serialize>(&self, dest: &str, ctype: &str, payload: T) -> Result<()> {
        let env = Envelope::new(format!("agent://local/{}", self.name), dest.to_string(), ctype.to_string(), payload);
        // TODO: send into local kernel's mailbox (stub)
        println!("[publish] {}", serde_json::to_string(&env)?);
        Ok(())
    }

    pub async fn subscribe<T: DeserializeOwned>(&self, _topic: &str, _handler: fn(Envelope<T>) -> Result<()>) -> Result<()> {
        // TODO: register handler with fabric bus (stub)
        Ok(())
    }
}
