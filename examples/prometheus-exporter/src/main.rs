// ============================================================================
// EXAMPLE: prometheus-exporter
// PATH:  examples/prometheus-exporter/src/main.rs
// VER:   1.0.0
// LIC:   MIT (Data_Lake Sovereign License)
// DESC:  Prometheus metrics exporter for MemoryTrace monitoring.
//        Reads trace JSON files, extracts metrics, exposes via HTTP endpoint.
// USAGE: cargo run --example prometheus-exporter --features prometheus,serde
// ============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use biomem_core::MemoryTrace;
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Application version for compatibility tracking.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default HTTP bind address for metrics endpoint.
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:9090";

/// Default directory for trace JSON files.
const DEFAULT_TRACE_DIR: &str = "output";

/// Default scrape interval for trace directory polling.
const DEFAULT_SCRAPE_INTERVAL_SEC: u64 = 15;

/// Default retention period for metrics (seconds).
const DEFAULT_RETENTION_SEC: u64 = 86400; // 24 hours

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Application configuration from environment or CLI args.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExporterConfig {
    /// HTTP bind address for Prometheus scraping.
    pub bind_address: String,
    
    /// Directory containing MemoryTrace JSON files.
    pub trace_dir: PathBuf,
    
    /// Polling interval for trace directory (seconds).
    pub scrape_interval_sec: u64,
    
    /// Metrics retention period (seconds).
    pub retention_sec: u64,
    
    /// Enable verbose logging.
    pub verbose: bool,
    
    /// Enable trace file deletion after processing.
    pub delete_after_process: bool,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            trace_dir: PathBuf::from(DEFAULT_TRACE_DIR),
            scrape_interval_sec: DEFAULT_SCRAPE_INTERVAL_SEC,
            retention_sec: DEFAULT_RETENTION_SEC,
            verbose: false,
            delete_after_process: false,
        }
    }
}

impl ExporterConfig {
    /// Load configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            bind_address: env::var("BIOMEM_BIND_ADDRESS")
                .unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string()),
            trace_dir: env::var("BIOMEM_TRACE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(DEFAULT_TRACE_DIR)),
            scrape_interval_sec: env::var("BIOMEM_SCRAPE_INTERVAL_SEC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_SCRAPE_INTERVAL_SEC),
            retention_sec: env::var("BIOMEM_RETENTION_SEC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_RETENTION_SEC),
            verbose: env::var("BIOMEM_VERBOSE")
                .ok()
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
            delete_after_process: env::var("BIOMEM_DELETE_AFTER_PROCESS")
                .ok()
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
        }
    }

    /// Validate configuration.
    pub fn validate(&self) -> Result<(), String> {
        if !self.trace_dir.exists() {
            return Err(format!("Trace directory does not exist: {:?}", self.trace_dir));
        }
        if !self.trace_dir.is_dir() {
            return Err(format!("Trace path is not a directory: {:?}", self.trace_dir));
        }
        if self.scrape_interval_sec == 0 {
            return Err("Scrape interval must be > 0".to_string());
        }
        if self.retention_sec == 0 {
            return Err("Retention period must be > 0".to_string());
        }
        Ok(())
    }
}

// ============================================================================
// METRICS REGISTRY
// ============================================================================

/// Prometheus metrics registry wrapper with biomem-specific metrics.
#[derive(Clone)]
pub struct MetricsRegistry {
    pub registry: Registry,
    pub encode_count: Counter,
    pub roh_gauge: Gauge,
    pub knowledge_gauge: Gauge,
    pub safety_gauge: Gauge,
    pub complexity_gauge: Gauge,
    pub nullspace_gauge: Gauge,
    pub l1_norm_gauge: Gauge,
    pub validation_errors: Counter,
    pub sko_wrapped: Counter,
    pub export_blocked: Counter,
    pub neurorights_violations: Counter,
    pub trace_age_histogram: Histogram,
    pub layer_dimension_gauge: Gauge,
    pub active_traces_gauge: Gauge,
    pub last_scrape_timestamp: Gauge,
    pub scrape_duration_histogram: Histogram,
}

