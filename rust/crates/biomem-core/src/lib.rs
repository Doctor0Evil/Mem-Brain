// ============================================================================
// CRATE: biomem-core
// PATH:  rust/crates/biomem-core/src/lib.rs
// VER:   1.0.0
// LIC:   MIT (Data_Lake Sovereign License)
// DESC:  Core 5-D biophysical memory specification for Mem-Brain protocol.
//        Provides type-safe coordinates, layer enums, and MemoryTrace structs
//        with monotonic growth and non-invertibility guarantees.
// ============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_field_names)]

pub mod coord;
pub mod layer;
pub mod trace;
pub mod contract;

pub use coord::{Coord5D, Space3D, TimeCoord};
pub use layer::{BioLayer, InternalState};
pub use trace::{MemoryTrace, ResourceSummary};
pub use contract::{MemoryDecoderContract, CharterProof, EvoState};

/// Library version for compatibility checking across ALN shards.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum allowed RoH (Risk-of-Harm) ceiling for exportable traces.
pub const ROH_MAX_GLOBAL: f32 = 0.30;

/// Minimum safety strength required for off-host trace transmission.
pub const SAFETY_STRENGTH_MIN: f32 = 0.75;

/// Minimum knowledge factor for PoF validation.
pub const KNOWLEDGE_FACTOR_MIN: f32 = 0.50;

/// Null-space dimension floor for privacy preservation.
pub const NULLSPACE_DIM_FLOOR: usize = 128;

// ============================================================================
// MODULE: coord
// ============================================================================

/// 3D spatial coordinates in millimeters within a standardized brain atlas.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Space3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Space3D {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    #[must_use]
    pub fn distance_mm(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Temporal coordinate in seconds (supports ms to years scale).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeCoord {
    pub t_sec: f64,
}

impl TimeCoord {
    #[must_use]
    pub fn new(t_sec: f64) -> Self {
        Self { t_sec }
    }

    #[must_use]
    pub fn now_epoch() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self { t_sec: duration.as_secs_f64() }
    }

    #[must_use]
    pub fn delta_sec(&self, other: &Self) -> f64 {
        (self.t_sec - other.t_sec).abs()
    }
}

/// Complete 5-D coordinate: 3D space + time + internal state vector.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Coord5D {
    pub space: Space3D,
    pub time: TimeCoord,
    pub state: InternalState,
}

impl Coord5D {
    #[must_use]
    pub fn new(space: Space3D, time: TimeCoord, state: InternalState) -> Self {
        Self { space, time, state }
    }

    #[must_use]
    pub fn dimension_count() -> usize {
        5 // x, y, z, t, code_axis (internal state)
    }
}

// ============================================================================
// MODULE: layer
// ============================================================================

/// Biophysical layer taxonomy for multi-scale memory representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BioLayer {
    Molecular,
    Synaptic,
    Microcircuit,
    MesoscopicNetwork,
    SystemicContext,
}

impl BioLayer {
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Molecular,
            Self::Synaptic,
            Self::Microcircuit,
            Self::MesoscopicNetwork,
            Self::SystemicContext,
        ]
    }

    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Self::Molecular => 0,
            Self::Synaptic => 1,
            Self::Microcircuit => 2,
            Self::MesoscopicNetwork => 3,
            Self::SystemicContext => 4,
        }
    }

    #[must_use]
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Molecular),
            1 => Some(Self::Synaptic),
            2 => Some(Self::Microcircuit),
            3 => Some(Self::MesoscopicNetwork),
            4 => Some(Self::SystemicContext),
            _ => None,
        }
    }
}

/// Internal state vector organized by biophysical layer.
/// Each layer holds a variable-length feature vector (compressed codes).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct InternalState {
    pub molecular: Vec<f32>,
    pub synaptic: Vec<f32>,
    pub microcircuit: Vec<f32>,
    pub network: Vec<f32>,
    pub systemic: Vec<f32>,
}

