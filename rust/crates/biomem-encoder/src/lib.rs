// ============================================================================
// CRATE: biomem-encoder
// PATH:  rust/crates/biomem-encoder/src/lib.rs
// VER:   1.0.0
// LIC:   MIT (Data_Lake Sovereign License)
// DESC:  Biomem encoder for converting EEG/network features into MemoryTrace
//        objects. Implements PoF (Proof-of-Functionality) and NFA (Non-Fair-
//        Allowances) validation with ALN shard compatibility.
// DEP:   biomem-core >= 1.0.0
// ============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_field_names)]

pub mod encoder;
pub mod sample;
pub mod validator;
pub mod metrics;

pub use encoder::{BiomemEncoder, EegNetworkEncoder, EncoderConfig};
pub use sample::EegNetworkSample;
pub use validator::{TraceValidator, ValidationReport};
pub use metrics::EncoderMetrics;

/// Library version for ALN shard compatibility checking.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum theta bandpower threshold for valid microcircuit encoding.
pub const THETA_MIN: f32 = 0.01;

/// Maximum alpha bandpower threshold for valid microcircuit encoding.
pub const ALPHA_MAX: f32 = 1.0;

/// Minimum coherence value for network layer validity.
pub const COHERENCE_MIN: f32 = 0.0;

/// Maximum coherence value (DMN default mode network).
pub const COHERENCE_MAX: f32 = 1.0;

/// HRV (Heart Rate Variability) acceptable range.
pub const HRV_MIN: f32 = 0.0;
pub const HRV_MAX: f32 = 100.0;

/// Default ROI label for unassigned samples.
pub const DEFAULT_ROI_LABEL: &str = "unassigned";

// ============================================================================
// MODULE: sample
// ============================================================================

/// Input sample from BCI/CyberNano pipeline containing EEG and physiological features.
/// All fields are validated before encoding to ensure MemoryTrace integrity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EegNetworkSample {
    /// Atlas ROI (Region of Interest) label for spatial anchoring.
    pub roi_label: String,
    
    /// 3D spatial coordinates in millimeters (atlas frame).
    pub x: f32,
    pub y: f32,
    pub z: f32,
    
    /// Timestamp in seconds since epoch.
    pub t_sec: f64,
    
    /// Theta bandpower (4-8 Hz) - microcircuit layer feature.
    pub bandpower_theta: f32,
    
    /// Alpha bandpower (8-13 Hz) - microcircuit layer feature.
    pub bandpower_alpha: f32,
    
    /// Default Mode Network coherence metric - network layer feature.
    pub coherence_dm: f32,
    
    /// Heart Rate Variability - systemic context feature.
    pub hrv: f32,
    
    /// Risk-of-Harm score (0.0-1.0) - must be <= 0.30 for export.
    pub roh: f32,
    
    /// Knowledge factor (0.0-1.0) - must be >= 0.50 for PoF.
    pub knowledge_factor: f32,
    
    /// Safety strength (0.0-1.0) - must be >= 0.75 for off-host transmission.
    pub safety_strength: f32,
}

impl EegNetworkSample {
    /// Construct a new sample with validation-ready defaults.
    #[must_use]
    pub fn new(
        roi_label: impl Into<String>,
        x: f32,
        y: f32,
        z: f32,
        t_sec: f64,
        bandpower_theta: f32,
        bandpower_alpha: f32,
        coherence_dm: f32,
        hrv: f32,
        roh: f32,
        knowledge_factor: f32,
        safety_strength: f32,
    ) -> Self {
        Self {
            roi_label: roi_label.into(),
            x,
            y,
            z,
            t_sec,
            bandpower_theta,
            bandpower_alpha,
            coherence_dm,
            hrv,
            roh,
            knowledge_factor,
            safety_strength,
        }
    }

    /// Validate all fields against acceptable ranges before encoding.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // Spatial coordinate validation
        if self.x.is_nan() || self.y.is_nan() || self.z.is_nan() {
            errors.push("Spatial coordinates contain NaN values".to_string());
        }