impl MetricsRegistry {
    /// Create a new metrics registry with all biomem metrics registered.
    #[must_use]
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let encode_count = Counter::with_opts(Opts::new(
            "biomem_encode_count_total",
            "Total number of successful trace encodings"
        ))?;

        let roh_gauge = Gauge::with_opts(Opts::new(
            "biomem_roh_current",
            "Current Risk-of-Harm score (must be <= 0.30)"
        ))?;

        let knowledge_gauge = Gauge::with_opts(Opts::new(
            "biomem_knowledge_factor_current",
            "Current knowledge factor (must be >= 0.50 for PoF)"
        ))?;

        let safety_gauge = Gauge::with_opts(Opts::new(
            "biomem_safety_strength_current",
            "Current safety strength (must be >= 0.75 for export)"
        ))?;

        let complexity_gauge = Gauge::with_opts(Opts::new(
            "biomem_complexity_current",
            "Current trace complexity score (monotonic non-decreasing)"
        ))?;

        let nullspace_gauge = Gauge::with_opts(Opts::new(
            "biomem_nullspace_dim_current",
            "Current null-space dimension (must be >= 128 for privacy)"
        ))?;

        let l1_norm_gauge = Gauge::with_opts(Opts::new(
            "biomem_state_l1_norm_current",
            "Current L1 norm of state hex encoding"
        ))?;

        let validation_errors = Counter::with_opts(Opts::new(
            "biomem_validation_errors_total",
            "Total trace validation errors"
        ))?;

        let sko_wrapped = Counter::with_opts(Opts::new(
            "biomem_sko_wrapped_total",
            "Total SKO wrapping operations"
        ))?;

        let export_blocked = Counter::with_opts(Opts::new(
            "biomem_export_blocked_total",
            "Total blocked export attempts (RoH/safety violations)"
        ))?;

        let neurorights_violations = Counter::with_opts(Opts::new(
            "biomem_neurorights_violations_total",
            "Total neurorights violations detected"
        ))?;

        let trace_age_histogram = Histogram::with_opts(HistogramOpts::new(
            "biomem_trace_age_seconds",
            "Age of traces in seconds (from creation to scrape)"
        ).buckets(vec![
            1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0,
        ]))?;

        let layer_dimension_gauge = Gauge::with_opts(Opts::new(
            "biomem_layer_dimensions_total",
            "Total dimensions across all biophysical layers"
        ))?;

        let active_traces_gauge = Gauge::with_opts(Opts::new(
            "biomem_active_traces_count",
            "Number of active traces in memory"
        ))?;

        let last_scrape_timestamp = Gauge::with_opts(Opts::new(
            "biomem_last_scrape_timestamp",
            "Unix timestamp of last successful scrape"
        ))?;

        let scrape_duration_histogram = Histogram::with_opts(HistogramOpts::new(
            "biomem_scrape_duration_seconds",
            "Duration of scrape operations in seconds"
        ).buckets(vec![
            0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]))?;

        registry.register(Box::new(encode_count.clone()))?;
        registry.register(Box::new(roh_gauge.clone()))?;
        registry.register(Box::new(knowledge_gauge.clone()))?;
        registry.register(Box::new(safety_gauge.clone()))?;
        registry.register(Box::new(complexity_gauge.clone()))?;
        registry.register(Box::new(nullspace_gauge.clone()))?;
        registry.register(Box::new(l1_norm_gauge.clone()))?;
        registry.register(Box::new(validation_errors.clone()))?;
        registry.register(Box::new(sko_wrapped.clone()))?;
        registry.register(Box::new(export_blocked.clone()))?;
        registry.register(Box::new(neurorights_violations.clone()))?;
        registry.register(Box::new(trace_age_histogram.clone()))?;
        registry.register(Box::new(layer_dimension_gauge.clone()))?;
        registry.register(Box::new(active_traces_gauge.clone()))?;
        registry.register(Box::new(last_scrape_timestamp.clone()))?;
        registry.register(Box::new(scrape_duration_histogram.clone()))?;

