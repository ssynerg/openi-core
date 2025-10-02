# openi-core (AI-Native Infrastructure)

> Not Helm. Not Kubernetes. A Rust kernel + typed fabric for agents, models, and tools.

## What is this?
- **Kernel**: secure runtime for agents (WASM/OCI), identity, policy, scheduling.
- **Fabric**: QUIC-ready, signed envelopes, typed topics.
- **Manifests**: AI-native "Agent Manifests" to ship capabilities (not Pods).

## Quick Start
```bash
# 1) build the workspace
cargo build

# 2) try the CLI
cargo run -p openi-core-cli -- help
cargo run -p openi-core-cli -- init my-agent
cargo run -p openi-core-cli -- package manifests/examples/schema-mapper
cargo run -p openi-core-cli -- deploy manifests/examples/schema-mapper

# 3) start a local node (stub)
cargo run -p openi-core-cli -- node