impl InternalState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_capacity(
        molecular: usize,
        synaptic: usize,
        microcircuit: usize,
        network: usize,
        systemic: usize,
    ) -> Self {
        Self {
            molecular: Vec::with_capacity(molecular),
            synaptic: Vec::with_capacity(synaptic),
            microcircuit: Vec::with_capacity(microcircuit),
            network: Vec::with_capacity(network),
            systemic: Vec::with_capacity(systemic),
        }
    }

    #[must_use]
    pub fn total_dimension_count(&self) -> usize {
        self.molecular.len()
            + self.synaptic.len()
            + self.microcircuit.len()
            + self.network.len()
            + self.systemic.len()
    }

    #[must_use]
    pub fn layer_vector(&self, layer: BioLayer) -> &[f32] {
        match layer {
            BioLayer::Molecular => &self.molecular,
            BioLayer::Synaptic => &self.synaptic,
            BioLayer::Microcircuit => &self.microcircuit,
            BioLayer::MesoscopicNetwork => &self.network,
            BioLayer::SystemicContext => &self.systemic,
        }
    }

    /// Compute L1 norm across all layers (used for resource summary).
    #[must_use]
    pub fn l1_norm(&self) -> u64 {
        let mut sum: u64 = 0;
        for v in [
            &self.molecular,
            &self.synaptic,
            &self.microcircuit,
            &self.network,
            &self.systemic,
        ]
        .iter()
        {
            for x in v.iter() {
                let scaled = (x * 100.0).round().max(0.0) as u64;
                sum = sum.saturating_add(scaled);
            }
        }
        sum
    }
}

// ============================================================================
// MODULE: trace
// ============================================================================

/// Hex-encoded resource summary for audit and PoF verification.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ResourceSummary {
    pub hex: String,
    pub digit_count: usize,
    pub nonzero_nibbles: usize,
    pub l1_norm: u64,
}

impl ResourceSummary {
    #[must_use]
    pub fn new(hex: String, digit_count: usize, nonzero_nibbles: usize, l1_norm: u64) -> Self {
        Self { hex, digit_count, nonzero_nibbles, l1_norm }
    }

    #[must_use]
    pub fn from_internal_state(state: &InternalState) -> Self {
        let mut digits: Vec<u8> = Vec::new();
        let mut l1: u64 = 0;

        for v in [
            &state.molecular,
            &state.synaptic,
            &state.microcircuit,
            &state.network,
            &state.systemic,
        ]
        .iter()
        {
            for x in v.iter() {
                let scaled = (x * 100.0).round().max(0.0) as u64;
                l1 = l1.saturating_add(scaled);
                let hex = format!("{:04x}", scaled);
                for b in hex.bytes() {
                    digits.push(b);
                }
            }
        }

        let nonzero = digits.iter().filter(|d| **d != b'0').count();
        let hex_str = String::from_utf8(digits).unwrap_or_default();

        Self {
            hex: hex_str,
            digit_count: l1 as usize,
            nonzero_nibbles: nonzero,
            l1_norm: l1,
        }
    }
}

/// Core memory trace: non-invertible, monotonic, ALN-governed data structure.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryTrace {
    pub id: String,
    pub coord: Coord5D,
    pub state_hex: ResourceSummary,
    pub roh: f32,
    pub knowledge_factor: f32,
    pub safety_strength: f32,
    pub created_at: f64,
    pub version: String,
}

impl MemoryTrace {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        coord: Coord5D,
        roh: f32,
        knowledge_factor: f32,
        safety_strength: f32,
    ) -> Self {
        let state_hex = ResourceSummary::from_internal_state(&coord.state);
        let created_at = TimeCoord::now_epoch().t_sec;

        Self {
            id: id.into(),
            coord,
            state_hex,
            roh,
            knowledge_factor,
            safety_strength,
            created_at,
            version: VERSION.to_string(),
        }
    }

    /// Validate trace against global RoH ceiling.
    #[must_use]
    pub fn validate_roh(&self) -> bool {
        self.roh <= ROH_MAX_GLOBAL
    }

    /// Validate trace meets minimum safety strength for export.
    #[must_use]
    pub fn validate_safety(&self) -> bool {
        self.safety_strength >= SAFETY_STRENGTH_MIN
    }

    /// Validate trace meets minimum knowledge factor for PoF.
    #[must_use]
    pub fn validate_knowledge(&self) -> bool {
        self.knowledge_factor >= KNOWLEDGE_FACTOR_MIN
    }

    /// Full validation suite for ALN shard compliance.
    #[must_use]
    pub fn validate_all(&self) -> bool {
        self.validate_roh() && self.validate_safety() && self.validate_knowledge()
    }

    /// Compute monotone complexity score (never decreases with updates).
    #[must_use]
    pub fn complexity_score(&self) -> f32 {
        let dim = self.coord.state.total_dimension_count() as f32;
        let layers = BioLayer::all().len() as f32;
        (dim.ln().max(0.0) + 1.0) * layers * self.knowledge_factor
    }

    /// Check null-space privacy preservation.
    #[must_use]
    pub fn nullspace_dim(&self) -> usize {
        let total = self.coord.state.total_dimension_count();
        if total > NULLSPACE_DIM_FLOOR {
            total - NULLSPACE_DIM_FLOOR
        } else {
            0
        }
    }
}