        Ok(Self {
            registry,
            encode_count,
            roh_gauge,
            knowledge_gauge,
            safety_gauge,
            complexity_gauge,
            nullspace_gauge,
            l1_norm_gauge,
            validation_errors,
            sko_wrapped,
            export_blocked,
            neurorights_violations,
            trace_age_histogram,
            layer_dimension_gauge,
            active_traces_gauge,
            last_scrape_timestamp,
            scrape_duration_histogram,
        })
    }

    /// Record metrics from a MemoryTrace.
    pub fn record_trace(&self, trace: &MemoryTrace) {
        self.encode_count.inc();
        self.roh_gauge.set(trace.roh as f64);
        self.knowledge_gauge.set(trace.knowledge_factor as f64);
        self.safety_gauge.set(trace.safety_strength as f64);
        self.complexity_gauge.set(trace.complexity_score() as f64);
        self.nullspace_gauge.set(trace.nullspace_dim() as f64);
        self.l1_norm_gauge.set(trace.state_hex.l1_norm as f64);
        self.layer_dimension_gauge.set(trace.coord.state.total_dimension_count() as f64);
    }

    /// Record validation error.
    pub fn record_validation_error(&self) {
        self.validation_errors.inc();
    }

    /// Record SKO wrap operation.
    pub fn record_sko_wrap(&self) {
        self.sko_wrapped.inc();
    }

    /// Record blocked export attempt.
    pub fn record_export_blocked(&self) {
        self.export_blocked.inc();
    }

    /// Record neurorights violation.
    pub fn record_neurorights_violation(&self) {
        self.neurorights_violations.inc();
    }

    /// Record trace age.
    pub fn record_trace_age(&self, age_seconds: f64) {
        self.trace_age_histogram.observe(age_seconds);
    }

    /// Update active traces count.
    pub fn set_active_traces(&self, count: usize) {
        self.active_traces_gauge.set(count as f64);
    }

    /// Record scrape completion.
    pub fn record_scrape_complete(&self, duration_seconds: f64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        self.last_scrape_timestamp.set(now);
        self.scrape_duration_histogram.observe(duration_seconds);
    }

    /// Export metrics as Prometheus text format.
    #[must_use]
    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap_or_default();
        String::from_utf8(buffer).unwrap_or_default()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics registry")
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Shared application state for Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: ExporterConfig,
    pub metrics: Arc<MetricsRegistry>,
    pub processed_traces: Arc<RwLock<HashMap<String, TraceMetadata>>>,
}

/// Metadata for processed traces (for deduplication and age tracking).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub trace_id: String,
    pub processed_at: f64,
    pub created_at: f64,
    pub file_path: String,
    pub roh: f32,
    pub knowledge_factor: f32,
    pub safety_strength: f32,
}

impl TraceMetadata {
    #[must_use]
    pub fn new(trace: &MemoryTrace, file_path: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        
        Self {
            trace_id: trace.id.clone(),
            processed_at: now,
            created_at: trace.created_at,
            file_path: file_path.to_string(),
            roh: trace.roh,
            knowledge_factor: trace.knowledge_factor,
            safety_strength: trace.safety_strength,
        }
    }

    #[must_use]
    pub fn age_seconds(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        now - self.created_at
    }
}

// ============================================================================
// TRACE PROCESSOR
// ============================================================================

/// Process MemoryTrace JSON files from directory.
pub struct TraceProcessor {
    config: ExporterConfig,
    metrics: Arc<MetricsRegistry>,
    processed_traces: Arc<RwLock<HashMap<String, TraceMetadata>>>,
}