        // Timestamp validation
        if self.t_sec < 0.0 {
            errors.push("Timestamp cannot be negative".to_string());
        }

        // Bandpower validation (microcircuit layer)
        if self.bandpower_theta < THETA_MIN {
            warnings.push(format!(
                "Theta bandpower {:.4} below minimum {:.4}",
                self.bandpower_theta, THETA_MIN
            ));
        }
        if self.bandpower_alpha < THETA_MIN || self.bandpower_alpha > ALPHA_MAX {
            warnings.push(format!(
                "Alpha bandpower {:.4} outside range [{:.4}, {:.4}]",
                self.bandpower_alpha, THETA_MIN, ALPHA_MAX
            ));
        }

        // Coherence validation (network layer)
        if self.coherence_dm < COHERENCE_MIN || self.coherence_dm > COHERENCE_MAX {
            errors.push(format!(
                "Coherence DM {:.4} outside valid range [0.0, 1.0]",
                self.coherence_dm
            ));
        }

        // HRV validation (systemic layer)
        if self.hrv < HRV_MIN || self.hrv > HRV_MAX {
            warnings.push(format!(
                "HRV {:.4} outside typical range [{:.4}, {:.4}]",
                self.hrv, HRV_MIN, HRV_MAX
            ));
        }

        // RoH validation (critical for neurorights)
        if self.roh < 0.0 || self.roh > 1.0 {
            errors.push(format!("RoH {:.4} must be in range [0.0, 1.0]", self.roh));
        } else if self.roh > biomem_core::ROH_MAX_GLOBAL {
            errors.push(format!(
                "RoH {:.4} exceeds global ceiling {:.4}",
                self.roh, biomem_core::ROH_MAX_GLOBAL
            ));
        }

        // Knowledge factor validation (PoF requirement)
        if self.knowledge_factor < 0.0 || self.knowledge_factor > 1.0 {
            errors.push(format!(
                "Knowledge factor {:.4} must be in range [0.0, 1.0]",
                self.knowledge_factor
            ));
        } else if self.knowledge_factor < biomem_core::KNOWLEDGE_FACTOR_MIN {
            errors.push(format!(
                "Knowledge factor {:.4} below PoF minimum {:.4}",
                self.knowledge_factor, biomem_core::KNOWLEDGE_FACTOR_MIN
            ));
        }

        // Safety strength validation (export requirement)
        if self.safety_strength < 0.0 || self.safety_strength > 1.0 {
            errors.push(format!(
                "Safety strength {:.4} must be in range [0.0, 1.0]",
                self.safety_strength
            ));
        } else if self.safety_strength < biomem_core::SAFETY_STRENGTH_MIN {
            errors.push(format!(
                "Safety strength {:.4} below export minimum {:.4}",
                self.safety_strength, biomem_core::SAFETY_STRENGTH_MIN
            ));
        }

        // ROI label validation
        if self.roi_label.is_empty() {
            warnings.push("ROI label is empty, will use default".to_string());
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            sample_hash: self.compute_hash(),
        }
    }

    /// Compute a deterministic hash for sample tracking and PoF evidence.
    #[must_use]
    pub fn compute_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.roi_label.hash(&mut hasher);
        self.x.to_bits().hash(&mut hasher);
        self.y.to_bits().hash(&mut hasher);
        self.z.to_bits().hash(&mut hasher);
        self.t_sec.to_bits().hash(&mut hasher);
        hasher.finish().to_string()
    }

    /// Normalize bandpower values to [0, 1] range for internal state encoding.
    #[must_use]
    pub fn normalized_bandpower(&self) -> (f32, f32) {
        let theta = self.bandpower_theta.clamp(0.0, 1.0);
        let alpha = self.bandpower_alpha.clamp(0.0, 1.0);
        (theta, alpha)
    }

    /// Normalize coherence to [0, 1] range.
    #[must_use]
    pub fn normalized_coherence(&self) -> f32 {
        self.coherence_dm.clamp(0.0, 1.0)
    }

    /// Normalize HRV to [0, 1] range (assuming max 100).
    #[must_use]
    pub fn normalized_hrv(&self) -> f32 {
        (self.hrv / HRV_MAX).clamp(0.0, 1.0)
    }
}

