# RFC-0001: Agent Manifest (AI-Native)

**Status:** Draft  
**Goal:** Define the packaging spec for AI-native agents (not Helm/K8s).

## Document Shape
- `apiVersion`: `openi.ai/v1`
- `kind`: `Agent` | `Model` | `Tool` | `Workflow`
- `metadata`: name, version, annotations
- `spec`:
  - `runtime`: `wasm` | `oci`
  - `source`: artifact URL (registry://… or oci://…)
  - `capabilities`: `inputs[]`, `outputs[]`
  - `routes`: `subscribe[]`, `publish[]`
  - `policies`: `scopes[]`, `maxLatencyMs`, `constraints`
  - `replicas`: int

This manifest is signed at rest and on publish. Provenance is recorded in the registry.