impl TraceProcessor {
    #[must_use]
    pub fn new(
        config: ExporterConfig,
        metrics: Arc<MetricsRegistry>,
        processed_traces: Arc<RwLock<HashMap<String, TraceMetadata>>>,
    ) -> Self {
        Self {
            config,
            metrics,
            processed_traces,
        }
    }

    /// Scan directory for new trace files and process them.
    pub async fn scan_and_process(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        let mut processed_count = 0;

        // Read directory entries
        let entries = fs::read_dir(&self.config.trace_dir)?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Skip non-JSON files
            if path.extension().map_or(true, |ext| ext != "json") {
                continue;
            }

            // Check if already processed
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            {
                let processed = self.processed_traces.read().await;
                if processed.contains_key(&file_name) {
                    continue;
                }
            }

            // Process trace file
            match self.process_trace_file(&path).await {
                Ok(metadata) => {
                    processed_count += 1;
                    
                    // Store metadata for deduplication
                    {
                        let mut processed = self.processed_traces.write().await;
                        processed.insert(file_name.clone(), metadata);
                    }

                    if self.config.verbose {
                        info!("Processed trace file: {:?}", path);
                    }

                    // Delete after processing if configured
                    if self.config.delete_after_process {
                        if let Err(e) = fs::remove_file(&path) {
                            warn!("Failed to delete processed file {:?}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to process trace file {:?}: {}", path, e);
                    self.metrics.record_validation_error();
                }
            }
        }

        // Update active traces count
        {
            let processed = self.processed_traces.read().await;
            self.metrics.set_active_traces(processed.len());
        }

        // Record scrape duration
        let duration = start_time.elapsed().unwrap_or_default().as_secs_f64();
        self.metrics.record_scrape_complete(duration);

        // Cleanup old traces beyond retention period
        self.cleanup_old_traces().await?;

        Ok(processed_count)
    }

    /// Process a single trace JSON file.
    async fn process_trace_file(
        &self,
        path: &Path,
    ) -> Result<TraceMetadata, Box<dyn std::error::Error>> {
        // Read file content
        let content = fs::read_to_string(path)?;
        
        // Parse MemoryTrace
        let trace: MemoryTrace = serde_json::from_str(&content)?;
        
        // Validate trace
        if !trace.validate_all() {
            self.metrics.record_export_blocked();
            
            // Check for neurorights violations
            if trace.roh > biomem_core::ROH_MAX_GLOBAL {
                self.metrics.record_neurorights_violation();
            }
            
            return Err(format!("Trace validation failed: {}", trace.id).into());
        }

        // Record metrics
        self.metrics.record_trace(&trace);
        
        // Record trace age
        let age = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            - trace.created_at;
        self.metrics.record_trace_age(age);

        // Create metadata
        let metadata = TraceMetadata::new(&trace, path.to_str().unwrap_or(""));

        Ok(metadata)
    }

    /// Cleanup traces older than retention period.
    async fn cleanup_old_traces(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        
        let retention_cutoff = now - self.config.retention_sec as f64;
        
        let mut to_remove = Vec::new();
        
        {
            let processed = self.processed_traces.read().await;
            for (key, metadata) in processed.iter() {
                if metadata.processed_at < retention_cutoff {
                    to_remove.push(key.clone());
                }
            }
        }

        if !to_remove.is_empty() {
            let mut processed = self.processed_traces.write().await;
            for key in to_remove {
                processed.remove(&key);
            }
            
            if self.config.verbose {
                info!("Cleaned up {} old trace entries", to_remove.len());
            }
        }

        Ok(())
    }
}

// ============================================================================
// HTTP HANDLERS
// ============================================================================

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Prometheus metrics endpoint.
async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.gather();
    (StatusCode::OK, metrics)
}

/// Status endpoint with application info.
async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let processed = state.processed_traces.read().await;
    
    let status = serde_json::json!({
        "version": APP_VERSION,
        "bind_address": state.config.bind_address,
        "trace_dir": state.config.trace_dir.to_string_lossy(),
        "scrape_interval_sec": state.config.scrape_interval_sec,
        "retention_sec": state.config.retention_sec,
        "active_traces": processed.len(),
        "uptime_seconds": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    });

    (StatusCode::OK, status.to_string())
}