impl Default for EegNetworkSample {
    fn default() -> Self {
        Self {
            roi_label: DEFAULT_ROI_LABEL.to_string(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            t_sec: 0.0,
            bandpower_theta: 0.5,
            bandpower_alpha: 0.5,
            coherence_dm: 0.5,
            hrv: 50.0,
            roh: 0.1,
            knowledge_factor: 0.8,
            safety_strength: 0.9,
        }
    }
}

// ============================================================================
// MODULE: encoder
// ============================================================================

/// Configuration for encoder behavior and validation strictness.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EncoderConfig {
    /// Enable strict validation (reject on warnings).
    pub strict_mode: bool,
    
    /// Enable automatic ROI label correction.
    pub auto_correct_roi: bool,
    
    /// Enable metrics collection for Prometheus export.
    pub collect_metrics: bool,
    
    /// Maximum trace ID length.
    pub max_id_length: usize,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            auto_correct_roi: true,
            collect_metrics: true,
            max_id_length: 64,
        }
    }
}

impl EncoderConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_metrics(mut self, collect: bool) -> Self {
        self.collect_metrics = collect;
        self
    }
}

/// Core encoder trait for converting input samples to MemoryTrace objects.
/// Implementations must guarantee non-invertibility and monotonic growth.
pub trait BiomemEncoder {
    type Input: Clone + std::fmt::Debug;
    type Output;
    type Error: std::fmt::Display;

    /// Encode input sample into a MemoryTrace.
    fn encode(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;

    /// Validate input before encoding.
    fn validate(&self, input: &Self::Input) -> ValidationReport;

    /// Get encoder configuration.
    fn config(&self) -> &EncoderConfig;
}

/// Encoder error types for graceful failure handling.
#[derive(Clone, Debug, thiserror::Error)]
pub enum EncoderError {
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Encoding failed: {0}")]
    EncodingError(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceError(String),
    
    #[error("ALN shard violation: {0}")]
    AlnViolation(String),
}

/// Primary EEG/Network encoder implementation for Mem-Brain protocol.
/// Converts BCI telemetry into 5-D MemoryTrace with full ALN compliance.
#[derive(Clone, Debug)]
pub struct EegNetworkEncoder {
    config: EncoderConfig,
    metrics: EncoderMetrics,
    encode_count: u64,
}

impl EegNetworkEncoder {
    /// Create a new encoder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EncoderConfig::default(),
            metrics: EncoderMetrics::default(),
            encode_count: 0,
        }
    }

    /// Create a new encoder with custom configuration.
    #[must_use]
    pub fn with_config(config: EncoderConfig) -> Self {
        Self {
            config,
            metrics: EncoderMetrics::default(),
            encode_count: 0,
        }
    }

    /// Build internal state vector from sample features.
    fn build_internal_state(&self, sample: &EegNetworkSample) -> biomem_core::InternalState {
        let (theta, alpha) = sample.normalized_bandpower();
        let coherence = sample.normalized_coherence();
        let hrv = sample.normalized_hrv();

        let mut state = biomem_core::InternalState::with_capacity(0, 0, 2, 1, 4);

        // Molecular layer: empty (reserved for future nanomolecular proxies)
        // synaptic layer: empty (reserved for future synaptic proxies)

        // Microcircuit layer: bandpower features
        state.microcircuit.push(theta);
        state.microcircuit.push(alpha);

        // Network layer: DMN coherence
        state.network.push(coherence);

        // Systemic context: HRV, RoH, knowledge factor, safety strength
        state.systemic.push(hrv);
        state.systemic.push(sample.roh);
        state.systemic.push(sample.knowledge_factor);
        state.systemic.push(sample.safety_strength);

        state
    }

    /// Generate unique trace ID from sample metadata.
    fn generate_trace_id(&self, sample: &EegNetworkSample) -> String {
        let timestamp = sample.t_sec as u64;
        let roi_hash = sample.compute_hash();
        format!(
            "biomem-{}-{:016x}",
            timestamp,
            u64::from_str_radix(&roi_hash[..16].to_string(), 10).unwrap_or(0)
        )
        .chars()
        .take(self.config.max_id_length)
        .collect()
    }

