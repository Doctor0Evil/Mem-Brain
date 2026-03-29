# MemoryTrace Format Specification

**Document Version:** 1.0.0  
**Last Updated:** 2024-01-15  
**Status:** Stable  
**Compatibility:** biomem-core>=1.0.0, biomem-encoder>=1.0.0  

---

## Table of Contents

1. [Overview](#overview)
2. [MemoryTrace Schema](#memorytrace-schema)
3. [EegNetworkSample Schema](#eegnetworksample-schema)
4. [Coordinate System](#coordinate-system)
5. [Biophysical Layers](#biophysical-layers)
6. [ALN Field Mapping](#aln-field-mapping)
7. [SKO Field Mapping](#sko-field-mapping)
8. [Validation Rules](#validation-rules)
9. [Examples](#examples)
10. [Version History](#version-history)

---

## Overview

This document defines the canonical JSON schema for `MemoryTrace` and `EegNetworkSample` objects used throughout the Mem-Brain protocol. All implementations MUST adhere to this specification to ensure interoperability across ALN-Blockchain nodes, Prometheus monitoring systems, Reality.os visualizations, and Data_Lake storage layers.

### Design Principles

| Principle | Description |
|-----------|-------------|
| **Non-Invertible** | Traces cannot be reverse-engineered to raw neural data |
| **Monotonic** | Complexity scores never decrease with updates |
| **Machine-Readable** | All fields parseable without human interpretation |
| **ALN-Compliant** | All fields map to ALN shard specifications |
| **SKO-Wrappable** | All traces can be wrapped as Sovereign Knowledge Objects |

### Encoding Requirements

- **Character Encoding:** UTF-8
- **Number Format:** IEEE 754 floating-point (f32/f64)
- **Timestamp Format:** Unix epoch seconds (f64)
- **String Format:** Unicode, max 1024 characters unless specified
- **Array Format:** JSON arrays with homogeneous element types

---

## MemoryTrace Schema

### Root Object

```json
{
  "$schema": "https://data-lake.so/schemas/biomem/trace-1.0.0.json",
  "type": "object",
  "required": [
    "id",
    "coord",
    "state_hex",
    "roh",
    "knowledge_factor",
    "safety_strength",
    "created_at",
    "version"
  ],
  "additionalProperties": false
}
```

### Field Definitions

| Field | Type | Required | Range | Unit | Description |
|-------|------|----------|-------|------|-------------|
| `id` | string | Yes | 1-64 chars | N/A | Unique trace identifier (deterministic hash) |
| `coord` | object | Yes | N/A | N/A | 5-D coordinate structure |
| `coord.space` | object | Yes | N/A | N/A | 3D spatial coordinates |
| `coord.space.x` | number | Yes | [-100, 100] | mm | X coordinate in MNI152 atlas |
| `coord.space.y` | number | Yes | [-100, 100] | mm | Y coordinate in MNI152 atlas |
| `coord.space.z` | number | Yes | [-100, 100] | mm | Z coordinate in MNI152 atlas |
| `coord.time` | object | Yes | N/A | N/A | Temporal coordinate |
| `coord.time.t_sec` | number | Yes | [0, 4102444800] | seconds | Unix epoch timestamp |
| `coord.state` | object | Yes | N/A | N/A | Internal state vector by layer |
| `coord.state.molecular` | array | No | 0-1024 | f32[] | Molecular layer features |
| `coord.state.synaptic` | array | No | 0-512 | f32[] | Synaptic layer features |
| `coord.state.microcircuit` | array | Yes | 2-256 | f32[] | Microcircuit layer features |
| `coord.state.network` | array | Yes | 1-128 | f32[] | Network layer features |
| `coord.state.systemic` | array | Yes | 4-64 | f32[] | Systemic context features |
| `state_hex` | object | Yes | N/A | N/A | Hex-encoded resource summary |
| `state_hex.hex` | string | Yes | variable | hex | Hex-encoded state digest |
| `state_hex.digit_count` | integer | Yes | [0, ∞) | count | Total digit count |
| `state_hex.nonzero_nibbles` | integer | Yes | [0, ∞) | count | Non-zero nibble count |
| `state_hex.l1_norm` | integer | Yes | [0, ∞) | count | L1 norm of scaled values |
| `roh` | number | Yes | [0.0, 0.30] | ratio | Risk-of-Harm score |
| `knowledge_factor` | number | Yes | [0.50, 1.0] | ratio | Knowledge factor (PoF) |
| `safety_strength` | number | Yes | [0.75, 1.0] | ratio | Safety strength (export) |
| `created_at` | number | Yes | [0, 4102444800] | seconds | Creation timestamp |
| `version` | string | Yes | semver | N/A | Schema version |

### Complete MemoryTrace Example

```json
{
  "id": "biomem-1700000000-a3f5e7c9d2b1",
  "coord": {
    "space": {
      "x": 12.5,
      "y": -8.3,
      "z": 15.7
    },
    "time": {
      "t_sec": 1700000000.0
    },
    "state": {
      "molecular": [],
      "synaptic": [],
      "microcircuit": [0.52, 0.38],
      "network": [0.67],
      "systemic": [0.625, 0.12, 0.82, 0.88]
    }
  },
  "state_hex": {
    "hex": "003400260043000c00520058",
    "digit_count": 52,
    "nonzero_nibbles": 18,
    "l1_norm": 301
  },
  "roh": 0.12,
  "knowledge_factor": 0.82,
  "safety_strength": 0.88,
  "created_at": 1700000000.0,
  "version": "1.0.0"
}
```

---

## EegNetworkSample Schema

### Root Object

```json
{
  "$schema": "https://data-lake.so/schemas/biomem/sample-1.0.0.json",
  "type": "object",
  "required": [
    "roi_label",
    "x",
    "y",
    "z",
    "t_sec",
    "bandpower_theta",
    "bandpower_alpha",
    "coherence_dm",
    "hrv",
    "roh",
    "knowledge_factor",
    "safety_strength"
  ],
  "additionalProperties": false
}
```

### Field Definitions

| Field | Type | Required | Range | Unit | Description |
|-------|------|----------|-------|------|-------------|
| `roi_label` | string | Yes | 1-128 chars | N/A | Atlas ROI label |
| `x` | number | Yes | [-100, 100] | mm | X coordinate (MNI152) |
| `y` | number | Yes | [-100, 100] | mm | Y coordinate (MNI152) |
| `z` | number | Yes | [-100, 100] | mm | Z coordinate (MNI152) |
| `t_sec` | number | Yes | [0, 4102444800] | seconds | Unix epoch timestamp |
| `bandpower_theta` | number | Yes | [0.0, 1.0] | normalized | Theta bandpower (4-8 Hz) |
| `bandpower_alpha` | number | Yes | [0.0, 1.0] | normalized | Alpha bandpower (8-13 Hz) |
| `coherence_dm` | number | Yes | [0.0, 1.0] | normalized | DMN coherence metric |
| `hrv` | number | Yes | [0.0, 100.0] | ms | Heart rate variability |
| `roh` | number | Yes | [0.0, 1.0] | ratio | Risk-of-Harm score |
| `knowledge_factor` | number | Yes | [0.0, 1.0] | ratio | Knowledge factor |
| `safety_strength` | number | Yes | [0.0, 1.0] | ratio | Safety strength |

### Complete EegNetworkSample Example

```json
{
  "roi_label": "hippocampus_CA3",
  "x": 12.5,
  "y": -8.3,
  "z": 15.7,
  "t_sec": 1700000000.0,
  "bandpower_theta": 0.52,
  "bandpower_alpha": 0.38,
  "coherence_dm": 0.67,
  "hrv": 62.5,
  "roh": 0.12,
  "knowledge_factor": 0.82,
  "safety_strength": 0.88
}
```

---

## Coordinate System

### Spatial Reference Frame

All spatial coordinates MUST use the **MNI152** standard atlas reference frame:

| Axis | Direction | Range | Origin |
|------|-----------|-------|--------|
| X | Left (-) to Right (+) | [-100, 100] mm | Mid-sagittal plane |
| Y | Posterior (-) to Anterior (+) | [-100, 100] mm | Anterior commissure |
| Z | Inferior (-) to Superior (+) | [-100, 100] mm | Anterior commissure |

### Temporal Reference Frame

All timestamps MUST use **Unix epoch** (seconds since 1970-01-01T00:00:00Z):

| Property | Value |
|----------|-------|
| Epoch | 1970-01-01T00:00:00Z (UTC) |
| Resolution | Milliseconds (f64) |
| Maximum | 4102444800 (Year 2100) |
| Timezone | UTC (no offset stored) |

---

## Biophysical Layers

### Layer Taxonomy

| Index | Name | Min Dim | Max Dim | Required Features |
|-------|------|---------|---------|-------------------|
| 0 | `molecular` | 0 | 1024 | None (reserved) |
| 1 | `synaptic` | 0 | 512 | None (reserved) |
| 2 | `microcircuit` | 2 | 256 | `bandpower_theta`, `bandpower_alpha` |
| 3 | `network` | 1 | 128 | `coherence_dm` |
| 4 | `systemic` | 4 | 64 | `hrv`, `roh`, `knowledge_factor`, `safety_strength` |

### State Vector Encoding

Each layer's state vector is encoded as follows:

```
For each value v in state vector:
  1. Scale: scaled = round(v * 100.0)
  2. Clamp: scaled = max(0, scaled)
  3. Hex: hex_string = format(scaled, "04x")
  4. Concatenate all hex strings
```

### L1 Norm Calculation

```
l1_norm = sum(scaled_values)
where scaled_values = [round(v * 100.0) for v in all_state_vectors]
```

---

## ALN Field Mapping

### MemoryTrace → ALN Shard Fields

| MemoryTrace Field | ALN Shard | ALN Field | Validation Rule |
|-------------------|-----------|-----------|-----------------|
| `roh` | `biomem-5d.aln` | `roh_ceiling` | `<= 0.30` |
| `safety_strength` | `biomem-5d.aln` | `safety_floor` | `>= 0.75` |
| `knowledge_factor` | `biomem-5d.aln` | `knowledge_floor` | `>= 0.50` |
| `coord.state` | `biomem-5d.aln` | `internal.code` | `non_invertible` |
| `version` | `biomem-5d.aln` | `version_match` | `semver_compatible` |
| `roh` | `biomem-risk.aln` | `roh_global` | `<= 0.30` |
| `safety_strength` | `biomem-risk.aln` | `safety_minimum` | `>= 0.75` |
| `coord.state.microcircuit` | `biomem.layer.microcircuit.aln` | `bandpower_theta` | `[0.0, 1.0]` |
| `coord.state.microcircuit` | `biomem.layer.microcircuit.aln` | `bandpower_alpha` | `[0.0, 1.0]` |
| `id` | `biomem.hypothesis.h1.aln` | `evidence_hash` | `sha256_truncate` |
| `knowledge_factor` | `biomem.hypothesis.h1.aln` | `knowledge_factor` | `>= 0.50` |
| `created_at` | `biomem-evolution.aln` | `timestamp` | `monotonic_increasing` |

### EegNetworkSample → ALN Shard Fields

| Sample Field | ALN Shard | ALN Field | Validation Rule |
|--------------|-----------|-----------|-----------------|
| `bandpower_theta` | `biomem.layer.microcircuit.aln` | `bandpower_theta` | `[0.01, 1.0]` |
| `bandpower_alpha` | `biomem.layer.microcircuit.aln` | `bandpower_alpha` | `[0.0, 1.0]` |
| `coherence_dm` | `biomem.layer.microcircuit.aln` | `coherence_dm` | `[0.0, 1.0]` |
| `hrv` | `biomem.layer.microcircuit.aln` | `hrv` | `[0.0, 100.0]` |
| `roh` | `biomem-risk.aln` | `roh_ceiling` | `<= 0.30` |
| `knowledge_factor` | `biomem-evolution.aln` | `knowledge_factor` | `>= 0.50` |
| `safety_strength` | `biomem-risk.aln` | `safety_minimum` | `>= 0.75` |
| `x, y, z` | `biomem-5d.aln` | `spatial.x, y, z` | `not_nan` |
| `t_sec` | `biomem-5d.aln` | `temporal.t` | `>= 0.0` |

---

## SKO Field Mapping

### MemoryTrace → SKO Wrapping

| MemoryTrace Field | SKO Field | Transformation |
|-------------------|-----------|----------------|
| `id` | `sko.payload.hash` | passthrough |
| `created_at` | `sko.timestamp` | passthrough |
| `version` | `sko.modality.version` | passthrough |
| `roh` | `sko.payload.roh` | passthrough |
| `knowledge_factor` | `sko.payload.knowledge_factor` | passthrough |
| `safety_strength` | `sko.payload.safety_strength` | passthrough |
| `coord.space` | `sko.payload.coordinates` | object_passthrough |
| `coord.state` | `sko.payload.state_vector` | serialized_base64 |
| `state_hex` | `sko.payload.state_digest` | passthrough |

### SKO Wrapper Template

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
      "hash": "biomem-1700000000-a3f5e7c9d2b1",
      "roh": 0.12,
      "knowledge_factor": 0.82,
      "safety_strength": 0.88,
      "coordinates": {
        "x": 12.5,
        "y": -8.3,
        "z": 15.7
      },
      "state_vector": "base64_encoded_state",
      "state_digest": {
        "hex": "003400260043000c00520058",
        "l1_norm": 301
      }
    },
    "timestamp": 1700000000.0,
    "encryption": "aes256_gcm",
    "signature": "ed25519_signature_hex",
    "metadata": {
      "alN_shards": ["biomem-5d.v1", "biomem-risk.v1"],
      "housing_protection": "protected",
      "neurorights_attestation": true
    }
  }
}
```

---

## Validation Rules

### Pre-Encoding Validation (EegNetworkSample)

| Rule | Field | Condition | Action on Fail |
|------|-------|-----------|----------------|
| `spatial_valid` | `x, y, z` | `not_nan AND in_range[-100, 100]` | Reject |
| `temporal_valid` | `t_sec` | `>= 0.0` | Reject |
| `theta_valid` | `bandpower_theta` | `>= 0.01` | Warning |
| `alpha_valid` | `bandpower_alpha` | `in_range[0.0, 1.0]` | Warning |
| `coherence_valid` | `coherence_dm` | `in_range[0.0, 1.0]` | Reject |
| `hrv_valid` | `hrv` | `in_range[0.0, 100.0]` | Warning |
| `roh_valid` | `roh` | `in_range[0.0, 0.30]` | Reject |
| `knowledge_valid` | `knowledge_factor` | `in_range[0.50, 1.0]` | Reject |
| `safety_valid` | `safety_strength` | `in_range[0.75, 1.0]` | Reject |

### Post-Encoding Validation (MemoryTrace)

| Rule | Field | Condition | Action on Fail |
|------|-------|-----------|----------------|
| `roh_ceiling` | `roh` | `<= 0.30` | Reject |
| `safety_floor` | `safety_strength` | `>= 0.75` | Reject |
| `knowledge_floor` | `knowledge_factor` | `>= 0.50` | Reject |
| `nullspace_floor` | `nullspace_dim()` | `>= 128` | Flag |
| `version_match` | `version` | `semver_compatible` | Flag |
| `id_format` | `id` | `matches ^biomem-[0-9]+-[a-f0-9]+$` | Reject |
| `hex_consistency` | `state_hex.hex` | `length == digit_count * 4` | Reject |
| `l1_consistency` | `state_hex.l1_norm` | `== sum(scaled_values)` | Reject |

### ALN Shard Compliance

| Shard | Compliance Check | Enforcement |
|-------|------------------|-------------|
| `biomem-5d.aln` | All invariants satisfied | Hard |
| `biomem-evolution.aln` | Monotonicity preserved | Hard |
| `biomem-risk.aln` | RoH and safety within bounds | Hard |
| `biomem.layer.microcircuit.aln` | Layer dimensions valid | Soft |
| `biomem.hypothesis.h1.aln` | Evidence requirements met | Soft |

---

## Examples

### Minimal Valid MemoryTrace

```json
{
  "id": "biomem-1700000000-0000000000000000",
  "coord": {
    "space": {"x": 0.0, "y": 0.0, "z": 0.0},
    "time": {"t_sec": 1700000000.0},
    "state": {
      "molecular": [],
      "synaptic": [],
      "microcircuit": [0.5, 0.5],
      "network": [0.5],
      "systemic": [0.5, 0.1, 0.8, 0.8]
    }
  },
  "state_hex": {
    "hex": "003200320032000a00500050",
    "digit_count": 48,
    "nonzero_nibbles": 16,
    "l1_norm": 280
  },
  "roh": 0.10,
  "knowledge_factor": 0.80,
  "safety_strength": 0.80,
  "created_at": 1700000000.0,
  "version": "1.0.0"
}
```

### Hippocampal Memory Encoding Trace

```json
{
  "id": "biomem-1700000000-a3f5e7c9d2b1",
  "coord": {
    "space": {"x": 12.5, "y": -8.3, "z": 15.7},
    "time": {"t_sec": 1700000000.0},
    "state": {
      "molecular": [],
      "synaptic": [],
      "microcircuit": [0.52, 0.38],
      "network": [0.67],
      "systemic": [0.625, 0.12, 0.82, 0.88]
    }
  },
  "state_hex": {
    "hex": "003400260043000c00520058",
    "digit_count": 52,
    "nonzero_nibbles": 18,
    "l1_norm": 301
  },
  "roh": 0.12,
  "knowledge_factor": 0.82,
  "safety_strength": 0.88,
  "created_at": 1700000000.0,
  "version": "1.0.0"
}
```

### Invalid Trace (RoH Exceeded)

```json
{
  "id": "biomem-1700000000-invalid",
  "coord": {
    "space": {"x": 0.0, "y": 0.0, "z": 0.0},
    "time": {"t_sec": 1700000000.0},
    "state": {
      "molecular": [],
      "synaptic": [],
      "microcircuit": [0.5, 0.5],
      "network": [0.5],
      "systemic": [0.5, 0.50, 0.8, 0.8]
    }
  },
  "state_hex": {
    "hex": "003200320032003200500050",
    "digit_count": 52,
    "nonzero_nibbles": 18,
    "l1_norm": 320
  },
  "roh": 0.50,
  "knowledge_factor": 0.80,
  "safety_strength": 0.80,
  "created_at": 1700000000.0,
  "version": "1.0.0"
}
```

**Validation Error:** `roh` (0.50) exceeds ceiling (0.30) per `biomem-5d.aln` and `biomem-risk.aln`.

---

## Version History

| Version | Date | Changes | Compatibility |
|---------|------|---------|---------------|
| 1.0.0 | 2024-01-15 | Initial release | biomem-core>=1.0.0 |
| 1.0.1 | planned_Q2_2024 | Add optional beta/gamma bandpower | Backward compatible |
| 1.1.0 | planned_Q3_2024 | Add streaming trace support | Breaking change |
| 2.0.0 | planned_Q4_2024 | Add molecular/synaptic proxies | Breaking change |

---

## References

- **ALN Shard Specifications:** `/spec/` directory
- **Rust Implementation:** `/rust/crates/biomem-core/`
- **Encoder Implementation:** `/rust/crates/biomem-encoder/`
- **SKO Wrapping:** `/docs/INTEGRATIONS.md`
- **Prometheus Metrics:** `/examples/prometheus-exporter/`

---

## Contact

- **Repository:** https://github.com/data-lake/mem-brain
- **Documentation:** https://docs.data-lake.so/mem-brain
- **Issues:** https://github.com/data-lake/mem-brain/issues
- **License:** MIT (Data_Lake Sovereign License)

---

*This document is machine-readable and validated against the canonical JSON Schema at `https://data-lake.so/schemas/biomem/format-1.0.0.json`*
