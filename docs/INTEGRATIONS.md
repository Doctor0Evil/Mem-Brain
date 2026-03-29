# Mem-Brain Integration Guide

**Document Version:** 1.0.0  
**Last Updated:** 2024-01-15  
**Status:** Stable  
**Compatibility:** biomem-core>=1.0.0, biomem-encoder>=1.0.0  

---

## Table of Contents

1. [Overview](#overview)
2. [SKO Wrapping](#sko-wrapping)
3. [ALN Shard Integration](#aln-shard-integration)
4. [Prometheus Metrics Export](#prometheus-metrics-export)
5. [Reality.os Integration](#realityos-integration)
6. [Organichain Integration](#organichain-integration)
7. [Bostrom Simulation Nodes](#bostrom-simulation-nodes)
8. [Data_Lake Storage](#datalake-storage)
9. [Housing-Protection Oversight](#housingprotection-oversight)
10. [Security & Compliance](#security--compliance)
11. [Troubleshooting](#troubleshooting)

---

## Overview

This guide provides implementation details for integrating Mem-Brain MemoryTrace objects with external systems in the Data_Lake ecosystem. All integrations MUST maintain:

| Requirement | Description |
|-------------|-------------|
| **Data Sovereignty** | Augmented-citizen control preserved across all systems |
| **Neurorights Enforcement** | All neurorights validated before cross-system transfer |
| **ALN Compliance** | All shards loaded and validated before operation |
| **Non-Invertibility** | Raw neural data never reconstructible from traces |
| **Monotonicity** | Capability scores never decrease across upgrades |

### Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Mem-Brain Encoder                           │
│  (EEG/Network → MemoryTrace)                                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ALN Shard Validation                         │
│  (biomem-5d, biomem-risk, biomem-evolution, etc.)               │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
    ┌─────────────────┐ ┌───────────┐ ┌──────────────┐
    │   SKO Wrapper   │ │ Prometheus│ │  Data_Lake   │
    │                 │ │  Exporter │ │   Storage    │
    └─────────────────┘ └───────────┘ └──────────────┘
              │                               │
              ▼                               ▼
    ┌─────────────────┐             ┌──────────────────┐
    │   Reality.os    │             │   Organichain    │
    │  Visualization  │             │   Blockchain     │
    └─────────────────┘             └──────────────────┘
              │
              ▼
    ┌─────────────────┐
    │    Bostrom      │
    │   Simulation    │
    └─────────────────┘
```

---

## SKO Wrapping

### Sovereign Knowledge Object Structure

MemoryTrace objects MUST be wrapped as SKOs for cross-system transfer. The SKO wrapper provides encryption, signatures, and metadata for sovereignty enforcement.

### SKO Wrapper Schema

```json
{
  "$schema": "https://data-lake.so/schemas/sko/wrapper-1.0.0.json",
  "type": "object",
  "required": [
    "sko",
    "timestamp",
    "encryption",
    "signature"
  ]
}
```

### SKO Payload Structure

```json
{
  "sko": {
    "modality": {
      "kind": "biomem.trace",
      "version": "1.0.0",
      "schema": "https://data-lake.so/schemas/biomem/trace-1.0.0.json"
    },
    "topic": "neuro.mem-trace",
    "payload": {
      "trace_id": "biomem-1700000000-a3f5e7c9d2b1",
      "roh": 0.12,
      "knowledge_factor": 0.82,
      "safety_strength": 0.88,
      "coordinates": {
        "x": 12.5,
        "y": -8.3,
        "z": 15.7
      },
      "timestamp": 1700000000.0,
      "state_digest": {
        "hex": "003400260043000c00520058",
        "l1_norm": 301
      },
      "alN_shards": [
        "biomem-5d.v1",
        "biomem-risk.v1",
        "biomem-evolution.v1"
      ],
      "neurorights_attestation": {
        "cognitive_liberty": true,
        "mental_privacy": true,
        "psychological_continuity": true,
        "right_to_audit": true
      }
    },
    "metadata": {
      "housing_protection": "protected",
      "sovereignty_class": "sovereign",
      "export_classification": "local_network",
      "curatorial_review_status": "approved",
      "pof_evidence_hash": "sha256:abc123..."
    }
  },
  "timestamp": 1700000000.0,
  "encryption": {
    "algorithm": "aes256_gcm",
    "key_id": "key-uuid-here",
    "nonce": "base64_encoded_nonce"
  },
  "signature": {
    "algorithm": "ed25519",
    "public_key": "base64_encoded_pubkey",
    "signature": "base64_encoded_signature"
  }
}
```

### SKO Wrapping Implementation (Rust)

```rust
use biomem_core::MemoryTrace;
use serde::{Serialize, Deserialize};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use ed25519_dalek::{Signer, SigningKey};

#[derive(Serialize, Deserialize)]
pub struct SKOWrapper {
    pub sko: SKO,
    pub timestamp: f64,
    pub encryption: EncryptionMetadata,
    pub signature: SignatureMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct SKO {
    pub modality: Modality,
    pub topic: String,
    pub payload: SKOPayload,
    pub metadata: SKOMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct Modality {
    pub kind: String,
    pub version: String,
    pub schema: String,
}

#[derive(Serialize, Deserialize)]
pub struct SKOPayload {
    pub trace_id: String,
    pub roh: f32,
    pub knowledge_factor: f32,
    pub safety_strength: f32,
    pub coordinates: Coordinates,
    pub timestamp: f64,
    pub state_digest: StateDigest,
    pub aln_shards: Vec<String>,
    pub neurorights_attestation: NeurorightsAttestation,
}

#[derive(Serialize, Deserialize)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize)]
pub struct StateDigest {
    pub hex: String,
    pub l1_norm: u64,
}

#[derive(Serialize, Deserialize)]
pub struct NeurorightsAttestation {
    pub cognitive_liberty: bool,
    pub mental_privacy: bool,
    pub psychological_continuity: bool,
    pub right_to_audit: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SKOMetadata {
    pub housing_protection: String,
    pub sovereignty_class: String,
    pub export_classification: String,
    pub curatorial_review_status: String,
    pub pof_evidence_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub algorithm: String,
    pub key_id: String,
    pub nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
}

pub fn wrap_trace_as_sko(
    trace: &MemoryTrace,
    signing_key: &SigningKey,
    encryption_key: &Key<Aes256Gcm>,
) -> Result<SKOWrapper, Box<dyn std::error::Error>> {
    use rand::RngCore;
    use aes_gcm::AeadInPlace;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // Build SKO payload from trace
    let payload = SKOPayload {
        trace_id: trace.id.clone(),
        roh: trace.roh,
        knowledge_factor: trace.knowledge_factor,
        safety_strength: trace.safety_strength,
        coordinates: Coordinates {
            x: trace.coord.space.x,
            y: trace.coord.space.y,
            z: trace.coord.space.z,
        },
        timestamp: trace.created_at,
        state_digest: StateDigest {
            hex: trace.state_hex.hex.clone(),
            l1_norm: trace.state_hex.l1_norm,
        },
        aln_shards: vec![
            "biomem-5d.v1".to_string(),
            "biomem-risk.v1".to_string(),
            "biomem-evolution.v1".to_string(),
        ],
        neurorights_attestation: NeurorightsAttestation {
            cognitive_liberty: true,
            mental_privacy: true,
            psychological_continuity: true,
            right_to_audit: true,
        },
    };

    // Serialize payload for encryption
    let mut payload_bytes = serde_json::to_vec(&payload)?;
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt payload
    let cipher = Aes256Gcm::new(encryption_key);
    cipher.encrypt_in_place(nonce, b"", &mut payload_bytes)?;
    
    // Sign the encrypted payload
    let signature = signing_key.sign(&payload_bytes);
    
    // Build wrapper
    let wrapper = SKOWrapper {
        sko: SKO {
            modality: Modality {
                kind: "biomem.trace".to_string(),
                version: trace.version.clone(),
                schema: "https://data-lake.so/schemas/biomem/trace-1.0.0.json".to_string(),
            },
            topic: "neuro.mem-trace".to_string(),
            payload, // Note: In production, store encrypted bytes instead
            metadata: SKOMetadata {
                housing_protection: "protected".to_string(),
                sovereignty_class: "sovereign".to_string(),
                export_classification: "local_network".to_string(),
                curatorial_review_status: "approved".to_string(),
                pof_evidence_hash: format!("sha256:{}", trace.id),
            },
        },
        timestamp: trace.created_at,
        encryption: EncryptionMetadata {
            algorithm: "aes256_gcm".to_string(),
            key_id: "key-uuid-placeholder".to_string(),
            nonce: STANDARD.encode(nonce_bytes),
        },
        signature: SignatureMetadata {
            algorithm: "ed25519".to_string(),
            public_key: STANDARD.encode(signing_key.verifying_key().as_bytes()),
            signature: STANDARD.encode(signature.to_bytes()),
        },
    };

    Ok(wrapper)
}
```

### SKO Validation Checklist

| Check | Description | Enforcement |
|-------|-------------|-------------|
| Signature Valid | ED25519 signature verifies | Hard |
| Encryption Valid | AES256-GCM decryption succeeds | Hard |
| Schema Valid | Payload matches JSON schema | Hard |
| ALN Shards Present | Required shards listed | Hard |
| Neurorights Attested | All rights explicitly stated | Hard |
| RoH Within Ceiling | `roh <= 0.30` | Hard |
| Safety Above Floor | `safety_strength >= 0.75` | Hard |
| Knowledge Above Floor | `knowledge_factor >= 0.50` | Hard |

---

## ALN Shard Integration

### Required Shards for MemoryTrace Validation

| Shard | Path | Purpose | Enforcement |
|-------|------|---------|-------------|
| `biomem-5d.aln` | `/spec/biomem-5d.aln` | 5-D coordinate & layer spec | Hard |
| `biomem-evolution.aln` | `/spec/biomem-evolution.aln` | Monotonicity & PoF/NFA | Hard |
| `biomem-risk.aln` | `/spec/biomem-risk.aln` | RoH & safety constraints | Hard |
| `biomem.layer.microcircuit.aln` | `/aln/biomem.layer.microcircuit.aln` | Microcircuit features | Soft |
| `biomem.hypothesis.h1.aln` | `/aln/biomem.hypothesis.h1.aln` | H1 evidence validation | Soft |

### Shard Loading Sequence

```rust
use std::fs;
use std::path::Path;

pub struct ALNShardLoader {
    loaded_shards: Vec<String>,
    shard_registry: std::collections::HashMap<String, ShardMetadata>,
}

pub struct ShardMetadata {
    pub version: String,
    pub compatibility: Vec<String>,
    pub enforcement: String, // "hard" or "soft"
    pub loaded_at: f64,
}

impl ALNShardLoader {
    pub fn new() -> Self {
        Self {
            loaded_shards: Vec::new(),
            shard_registry: std::collections::HashMap::new(),
        }
    }

    pub fn load_shard(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        
        // Parse ALN shard header
        let version = self.extract_version(&content)?;
        let compatibility = self.extract_compatibility(&content)?;
        let enforcement = self.extract_enforcement(&content)?;
        
        let metadata = ShardMetadata {
            version,
            compatibility,
            enforcement,
            loaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        
        let shard_id = self.extract_shard_id(&content)?;
        self.shard_registry.insert(shard_id.clone(), metadata);
        self.loaded_shards.push(shard_id);
        
        Ok(())
    }

    pub fn validate_trace(&self, trace: &MemoryTrace) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check required hard shards
        let required_shards = [
            "biomem.5d.v1",
            "biomem.evolution.v1",
            "biomem.risk.v1",
        ];
        
        for shard_id in required_shards.iter() {
            if !self.loaded_shards.iter().any(|s| s.starts_with(shard_id)) {
                errors.push(format!("Required shard not loaded: {}", shard_id));
            }
        }
        
        // Validate against each loaded shard
        for (shard_id, metadata) in &self.shard_registry {
            let result = self.validate_against_shard(trace, shard_id, metadata);
            if metadata.enforcement == "hard" && !result.valid {
                errors.extend(result.errors);
            } else if !result.valid {
                warnings.extend(result.errors);
            }
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    fn extract_version(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse @version directive from ALN content
        for line in content.lines() {
            if line.starts_with("@version") {
                return Ok(line.split_whitespace().nth(1)
                    .ok_or("Invalid version format")?
                    .to_string());
            }
        }
        Err("Version not found in shard".into())
    }

    fn extract_compatibility(&self, content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Parse @compatibility directive from ALN content
        for line in content.lines() {
            if line.starts_with("@compatibility") {
                let deps = line.split_whitespace().skip(1)
                    .map(|s| s.to_string())
                    .collect();
                return Ok(deps);
            }
        }
        Ok(Vec::new())
    }

    fn extract_enforcement(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Default enforcement based on shard type
        if content.contains("biomem-5d") || content.contains("biomem-risk") {
            Ok("hard".to_string())
        } else {
            Ok("soft".to_string())
        }
    }

    fn extract_shard_id(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        for line in content.lines() {
            if line.starts_with("@shard") {
                return Ok(line.split_whitespace().nth(1)
                    .ok_or("Invalid shard ID format")?
                    .to_string());
            }
        }
        Err("Shard ID not found".into())
    }

    fn validate_against_shard(
        &self,
        trace: &MemoryTrace,
        shard_id: &str,
        _metadata: &ShardMetadata,
    ) -> ValidationResult {
        // Shard-specific validation logic
        let mut errors = Vec::new();
        
        match shard_id {
            id if id.contains("biomem.5d") => {
                if trace.roh > 0.30 {
                    errors.push("RoH exceeds 5D shard ceiling".to_string());
                }
            }
            id if id.contains("biomem.risk") => {
                if trace.safety_strength < 0.75 {
                    errors.push("Safety below risk shard floor".to_string());
                }
            }
            id if id.contains("biomem.evolution") => {
                if trace.knowledge_factor < 0.50 {
                    errors.push("Knowledge below evolution shard floor".to_string());
                }
            }
            _ => {}
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }
}

pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### Shard Validation Flow

```
1. Load required shards from /spec/ and /aln/ directories
2. Parse shard metadata (version, compatibility, enforcement)
3. Verify shard compatibility with biomem-core version
4. For each MemoryTrace:
   a. Validate against hard-enforcement shards
   b. Validate against soft-enforcement shards
   c. Collect errors and warnings
   d. Reject if any hard shard validation fails
   e. Flag for review if soft shard validation fails
5. Record validation results in audit log
6. Export validation metrics to Prometheus
```

---

## Prometheus Metrics Export

### Required Metrics

| Metric Name | Type | Description | Labels |
|-------------|------|-------------|--------|
| `biomem_encode_count_total` | Counter | Total successful encodings | `encoder_id`, `version` |
| `biomem_roh_current` | Gauge | Current Risk-of-Harm score | `trace_id`, `roi` |
| `biomem_knowledge_factor_current` | Gauge | Current knowledge factor | `trace_id` |
| `biomem_safety_strength_current` | Gauge | Current safety strength | `trace_id` |
| `biomem_complexity_current` | Gauge | Current trace complexity | `trace_id` |
| `biomem_nullspace_dim_current` | Gauge | Current null-space dimension | `trace_id` |
| `biomem_validation_errors_total` | Counter | Validation error count | `error_type`, `shard_id` |
| `biomem_sko_wrapped_total` | Counter | SKO wrapping count | `modality` |
| `biomem_export_blocked_total` | Counter | Blocked export attempts | `reason` |
| `biomem_neurorights_violations_total` | Counter | Neurorights violation count | `right_type` |

### Prometheus Exporter Implementation (Rust)

```rust
use prometheus::{Registry, Gauge, Counter, Histogram, Opts};
use std::sync::Arc;

pub struct PrometheusExporter {
    registry: Registry,
    encode_count: Counter,
    roh_gauge: Gauge,
    knowledge_gauge: Gauge,
    safety_gauge: Gauge,
    complexity_gauge: Gauge,
    nullspace_gauge: Gauge,
    validation_errors: Counter,
    sko_wrapped: Counter,
    export_blocked: Counter,
    neurorights_violations: Counter,
}

impl PrometheusExporter {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        let encode_count = Counter::with_opts(Opts::new(
            "biomem_encode_count_total",
            "Total number of successful encodings"
        ))?;
        
        let roh_gauge = Gauge::with_opts(Opts::new(
            "biomem_roh_current",
            "Current Risk-of-Harm score"
        ))?;
        
        let knowledge_gauge = Gauge::with_opts(Opts::new(
            "biomem_knowledge_factor_current",
            "Current knowledge factor"
        ))?;
        
        let safety_gauge = Gauge::with_opts(Opts::new(
            "biomem_safety_strength_current",
            "Current safety strength"
        ))?;
        
        let complexity_gauge = Gauge::with_opts(Opts::new(
            "biomem_complexity_current",
            "Current trace complexity score"
        ))?;
        
        let nullspace_gauge = Gauge::with_opts(Opts::new(
            "biomem_nullspace_dim_current",
            "Current null-space dimension"
        ))?;
        
        let validation_errors = Counter::with_opts(Opts::new(
            "biomem_validation_errors_total",
            "Total validation errors"
        ))?;
        
        let sko_wrapped = Counter::with_opts(Opts::new(
            "biomem_sko_wrapped_total",
            "Total SKO wrapping operations"
        ))?;
        
        let export_blocked = Counter::with_opts(Opts::new(
            "biomem_export_blocked_total",
            "Total blocked export attempts"
        ))?;
        
        let neurorights_violations = Counter::with_opts(Opts::new(
            "biomem_neurorights_violations_total",
            "Total neurorights violations"
        ))?;
        
        registry.register(Box::new(encode_count.clone()))?;
        registry.register(Box::new(roh_gauge.clone()))?;
        registry.register(Box::new(knowledge_gauge.clone()))?;
        registry.register(Box::new(safety_gauge.clone()))?;
        registry.register(Box::new(complexity_gauge.clone()))?;
        registry.register(Box::new(nullspace_gauge.clone()))?;
        registry.register(Box::new(validation_errors.clone()))?;
        registry.register(Box::new(sko_wrapped.clone()))?;
        registry.register(Box::new(export_blocked.clone()))?;
        registry.register(Box::new(neurorights_violations.clone()))?;
        
        Ok(Self {
            registry,
            encode_count,
            roh_gauge,
            knowledge_gauge,
            safety_gauge,
            complexity_gauge,
            nullspace_gauge,
            validation_errors,
            sko_wrapped,
            export_blocked,
            neurorights_violations,
        })
    }

    pub fn record_encode(&self) {
        self.encode_count.inc();
    }

    pub fn record_trace_metrics(&self, trace: &MemoryTrace) {
        self.roh_gauge.set(trace.roh as f64);
        self.knowledge_gauge.set(trace.knowledge_factor as f64);
        self.safety_gauge.set(trace.safety_strength as f64);
        self.complexity_gauge.set(trace.complexity_score() as f64);
        self.nullspace_gauge.set(trace.nullspace_dim() as f64);
    }

    pub fn record_validation_error(&self, error_type: &str) {
        self.validation_errors.inc();
    }

    pub fn record_sko_wrap(&self) {
        self.sko_wrapped.inc();
    }

    pub fn record_export_blocked(&self, reason: &str) {
        self.export_blocked.inc();
    }

    pub fn record_neurorights_violation(&self, right_type: &str) {
        self.neurorights_violations.inc();
    }

    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// HTTP endpoint handler for Prometheus scraping
pub async fn metrics_handler(
    exporter: Arc<PrometheusExporter>,
) -> Result<String, std::fmt::Error> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = exporter.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).unwrap())
}
```

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'biomem-encoder'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    
  - job_name: 'biomem-validator'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: '/metrics'
    
  - job_name: 'biomem-sko-wrapper'
    static_configs:
      - targets: ['localhost:9092']
    metrics_path: '/metrics'

rule_files:
  - 'biomem_alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### Alert Rules

```yaml
# biomem_alerts.yml
groups:
  - name: biomem_alerts
    rules:
      - alert: RoHCeilingBreached
        expr: biomem_roh_current > 0.30
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "RoH ceiling breached"
          description: "Trace RoH {{ $value }} exceeds maximum 0.30"
          
      - alert: SafetyStrengthLow
        expr: biomem_safety_strength_current < 0.75
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Safety strength below minimum"
          description: "Trace safety {{ $value }} below minimum 0.75"
          
      - alert: NeurorightsViolation
        expr: increase(biomem_neurorights_violations_total[1h]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Neurorights violation detected"
          description: "{{ $value }} violations in the last hour"
          
      - alert: ValidationErrorsHigh
        expr: increase(biomem_validation_errors_total[5m]) > 10
        for: 0m
        labels:
          severity: warning
        annotations:
          summary: "High validation error rate"
          description: "{{ $value }} errors in the last 5 minutes"
```

---

## Reality.os Integration

### Visualization Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/trace/{id}` | GET | Retrieve trace for 3D rendering |
| `/api/v1/trace/{id}/render` | GET | Get render-ready 3D data |
| `/api/v1/layer/{layer_id}/activate` | POST | Activate layer visualization |
| `/api/v1/roi/{roi_id}/highlight` | POST | Highlight ROI on brain atlas |
| `/api/v1/timeline/{trace_id}` | GET | Get temporal evolution data |

### Reality.os Trace Rendering Schema

```json
{
  "trace_id": "biomem-1700000000-a3f5e7c9d2b1",
  "render_data": {
    "coordinates": {
      "x": 12.5,
      "y": -8.3,
      "z": 15.7,
      "atlas": "MNI152"
    },
    "layers": [
      {
        "name": "microcircuit",
        "index": 2,
        "activation": 0.52,
        "color": "#4A90E2"
      },
      {
        "name": "network",
        "index": 3,
        "activation": 0.67,
        "color": "#50C878"
      },
      {
        "name": "systemic",
        "index": 4,
        "activation": 0.82,
        "color": "#FF6B6B"
      }
    ],
    "temporal": {
      "start": 1700000000.0,
      "end": 1700003600.0,
      "resolution_ms": 100
    },
    "metadata": {
      "roh": 0.12,
      "knowledge_factor": 0.82,
      "safety_strength": 0.88,
      "housing_protection": "protected"
    }
  },
  "interaction_modes": [
    "inspect_trace",
    "compare_sessions",
    "view_hypothesis_status",
    "export_time_window"
  ]
}
```

### Reality.os WebSocket Connection

```javascript
// Reality.os client-side integration
const ws = new WebSocket('wss://reality.os/api/v1/biomem/stream');

ws.onopen = () => {
  console.log('Connected to Reality.os biomem stream');
  
  // Subscribe to trace updates
  ws.send(JSON.stringify({
    action: 'subscribe',
    channel: 'biomem.trace',
    filters: {
      min_knowledge_factor: 0.50,
      max_roh: 0.30
    }
  }));
};

ws.onmessage = (event) => {
  const trace = JSON.parse(event.data);
  renderTrace3D(trace);
  updateLayerActivations(trace.layers);
  updateTimeline(trace.temporal);
};

function renderTrace3D(trace) {
  // Three.js or Unity integration for 3D brain rendering
  const marker = create3DMarker({
    position: [trace.coordinates.x, trace.coordinates.y, trace.coordinates.z],
    color: getLayerColor(trace.layers),
    size: getActivationSize(trace.layers)
  });
  brainScene.add(marker);
}
```

---

## Organichain Integration

### On-Chain Trace Registration

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract BiomemTraceRegistry {
    struct TraceRecord {
        string traceId;
        bytes32 stateHash;
        uint256 timestamp;
        address owner;
        bool exists;
        uint8 roh; // scaled by 100
        uint8 knowledgeFactor; // scaled by 100
        uint8 safetyStrength; // scaled by 100
    }
    
    mapping(bytes32 => TraceRecord) public traces;
    mapping(address => bytes32[]) public ownerTraces;
    
    event TraceRegistered(
        bytes32 indexed traceHash,
        string traceId,
        address indexed owner,
        uint256 timestamp
    );
    
    event TraceValidated(
        bytes32 indexed traceHash,
        bool valid,
        uint8 validationScore
    );
    
    function registerTrace(
        string memory traceId,
        bytes32 stateHash,
        uint8 roh,
        uint8 knowledgeFactor,
        uint8 safetyStrength
    ) external returns (bool) {
        require(roh <= 30, "RoH exceeds ceiling");
        require(knowledgeFactor >= 50, "Knowledge below floor");
        require(safetyStrength >= 75, "Safety below floor");
        
        bytes32 traceHash = keccak256(abi.encodePacked(traceId));
        require(!traces[traceHash].exists, "Trace already registered");
        
        traces[traceHash] = TraceRecord({
            traceId: traceId,
            stateHash: stateHash,
            timestamp: block.timestamp,
            owner: msg.sender,
            exists: true,
            roh: roh,
            knowledgeFactor: knowledgeFactor,
            safetyStrength: safetyStrength
        });
        
        ownerTraces[msg.sender].push(traceHash);
        
        emit TraceRegistered(traceHash, traceId, msg.sender, block.timestamp);
        
        return true;
    }
    
    function validateTrace(
        bytes32 traceHash,
        bool valid,
        uint8 validationScore
    ) external {
        require(traces[traceHash].exists, "Trace not found");
        emit TraceValidated(traceHash, valid, validationScore);
    }
    
    function getTrace(bytes32 traceHash) 
        external 
        view 
        returns (
            string memory traceId,
            bytes32 stateHash,
            uint256 timestamp,
            address owner,
            uint8 roh,
            uint8 knowledgeFactor,
            uint8 safetyStrength
        ) 
    {
        TraceRecord memory record = traces[traceHash];
        require(record.exists, "Trace not found");
        
        return (
            record.traceId,
            record.stateHash,
            record.timestamp,
            record.owner,
            record.roh,
            record.knowledgeFactor,
            record.safetyStrength
        );
    }
}
```

### Organichain Trace Submission (Rust)

```rust
use web3::{Web3, transports::Http, types::Address};
use ethabi::Token;

pub struct OrganichainSubmitter {
    web3: Web3<Http>,
    contract_address: Address,
    sender_address: Address,
}

impl OrganichainSubmitter {
    pub async fn submit_trace(
        &self,
        trace: &MemoryTrace,
    ) -> Result<web3::types::H256, Box<dyn std::error::Error>> {
        // Scale values for Solidity uint8
        let roh_scaled = (trace.roh * 100.0) as u8;
        let knowledge_scaled = (trace.knowledge_factor * 100.0) as u8;
        let safety_scaled = (trace.safety_strength * 100.0) as u8;
        
        // Compute state hash
        let state_hash = self.compute_state_hash(trace)?;
        
        // Call smart contract
        let tx_hash = self.call_contract(
            "registerTrace",
            vec![
                Token::String(trace.id.clone()),
                Token::FixedBytes(state_hash.to_vec()),
                Token::Uint(rho_scaled.into()),
                Token::Uint(knowledge_scaled.into()),
                Token::Uint(safety_scaled.into()),
            ],
        ).await?;
        
        Ok(tx_hash)
    }

    fn compute_state_hash(&self, trace: &MemoryTrace) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(trace.state_hex.hex.as_bytes());
        hasher.update(trace.id.as_bytes());
        let result = hasher.finalize();
        Ok(result.into())
    }

    async fn call_contract(
        &self,
        function_name: &str,
        tokens: Vec<Token>,
    ) -> Result<web3::types::H256, Box<dyn std::error::Error>> {
        // Implementation depends on web3.rs contract binding
        // This is a simplified example
        Ok(web3::types::H256::zero())
    }
}
```

---

## Bostrom Simulation Nodes

### Simulation Node Configuration

```yaml
# bostrom_node_config.yml
node:
  id: bostrom-sim-001
  type: simulation
  isolation: air_gapped
  export_allowed: false
  
biomem_integration:
  enabled: true
  trace_validation: required
  aln_shards:
    - biomem-5d.v1
    - biomem-risk.v1
  max_roh: 0.30
  min_safety: 0.75
  
simulation:
  type: hypothetical_modeling
  sandbox: mandatory
  logging: complete
  raw_data_access: denied
  
security:
  network_isolation: true
  data_egress_filter: strict
  audit_logging: enabled
  access_control: multi_factor
```

### Bostrom Trace Loading

```rust
pub struct BostromSimulationNode {
    config: BostromConfig,
    loaded_traces: Vec<MemoryTrace>,
    simulation_state: SimulationState,
}

impl BostromSimulationNode {
    pub fn load_trace(&mut self, trace: MemoryTrace) -> Result<(), SimulationError> {
        // Validate trace before loading
        if trace.roh > self.config.max_roh {
            return Err(SimulationError::RoHExceeded);
        }
        if trace.safety_strength < self.config.min_safety {
            return Err(SimulationError::SafetyBelowMinimum);
        }
        
        // Verify ALN shard compliance
        if !self.validate_aln_compliance(&trace) {
            return Err(SimulationError::ALNNonCompliant);
        }
        
        // Load trace into simulation (non-invertible)
        self.loaded_traces.push(trace);
        self.simulation_state.trace_count += 1;
        
        Ok(())
    }

    pub fn run_simulation(&mut self) -> SimulationResult {
        // Run hypothetical modeling with loaded traces
        // No raw data access permitted
        // All outputs are aggregated/anonymized
        SimulationResult {
            trace_count: self.simulation_state.trace_count,
            aggregate_metrics: self.compute_aggregate_metrics(),
            raw_data_exposed: false,
        }
    }

    fn validate_aln_compliance(&self, trace: &MemoryTrace) -> bool {
        // Check all required ALN shards
        trace.validate_all()
    }

    fn compute_aggregate_metrics(&self) -> AggregateMetrics {
        // Compute metrics without exposing individual traces
        AggregateMetrics {
            avg_roh: self.loaded_traces.iter()
                .map(|t| t.roh)
                .sum::<f32>() / self.loaded_traces.len() as f32,
            avg_knowledge: self.loaded_traces.iter()
                .map(|t| t.knowledge_factor)
                .sum::<f32>() / self.loaded_traces.len() as f32,
            total_traces: self.loaded_traces.len(),
        }
    }
}
```

---

## Data_Lake Storage

### Storage Class Configuration

| Class | RoH Range | Encryption | Backup | Access Control |
|-------|-----------|------------|--------|----------------|
| `standard` | [0.0, 0.20] | AES256 | Daily | Capability-based |
| `protected` | (0.20, 0.30] | AES256-GCM | Hourly | Capability-based + Audit |
| `restricted` | (0.30, 0.50] | AES256-GCM | Real-time | Multi-factor + Curator |
| `quarantined` | (0.50, 1.0] | AES256-GCM | Real-time | Air-gapped + Curator |

### Data_Lake Storage API

```rust
pub struct DataLakeStorage {
    client: StorageClient,
    bucket: String,
}

impl DataLakeStorage {
    pub async fn store_trace(
        &self,
        trace: &MemoryTrace,
        sko_wrapper: &SKOWrapper,
    ) -> Result<String, StorageError> {
        // Determine storage class based on RoH
        let storage_class = self.classify_storage(trace.roh)?;
        
        // Generate storage key
        let key = format!("biomem/traces/{}/{}.sko", trace.created_at, trace.id);
        
        // Serialize SKO wrapper
        let bytes = serde_json::to_vec(sko_wrapper)?;
        
        // Store with appropriate metadata
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(bytes.into())
            .metadata("trace_id", &trace.id)
            .metadata("roh", &trace.roh.to_string())
            .metadata("storage_class", &storage_class)
            .metadata("encryption", "aes256_gcm")
            .send()
            .await?;
        
        Ok(key)
    }

    fn classify_storage(&self, roh: f32) -> Result<String, StorageError> {
        match roh {
            r if r <= 0.20 => Ok("standard".to_string()),
            r if r <= 0.30 => Ok("protected".to_string()),
            r if r <= 0.50 => Ok("restricted".to_string()),
            _ => Ok("quarantined".to_string()),
        }
    }

    pub async fn retrieve_trace(&self, key: &str) -> Result<SKOWrapper, StorageError> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        
        let bytes = response.body.collect().await?.into_bytes();
        let sko: SKOWrapper = serde_json::from_slice(&bytes)?;
        
        Ok(sko)
    }

    pub async fn delete_trace(&self, key: &str) -> Result<(), StorageError> {
        // Note: Deletion subject to sovereignty rights
        // Monotonicity must be preserved
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        
        Ok(())
    }
}
```

---

## Housing-Protection Oversight

### Oversight Dashboard Configuration

```yaml
# housing_protection_dashboard.yml
dashboard:
  title: "Housing-Protection Oversight"
  refresh_interval: 15s
  
panels:
  - title: "RoH Distribution"
    type: histogram
    metric: biomem_roh_current
    buckets: 20
    
  - title: "Storage Class Breakdown"
    type: pie_chart
    metrics:
      - storage_class_standard
      - storage_class_protected
      - storage_class_restricted
      - storage_class_quarantined
      
  - title: "Validation Errors"
    type: time_series
    metric: biomem_validation_errors_total
    time_range: 24h
    
  - title: "Export Attempts"
    type: table
    columns:
      - trace_id
      - timestamp
      - destination
      - status
      - blocker_reason
      
  - title: "Neurorights Compliance"
    type: stat
    metrics:
      - biomem_neurorights_violations_total
      - biomem_consent_expirations_soon
      
  - title: "Curatorial Review Queue"
    type: table
    metric: biomem_curator_reviews_pending
    columns:
      - trace_id
      - submitted_at
      - review_type
      - assigned_reviewer
      - status
```

### Oversight Alert Thresholds

| Alert | Metric | Threshold | Severity | Action |
|-------|--------|-----------|----------|--------|
| RoH Breach | `biomem_roh_current` | > 0.30 | Critical | Quarantine trace |
| Safety Low | `biomem_safety_strength_current` | < 0.75 | Warning | Block export |
| Validation Spike | `biomem_validation_errors_total` | > 10/5min | Warning | Investigate |
| Neurorights Violation | `biomem_neurorights_violations_total` | > 0/1h | Critical | Audit required |
| Review Queue Backlog | `biomem_curator_reviews_pending` | > 50 | Warning | Scale reviewers |

---

## Security & Compliance

### Encryption Requirements

| Data State | Algorithm | Key Length | Mode |
|------------|-----------|------------|------|
| At Rest | AES | 256-bit | GCM |
| In Transit | TLS | 256-bit | 1.3 |
| Signatures | ED25519 | 256-bit | Deterministic |
| Hashes | SHA-256 | 256-bit | N/A |

### Access Control Matrix

| Role | Read | Write | Export | Delete | Audit |
|------|------|-------|--------|--------|-------|
| Citizen Owner | ✅ | ✅ | ✅ | ✅ | ✅ |
| Researcher | ⚠️ | ❌ | ⚠️ | ❌ | ✅ |
| Curator | ✅ | ⚠️ | ✅ | ⚠️ | ✅ |
| Auditor | ✅ | ❌ | ❌ | ❌ | ✅ |
| System | ✅ | ✅ | ⚠️ | ❌ | ✅ |

⚠️ = Conditional on consent/approval

### Compliance Checklist

| Requirement | Standard | Enforcement |
|-------------|----------|-------------|
| Data Minimization | GDPR Art. 5(1)(c) | ALN shards |
| Purpose Limitation | GDPR Art. 5(1)(b) | SKO metadata |
| Storage Limitation | GDPR Art. 5(1)(e) | Retention policies |
| Integrity & Confidentiality | GDPR Art. 5(1)(f) | Encryption |
| Accountability | GDPR Art. 5(2) | Audit logs |
| Right to Access | GDPR Art. 15 | Citizen portal |
| Right to Portability | GDPR Art. 20 | Export API |
| Right to Erasure | GDPR Art. 17 | Delete API (monotonicity preserved) |

---

## Troubleshooting

### Common Issues

| Issue | Symptom | Resolution |
|-------|---------|------------|
| Shard Load Failure | `ALN shard not found` | Verify shard path in `/spec/` or `/aln/` |
| RoH Validation Fail | `RoH exceeds ceiling` | Check encoder RoH calculation |
| Safety Floor Fail | `Safety below minimum` | Verify safety_strength input >= 0.75 |
| SKO Signature Invalid | `Signature verification failed` | Regenerate with correct key pair |
| Prometheus Scrape Fail | `Target unreachable` | Check exporter port and firewall |
| Organichain TX Fail | `Transaction reverted` | Check gas limit and contract state |
| Reality.os Render Fail | `3D marker not visible` | Verify coordinate atlas alignment |

### Debug Commands

```bash
# Validate ALN shards
aln-cli validate /spec/biomem-5d.aln
aln-cli validate /spec/biomem-risk.aln

# Check trace validation
biomem-cli validate --trace trace.json --shards /spec/

# Export Prometheus metrics
curl http://localhost:9090/metrics | grep biomem

# Verify SKO wrapper
sko-cli verify --input wrapped.sko --pubkey public.pem

# Check Organichain registration
web3-cli call --contract 0x... --function getTrace --args [trace_hash]
```

### Support Channels

| Channel | Purpose | Response Time |
|---------|---------|---------------|
| GitHub Issues | Bug reports, feature requests | 48 hours |
| Discord | Community support | 24 hours |
| Email (security) | Security vulnerabilities | 4 hours |
| Curatorial Board | NFA claims, exceptions | 14 days |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial release |
| 1.0.1 | planned_Q2_2024 | Add Organichain smart contract examples |
| 1.1.0 | planned_Q3_2024 | Add streaming integration support |

---

## References

- **FORMAT.md:** `/docs/FORMAT.md`
- **ALN Shards:** `/spec/`, `/aln/`
- **Rust Crates:** `/rust/crates/`
- **Examples:** `/examples/`
- **License:** MIT (Data_Lake Sovereign License)

---

*This document is part of the Mem-Brain protocol specification. All implementations MUST maintain compatibility with biomem-core>=1.0.0.*