    /// Record encoding metrics for Prometheus export.
    fn record_metrics(&mut self, sample: &EegNetworkSample, trace: &biomem_core::MemoryTrace) {
        if self.config.collect_metrics {
            self.metrics.encode_count += 1;
            self.metrics.last_roh = sample.roh;
            self.metrics.last_knowledge_factor = sample.knowledge_factor;
            self.metrics.last_safety_strength = sample.safety_strength;
            self.metrics.last_complexity = trace.complexity_score();
            self.metrics.last_nullspace_dim = trace.nullspace_dim();
        }
    }
}

impl Default for EegNetworkEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomemEncoder for EegNetworkEncoder {
    type Input = EegNetworkSample;
    type Output = biomem_core::MemoryTrace;
    type Error = EncoderError;

    fn encode(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
        // Validate input first
        let report = self.validate(input);
        if !report.valid {
            return Err(EncoderError::ValidationError(report.errors.join("; ")));
        }

        if self.config.strict_mode && !report.warnings.is_empty() {
            return Err(EncoderError::ValidationError(
                format!("Strict mode: warnings present: {}", report.warnings.join("; "))
            ));
        }

        // Build 5-D coordinate
        let space = biomem_core::Space3D::new(input.x, input.y, input.z);
        let time = biomem_core::TimeCoord::new(input.t_sec);
        let state = self.build_internal_state(input);
        let coord = biomem_core::Coord5D::new(space, time, state);

        // Create MemoryTrace with validation guarantees
        let trace = biomem_core::MemoryTrace::new(
            self.generate_trace_id(input),
            coord,
            input.roh,
            input.knowledge_factor,
            input.safety_strength,
        );

        // Final validation against ALN constraints
        if !trace.validate_all() {
            return Err(EncoderError::AlnViolation(
                "Trace failed ALN shard validation".to_string()
            ));
        }

        Ok(trace)
    }

    fn validate(&self, input: &Self::Input) -> ValidationReport {
        input.validate()
    }

    fn config(&self) -> &EncoderConfig {
        &self.config
    }
}

// ============================================================================
// MODULE: validator
// ============================================================================

/// Validation report for samples and traces.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub sample_hash: String,
}

impl ValidationReport {
    #[must_use]
    pub fn new(valid: bool, errors: Vec<String>, warnings: Vec<String>, sample_hash: String) -> Self {
        Self { valid, errors, warnings, sample_hash }
    }

    #[must_use]
    pub fn success(sample_hash: String) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            sample_hash,
        }
    }

    #[must_use]
    pub fn failure(errors: Vec<String>, sample_hash: String) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            sample_hash,
        }
    }
}

/// Trace validator for post-encoding ALN compliance checks.
#[derive(Clone, Debug)]
pub struct TraceValidator {
    strict_mode: bool,
}

impl TraceValidator {
    #[must_use]
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    #[must_use]
    pub fn validate(&self, trace: &biomem_core::MemoryTrace) -> ValidationReport {
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // RoH validation
        if trace.roh > biomem_core::ROH_MAX_GLOBAL {
            errors.push(format!(
                "RoH {:.4} exceeds global ceiling {:.4}",
                trace.roh, biomem_core::ROH_MAX_GLOBAL
            ));
        }

        // Safety strength validation
        if trace.safety_strength < biomem_core::SAFETY_STRENGTH_MIN {
            errors.push(format!(
                "Safety strength {:.4} below minimum {:.4}",
                trace.safety_strength, biomem_core::SAFETY_STRENGTH_MIN
            ));
        }

        // Knowledge factor validation
        if trace.knowledge_factor < biomem_core::KNOWLEDGE_FACTOR_MIN {
            errors.push(format!(
                "Knowledge factor {:.4} below PoF minimum {:.4}",
                trace.knowledge_factor, biomem_core::KNOWLEDGE_FACTOR_MIN
            ));
        }

        // Null-space privacy validation
        if trace.nullspace_dim() < biomem_core::NULLSPACE_DIM_FLOOR {
            warnings.push(format!(
                "Null-space dimension {} below recommended floor {}",
                trace.nullspace_dim(),
                biomem_core::NULLSPACE_DIM_FLOOR
            ));
        }

        // Version compatibility check
        if trace.version != biomem_core::VERSION {
            warnings.push(format!(
                "Version mismatch: trace {} vs core {}",
                trace.version, biomem_core::VERSION
            ));
        }

        ValidationReport {
            valid: errors.is_empty(),
            errors,
            warnings,
            sample_hash: trace.id.clone(),
        }
    }
}