/// Traces list endpoint.
async fn traces_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let processed = state.processed_traces.read().await;
    
    let traces: Vec<_> = processed.values()
        .map(|m| {
            serde_json::json!({
                "trace_id": m.trace_id,
                "processed_at": m.processed_at,
                "created_at": m.created_at,
                "age_seconds": m.age_seconds(),
                "roh": m.roh,
                "knowledge_factor": m.knowledge_factor,
                "safety_strength": m.safety_strength,
            })
        })
        .collect();

    (StatusCode::OK, serde_json::json!({ "traces": traces }).to_string())
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║         Mem-Brain Prometheus Exporter (PoF)                 ║");
    info!("║                    Data_Lake Sovereign                       ║");
    info!("╚══════════════════════════════════════════════════════════════╝");

    // Load configuration
    let config = ExporterConfig::from_env();
    
    // Validate configuration
    config.validate()?;
    
    info!("Configuration loaded:");
    info!("  Bind Address:     {}", config.bind_address);
    info!("  Trace Directory:  {:?}", config.trace_dir);
    info!("  Scrape Interval:  {}s", config.scrape_interval_sec);
    info!("  Retention:        {}s", config.retention_sec);
    info!("  Verbose:          {}", config.verbose);
    info!("  Delete After:     {}", config.delete_after_process);

    // Create metrics registry
    let metrics = Arc::new(MetricsRegistry::new()?);
    info!("Metrics registry initialized");

    // Create shared state
    let processed_traces: Arc<RwLock<HashMap<String, TraceMetadata>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    
    let app_state = Arc::new(AppState {
        config: config.clone(),
        metrics: metrics.clone(),
        processed_traces: processed_traces.clone(),
    });

    // Build Axum router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/status", get(status_handler))
        .route("/traces", get(traces_handler))
        .with_state(app_state.clone());

    // Start background trace processor
    let processor_config = config.clone();
    let processor_metrics = metrics.clone();
    let processor_traces = processed_traces.clone();
    
    tokio::spawn(async move {
        let processor = TraceProcessor::new(
            processor_config,
            processor_metrics,
            processor_traces,
        );
        
        let mut interval = interval(Duration::from_secs(processor.config.scrape_interval_sec));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            
            match processor.scan_and_process().await {
                Ok(count) => {
                    if count > 0 {
                        info!("Scrape complete: {} new traces processed", count);
                    }
                }
                Err(e) => {
                    error!("Scrape failed: {}", e);
                }
            }
        }
    });

    info!("Background trace processor started");

    // Parse bind address
    let addr: SocketAddr = config.bind_address.parse()?;
    info!("Starting HTTP server on {}", addr);

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ExporterConfig::default();
        assert_eq!(config.bind_address, DEFAULT_BIND_ADDRESS);
        assert_eq!(config.trace_dir, PathBuf::from(DEFAULT_TRACE_DIR));
        assert_eq!(config.scrape_interval_sec, DEFAULT_SCRAPE_INTERVAL_SEC);
    }

    #[test]
    fn test_config_validation() {
        let config = ExporterConfig::default();
        // Should fail if trace_dir doesn't exist
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_registry_creation() {
        let metrics = MetricsRegistry::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_trace_metadata_age() {
        let trace = biomem_core::MemoryTrace::new(
            "test-001",
            biomem_core::Coord5D::new(
                biomem_core::Space3D::origin(),
                biomem_core::TimeCoord::now_epoch(),
                biomem_core::InternalState::new(),
            ),
            0.10,
            0.80,
            0.85,
        );
        
        let metadata = TraceMetadata::new(&trace, "test.json");
        assert!(metadata.age_seconds() >= 0.0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.into_response().status(), StatusCode::OK);
    }
}