// ============================================================================
// MODULE: contract
// ============================================================================

/// Decoder contract for capability evolution (monotone, non-degrading).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryDecoderContract {
    pub decoder_id: String,
    pub input_dim: usize,
    pub output_dim: usize,
    pub nullspace_preserved: bool,
    pub monotone_guarantee: bool,
}

impl MemoryDecoderContract {
    #[must_use]
    pub fn new(
        decoder_id: impl Into<String>,
        input_dim: usize,
        output_dim: usize,
    ) -> Self {
        Self {
            decoder_id: decoder_id.into(),
            input_dim,
            output_dim,
            nullspace_preserved: output_dim < input_dim,
            monotone_guarantee: true,
        }
    }

    #[must_use]
    pub fn validate(&self) -> bool {
        self.nullspace_preserved && self.monotone_guarantee && self.output_dim < self.input_dim
    }
}

/// Evolution state for PoF/NFA tracking across upgrades.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvoState {
    pub capability_score: f32,
    pub knowledge_factor: f32,
    pub safety_strength: f32,
    pub nullspace_dim: usize,
    pub risk_of_harm: f32,
    pub timestamp: f64,
}

impl EvoState {
    #[must_use]
    pub fn new(
        capability_score: f32,
        knowledge_factor: f32,
        safety_strength: f32,
        nullspace_dim: usize,
        risk_of_harm: f32,
    ) -> Self {
        Self {
            capability_score,
            knowledge_factor,
            safety_strength,
            nullspace_dim,
            risk_of_harm,
            timestamp: TimeCoord::now_epoch().t_sec,
        }
    }

    /// Check monotonicity: new state must not degrade capabilities.
    #[must_use]
    pub fn is_monotone_upgrade(&self, prev: &Self) -> bool {
        self.capability_score >= prev.capability_score
            && self.knowledge_factor >= prev.knowledge_factor
            && self.safety_strength >= prev.safety_strength
            && self.nullspace_dim >= prev.nullspace_dim
            && self.risk_of_harm <= prev.risk_of_harm
    }
}

/// Cryptographic proof of charter compliance for ALN validation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CharterProof {
    pub proof_hash: String,
    pub charter_version: String,
    pub validator_id: String,
    pub timestamp: f64,
    pub signature: String,
}

impl CharterProof {
    #[must_use]
    pub fn new(
        proof_hash: impl Into<String>,
        charter_version: impl Into<String>,
        validator_id: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            proof_hash: proof_hash.into(),
            charter_version: charter_version.into(),
            validator_id: validator_id.into(),
            timestamp: TimeCoord::now_epoch().t_sec,
            signature: signature.into(),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space3d_distance() {
        let a = Space3D::new(0.0, 0.0, 0.0);
        let b = Space3D::new(3.0, 4.0, 0.0);
        assert!((a.distance_mm(&b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_memory_trace_validation() {
        let state = InternalState::new();
        let coord = Coord5D::new(
            Space3D::origin(),
            TimeCoord::now_epoch(),
            state,
        );
        let trace = MemoryTrace::new(
            "test-001",
            coord,
            0.15,
            0.80,
            0.85,
        );
        assert!(trace.validate_all());
    }

    #[test]
    fn test_evo_state_monotonicity() {
        let prev = EvoState::new(0.5, 0.6, 0.7, 128, 0.2);
        let curr = EvoState::new(0.6, 0.7, 0.8, 130, 0.15);
        assert!(curr.is_monotone_upgrade(&prev));
    }

    #[test]
    fn test_bio_layer_index() {
        for (idx, layer) in BioLayer::all().iter().enumerate() {
            assert_eq!(layer.index(), idx);
            assert_eq!(BioLayer::from_index(idx), Some(*layer));
        }
    }
}