// ============================================================================
// MODULE: metrics
// ============================================================================

/// Encoder metrics for Prometheus export and monitoring.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct EncoderMetrics {
    pub encode_count: u64,
    pub last_roh: f32,
    pub last_knowledge_factor: f32,
    pub last_safety_strength: f32,
    pub last_complexity: f32,
    pub last_nullspace_dim: usize,
}

impl EncoderMetrics {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Export metrics as Prometheus-format strings.
    #[must_use]
    pub fn to_prometheus(&self) -> Vec<String> {
        vec![
            format!("biomem_encode_count_total {}", self.encode_count),
            format!("biomem_roh_current {}", self.last_roh),
            format!("biomem_knowledge_factor_current {}", self.last_knowledge_factor),
            format!("biomem_safety_strength_current {}", self.last_safety_strength),
            format!("biomem_complexity_current {}", self.last_complexity),
            format!("biomem_nullspace_dim_current {}", self.last_nullspace_dim),
        ]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_sample() -> EegNetworkSample {
        EegNetworkSample::new(
            "hippocampus",
            10.0,
            20.0,
            30.0,
            1700000000.0,
            0.5,
            0.4,
            0.6,
            60.0,
            0.15,
            0.80,
            0.85,
        )
    }

    #[test]
    fn test_sample_validation_success() {
        let sample = create_valid_sample();
        let report = sample.validate();
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_sample_validation_roh_failure() {
        let mut sample = create_valid_sample();
        sample.roh = 0.50; // Exceeds 0.30 ceiling
        let report = sample.validate();
        assert!(!report.valid);
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_encoder_success() {
        let encoder = EegNetworkEncoder::new();
        let sample = create_valid_sample();
        let result = encoder.encode(&sample);
        assert!(result.is_ok());
        let trace = result.unwrap();
        assert!(trace.validate_all());
    }

    #[test]
    fn test_encoder_validation_failure() {
        let encoder = EegNetworkEncoder::new();
        let mut sample = create_valid_sample();
        sample.knowledge_factor = 0.30; // Below 0.50 minimum
        let result = encoder.encode(&sample);
        assert!(result.is_err());
    }

    #[test]
    fn test_trace_validator() {
        let encoder = EegNetworkEncoder::new();
        let sample = create_valid_sample();
        let trace = encoder.encode(&sample).unwrap();
        
        let validator = TraceValidator::new(false);
        let report = validator.validate(&trace);
        assert!(report.valid);
    }

    #[test]
    fn test_metrics_prometheus_export() {
        let metrics = EncoderMetrics {
            encode_count: 100,
            last_roh: 0.15,
            last_knowledge_factor: 0.80,
            last_safety_strength: 0.85,
            last_complexity: 12.5,
            last_nullspace_dim: 256,
        };
        let prom_lines = metrics.to_prometheus();
        assert_eq!(prom_lines.len(), 6);
        assert!(prom_lines[0].contains("biomem_encode_count_total"));
    }

    #[test]
    fn test_internal_state_builder() {
        let encoder = EegNetworkEncoder::new();
        let sample = create_valid_sample();
        let state = encoder.build_internal_state(&sample);
        
        assert_eq!(state.molecular.len(), 0);
        assert_eq!(state.synaptic.len(), 0);
        assert_eq!(state.microcircuit.len(), 2);
        assert_eq!(state.network.len(), 1);
        assert_eq!(state.systemic.len(), 4);
    }
}
