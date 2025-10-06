# OpenI Core Fabric — Universal AI Agent Fabric

### Version: 1.2  
**Language:** Rust (core), YAML (manifests)  
**Maintainer:** Synergy Technologies LLC — Shane Sipe  

---

## 🧠 Overview

**OpenI Core Fabric** is the universal runtime and orchestration layer for all OpenI verticals — Healthcare, DevOps, Database Transformation, and beyond.  
It defines a modular, autonomous **Agent Fabric** capable of executing complex enterprise workflows using micro-agents written in Rust with declarative manifests in YAML.  

The fabric is engineered for **sovereign, self-healing, policy-enforced automation** across any domain.  
Each “vertical” is isolated through its own `AgentClass`, adhering to common runtime, policy, and observability contracts.

---

## ⚙️ Architecture Summary

| OpenI Fabric                                                   |
| -------------------------------------------------------------- |
| Rust Crates:                                                   |
| - core-fabric   → Message bus, envelope, signing, content      |
| - core-kernel   → Runtime, policy, identity, RBAC              |
| - core-cli      → CLI orchestration & simulation               |
| ------------------------------------------------------------   |
| Agent Classes (Completed):                                     |
| - HealthcareAgentClass                                         |
| - DevOpsAgentClass                                             |
| - DatabaseAgentClass                                           |
| ------------------------------------------------------------   |
| Agent Classes (In Progress):                                   |
| - AIOpsAgentClass (MLOps / Model Lifecycle)                    |
| +------------------------------------------------------------+ |

---

## ✅ Completed Verticals

### 🩺 **HealthcareAgentClass**
Autonomous clinical and administrative automation: EMR integration, RCM, compliance, triage, and analytics.  
All agents are HIPAA-compliant and communicate through the Rust fabric bus.  

Path: manifests/examples/HealthcareAgentClass/

---

### 🧰 **DevOpsAgentClass**
Full DevSecOps automation covering provisioning, CI/CD, compliance, observability, and resilience.  

**Agents**
- `InfraProvisioner`
- `CICDPipeline`
- `SecretsRotator`
- `ComplianceGate`
- `MonitoringAgent`
- `LogAggregator`
- `IncidentResponder`
- `ChaosEngineer`

Path:manifests/examples/DevOpsAgentClass/

---

### 🧩 **DatabaseAgentClass**
Responsible for schema evolution, ETL, anonymization, and audit logging for transformation workloads.  

**Agents**
- `SchemaMigrator`
- `ETLTransformer`
- `Anonymizer`
- `Tokenizer`
- `DataValidator`
- `AuditLogger`

Path: manifests/examples/DatabaseAgentClass/

---

## 🚧 Current Focus — Stage 4: AI Ops / MLOps AgentClass

The next milestone introduces AI-native operations: model training, deployment, drift detection, and feedback loops directly within the fabric.  

**Planned Agents**
- `ModelTrainer` — trains local & federated micro-LLMs  
- `ModelRegistryAgent` — tracks model versions & metadata  
- `InferenceScaler` — elastic model serving orchestration  
- `DriftDetector` — monitors data and prediction drift  
- `DataFeedbackCollector` — captures live signals for re-training  
- `ExplainabilityAgent` — model interpretability and bias reporting  
- `MLComplianceAgent` — ensures traceability and audit evidence  

This will complete the first universal operational fabric cycle (**OpenI v1.5**).

---

## 🧱 Manifests Layout
manifests/
examples/
HealthcareAgentClass/
DevOpsAgentClass/
DatabaseAgentClass/
AIOpsAgentClass/ ← next in development


Each class directory contains:
- **Blueprint/** → defines agents and relationships  
- **Agent folders/** → each with `AgentManifest.yaml`  
- **Deployment.yaml** → describes runtime deployment  

---

## 🧠 Runtime Fabric (Rust)

| Crate | Purpose |
|-------|----------|
| `core-fabric` | message bus, content envelope, signing |
| `core-kernel` | runtime policy, identity management |
| `core-cli` | CLI entry point & simulation engine |

### Build
```bash
cd openi-core/openi-core
cargo build --release

Run
cargo run --bin openi-core-cli -- agent run \
  --manifest manifests/examples/DevOpsAgentClass/InfraProvisioner/AgentManifest.yaml

🧩 Agent Manifest Standard

All agents adhere to a strict schema:

apiVersion: openi.ai/v1
kind: Agent
metadata:
  name: <agent-name>
  class: <AgentClass>
spec:
  runtime: rust
  image: openi/<agent-name>:<version>
  inputs:
    - type: <event|file|queue|metric>
      format: <json|yaml|ddl|csv|prometheus>
  outputs:
    - type: <event|artifact|dataset>
      format: <json|parquet|yaml>
  policies:
    - name: <policy-name>
      action: <enforce|advisory|alert>
  permissions:
    - <system>: <capability>


This unified schema ensures deployment consistency, security, and observability across all verticals.

🧩 Compliance & Security

Signed manifests and artifacts (SBOM required)

Zero-trust runtime with policy enforcement

Immutable audit evidence stores (SOC2/HIPAA alignment)

Automated secret rotation and encryption of data in transit and at rest

🧪 Testing and Simulation

Run tests

cargo test


Simulate a Blueprint

cargo run --bin openi-core-cli -- simulate \
  --blueprint manifests/examples/DevOpsAgentClass/Blueprint/AgentBlueprint.yaml


Validate YAML

yamllint manifests/examples/**/*.yaml

🚀 Deployment Roadmap
Stage	Milestone	Status
1	HealthcareAgentClass — clinical & admin automation	✅ Complete
2	DevOpsAgentClass — infra, CI/CD, observability, compliance	✅ Complete
3	DatabaseAgentClass — transformation & lineage automation	✅ Complete
4	AI Ops / MLOps AgentClass — model lifecycle automation	🚧 In Progress
5	Governance & Sovereign Ops Layer — federated policy engine	🔮 Planned
🧭 License

Synergy Technologies Internal License v1.0 — proprietary and confidential.


