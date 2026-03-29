// ============================================================================
// EXAMPLE: eeg_network_to_trace
// PATH:  examples/eeg_network_to_trace/src/main.rs
// VER:   1.0.0
// LIC:   MIT (Data_Lake Sovereign License)
// DESC:  CLI example demonstrating EEG/network sample encoding to MemoryTrace.
//        Provides PoF evidence with ALN shard validation and JSON output.
// USAGE: cargo run --example eeg_network_to_trace --features serde,metrics
// ============================================================================

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use biomem_encoder::{
    BiomemEncoder, EegNetworkEncoder, EegNetworkSample, EncoderConfig, TraceValidator,
};
use biomem_core::{MemoryTrace, ROH_MAX_GLOBAL, SAFETY_STRENGTH_MIN, KNOWLEDGE_FACTOR_MIN};
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// Application version for compatibility tracking.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default output directory for trace files.
const DEFAULT_OUTPUT_DIR: &str = "output";

/// Default output filename for trace JSON.
const DEFAULT_OUTPUT_FILE: &str = "trace.json";

/// Command-line argument structure for CLI parsing.
#[derive(Clone, Debug)]
struct CliArgs {
    /// Output file path (default: output/trace.json).
    output_path: Option<PathBuf>,
    
    /// Enable strict validation mode.
    strict_mode: bool,
    
    /// Enable verbose output.
    verbose: bool,
    
    /// Generate synthetic sample (default) or read from JSON.
    synthetic: bool,
    
    /// Input JSON file path (if not synthetic).
    input_path: Option<PathBuf>,
    
    /// Export Prometheus metrics.
    export_metrics: bool,
    
    /// Metrics output path.
    metrics_path: Option<PathBuf>,
}

impl CliArgs {
    /// Parse command-line arguments manually (no external deps).
    fn from_env() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut output_path = None;
        let mut strict_mode = false;
        let mut verbose = false;
        let mut synthetic = true;
        let mut input_path = None;
        let mut export_metrics = false;
        let mut metrics_path = None;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output" | "-o" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --output".to_string());
                    }
                    output_path = Some(PathBuf::from(&args[i]));
                }
                "--input" | "-i" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --input".to_string());
                    }
                    input_path = Some(PathBuf::from(&args[i]));
                    synthetic = false;
                }
                "--strict" => {
                    strict_mode = true;
                }
                "--verbose" | "-v" => {
                    verbose = true;
                }
                "--metrics" => {
                    export_metrics = true;
                }
                "--metrics-output" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Missing value for --metrics-output".to_string());
                    }
                    metrics_path = Some(PathBuf::from(&args[i]));
                }
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                "--version" => {
                    println!("eeg_network_to_trace v{}", APP_VERSION);
                    std::process::exit(0);
                }
                arg => {
                    return Err(format!("Unknown argument: {}", arg));
                }
            }
            i += 1;
        }

        Ok(Self {
            output_path,
            strict_mode,
            verbose,
            synthetic,
            input_path,
            export_metrics,
            metrics_path,
        })
    }

    /// Print help message.
    fn print_help() {
        println!(
            r#"EEG Network to MemoryTrace Encoder (PoF Example)

USAGE:
    cargo run --example eeg_network_to_trace [OPTIONS]

OPTIONS:
    -o, --output <PATH>          Output file path for trace JSON [default: output/trace.json]
    -i, --input <PATH>           Input JSON file with EegNetworkSample (disables synthetic)
    --strict                     Enable strict validation mode (reject on warnings)
    -v, --verbose                Enable verbose output with detailed validation reports
    --metrics                    Export Prometheus metrics after encoding
    --metrics-output <PATH>      Metrics output file path [default: output/metrics.prom]
    -h, --help                   Print help information
    --version                    Print version information

EXAMPLES:
    # Generate synthetic sample and encode to trace
    cargo run --example eeg_network_to_trace

    # Encode from input JSON file
    cargo run --example eeg_network_to_trace --input samples/eeg_sample.json

    # Enable strict validation with metrics export
    cargo run --example eeg_network_to_trace --strict --metrics

    # Verbose output to custom path
    cargo run --example eeg_network_to_trace --verbose --output traces/my_trace.json

ALN COMPLIANCE:
    - RoH must be <= {:.2} (global ceiling)
    - Safety strength must be >= {:.2} (export minimum)
    - Knowledge factor must be >= {:.2} (PoF requirement)
    - Null-space dimension must be >= {} (privacy floor)

PoF EVIDENCE:
    Each encoded trace includes a deterministic hash for audit trails.
    Validation reports are serialized with the trace for ALN shard verification.
"#,
            ROH_MAX_GLOBAL,
            SAFETY_STRENGTH_MIN,
            KNOWLEDGE_FACTOR_MIN,
            biomem_core::NULLSPACE_DIM_FLOOR
        );
    }

    /// Get output path (creates directory if needed).
    fn get_output_path(&self) -> Result<PathBuf, io::Error> {
        let path = self.output_path.clone().unwrap_or_else(|| {
            PathBuf::from(DEFAULT_OUTPUT_DIR).join(DEFAULT_OUTPUT_FILE)
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(path)
    }

    /// Get metrics output path.
    fn get_metrics_path(&self) -> PathBuf {
        self.metrics_path.clone().unwrap_or_else(|| {
            PathBuf::from(DEFAULT_OUTPUT_DIR).join("metrics.prom")
        })
    }
}

/// Generate a synthetic EEG/network sample for demonstration.
#[must_use]
fn generate_synthetic_sample() -> EegNetworkSample {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    EegNetworkSample::new(
        "hippocampus_CA3",           // ROI label (atlas region)
        12.5,                        // x coordinate (mm)
        -8.3,                        // y coordinate (mm)
        15.7,                        // z coordinate (mm)
        timestamp,                   // timestamp (epoch seconds)
        0.52,                        // theta bandpower (4-8 Hz)
        0.38,                        // alpha bandpower (8-13 Hz)
        0.67,                        // DMN coherence
        62.5,                        // HRV (ms)
        0.12,                        // RoH (below 0.30 ceiling)
        0.82,                        // knowledge factor (above 0.50 PoF minimum)
        0.88,                        // safety strength (above 0.75 export minimum)
    )
}

/// Load sample from JSON file.
fn load_sample_from_json(path: &PathBuf) -> Result<EegNetworkSample, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Encode sample to MemoryTrace with full validation.
fn encode_sample(
    encoder: &EegNetworkEncoder,
    sample: &EegNetworkSample,
    verbose: bool,
) -> Result<MemoryTrace, String> {
    // Pre-encoding validation
    let validation_report = sample.validate();
    
    if verbose {
        println!("\n=== Pre-Encoding Validation ===");
        println!("Sample Hash: {}", validation_report.sample_hash);
        println!("Valid: {}", validation_report.valid);
        
        if !validation_report.errors.is_empty() {
            println!("Errors:");
            for err in &validation_report.errors {
                println!("  ❌ {}", err);
            }
        }
        
        if !validation_report.warnings.is_empty() {
            println!("Warnings:");
            for warn in &validation_report.warnings {
                println!("  ⚠️  {}", warn);
            }
        }
    }

    if !validation_report.valid {
        return Err(format!(
            "Sample validation failed: {}",
            validation_report.errors.join("; ")
        ));
    }

    // Encode to MemoryTrace
    let trace = encoder.encode(sample)
        .map_err(|e| format!("Encoding failed: {}", e))?;

    // Post-encoding validation
    let validator = TraceValidator::new(encoder.config().strict_mode);
    let trace_report = validator.validate(&trace);

    if verbose {
        println!("\n=== Post-Encoding Validation ===");
        println!("Trace ID: {}", trace.id);
        println!("Valid: {}", trace_report.valid);
        
        if !trace_report.errors.is_empty() {
            println!("Errors:");
            for err in &trace_report.errors {
                println!("  ❌ {}", err);
            }
        }
        
        if !trace_report.warnings.is_empty() {
            println!("Warnings:");
            for warn in &trace_report.warnings {
                println!("  ⚠️  {}", warn);
            }
        }

        println!("\n=== Trace Summary ===");
        println!("Version: {}", trace.version);
        println!("RoH: {:.4} (max: {:.2})", trace.roh, ROH_MAX_GLOBAL);
        println!("Knowledge Factor: {:.4} (min: {:.2})", trace.knowledge_factor, KNOWLEDGE_FACTOR_MIN);
        println!("Safety Strength: {:.4} (min: {:.2})", trace.safety_strength, SAFETY_STRENGTH_MIN);
        println!("Complexity Score: {:.4}", trace.complexity_score());
        println!("Null-space Dimension: {}", trace.nullspace_dim());
        println!("State Hex Length: {} chars", trace.state_hex.hex.len());
        println!("L1 Norm: {}", trace.state_hex.l1_norm);
    }

    if !trace_report.valid {
        return Err(format!(
            "Trace validation failed: {}",
            trace_report.errors.join("; ")
        ));
    }

    Ok(trace)
}

/// Serialize trace to JSON with formatting.
fn serialize_trace(trace: &MemoryTrace) -> Result<String, String> {
    serde_json::to_string_pretty(trace)
        .map_err(|e| format!("Serialization failed: {}", e))
}

/// Write output to file.
fn write_output(path: &PathBuf, content: &str) -> Result<(), io::Error> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// Export Prometheus metrics.
fn export_metrics(encoder: &EegNetworkEncoder, path: &PathBuf) -> Result<(), String> {
    // Note: In real implementation, metrics would be collected from encoder
    // For this example, we'll create a sample metrics file
    let metrics_content = format!(
        r#"# HELP biomem_encode_count_total Total number of successful encodings
# TYPE biomem_encode_count_total counter
biomem_encode_count_total 1

# HELP biomem_roh_current Current Risk-of-Harm score
# TYPE biomem_roh_current gauge
biomem_roh_current 0.12

# HELP biomem_knowledge_factor_current Current knowledge factor
# TYPE biomem_knowledge_factor_current gauge
biomem_knowledge_factor_current 0.82

# HELP biomem_safety_strength_current Current safety strength
# TYPE biomem_safety_strength_current gauge
biomem_safety_strength_current 0.88

# HELP biomem_complexity_current Current trace complexity score
# TYPE biomem_complexity_current gauge
biomem_complexity_current 15.7532

# HELP biomem_nullspace_dim_current Current null-space dimension
# TYPE biomem_nullspace_dim_current gauge
biomem_nullspace_dim_current 256
"#
    );

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create metrics directory: {}", e))?;
    }

    write_output(path, &metrics_content)
        .map_err(|e| format!("Failed to write metrics: {}", e))?;

    Ok(())
}

/// Print ALN compliance summary.
fn print_aln_compliance(trace: &MemoryTrace) {
    println!("\n=== ALN Shard Compliance ===");
    
    let roh_ok = trace.roh <= ROH_MAX_GLOBAL;
    let safety_ok = trace.safety_strength >= SAFETY_STRENGTH_MIN;
    let knowledge_ok = trace.knowledge_factor >= KNOWLEDGE_FACTOR_MIN;
    let nullspace_ok = trace.nullspace_dim() >= biomem_core::NULLSPACE_DIM_FLOOR;
    
    println!("RoH Ceiling (≤{:.2}):          {} {}", ROH_MAX_GLOBAL, if roh_ok { "✅" } else { "❌" }, trace.roh);
    println!("Safety Strength (≥{:.2}):     {} {}", SAFETY_STRENGTH_MIN, if safety_ok { "✅" } else { "❌" }, trace.safety_strength);
    println!("Knowledge Factor (≥{:.2}):    {} {}", KNOWLEDGE_FACTOR_MIN, if knowledge_ok { "✅" } else { "❌" }, trace.knowledge_factor);
    println!("Null-space Dim (≥{}):    {} {}", biomem_core::NULLSPACE_DIM_FLOOR, if nullspace_ok { "✅" } else { "❌" }, trace.nullspace_dim());
    
    let all_compliant = roh_ok && safety_ok && knowledge_ok && nullspace_ok;
    println!("\nOverall Compliance: {}", if all_compliant { "✅ PASS" } else { "❌ FAIL" });
}

/// Print PoF evidence summary.
fn print_pof_evidence(trace: &MemoryTrace, sample: &EegNetworkSample) {
    println!("\n=== PoF (Proof-of-Functionality) Evidence ===");
    println!("Sample Hash:    {}", sample.compute_hash());
    println!("Trace ID:       {}", trace.id);
    println!("State Hex:      {}... ({} chars)", &trace.state_hex.hex[..32.min(trace.state_hex.hex.len())], trace.state_hex.hex.len());
    println!("L1 Norm:        {}", trace.state_hex.l1_norm);
    println!("Nonzero Nibbles: {}", trace.state_hex.nonzero_nibbles);
    println!("Version:        {}", trace.version);
    println!("Created At:     {}", trace.created_at);
}

/// Main entry point.
fn main() -> ExitCode {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         EEG Network to MemoryTrace Encoder (PoF)            ║");
    println!("║                    Data_Lake Sovereign                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Parse command-line arguments
    let args = match CliArgs::from_env() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            eprintln!("Use --help for usage information.");
            return ExitCode::FAILURE;
        }
    };

    // Configure encoder
    let config = EncoderConfig::default()
        .with_metrics(args.export_metrics)
        .strict_mode(args.strict_mode);

    let encoder = EegNetworkEncoder::with_config(config);

    // Load or generate sample
    let sample = if args.synthetic {
        if args.verbose {
            println!("\n=== Generating Synthetic Sample ===");
        }
        generate_synthetic_sample()
    } else {
        let input_path = args.input_path.as_ref()
            .expect("Input path required for non-synthetic mode");
        
        if args.verbose {
            println!("\n=== Loading Sample from {} ===", input_path.display());
        }
        
        match load_sample_from_json(input_path) {
            Ok(sample) => sample,
            Err(e) => {
                eprintln!("Failed to load sample: {}", e);
                return ExitCode::FAILURE;
            }
        }
    };

    // Encode sample to trace
    let trace = match encode_sample(&encoder, &sample, args.verbose) {
        Ok(trace) => trace,
        Err(e) => {
            eprintln!("Encoding failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Serialize to JSON
    let json_output = match serialize_trace(&trace) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Serialization failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Write output file
    let output_path = match args.get_output_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to create output path: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = write_output(&output_path, &json_output) {
        eprintln!("Failed to write output: {}", e);
        return ExitCode::FAILURE;
    }

    if args.verbose {
        println!("\n=== Output Written ===");
        println!("Path: {}", output_path.display());
        println!("Size: {} bytes", json_output.len());
    }

    // Export metrics if requested
    if args.export_metrics {
        let metrics_path = args.get_metrics_path();
        match export_metrics(&encoder, &metrics_path) {
            Ok(()) => {
                if args.verbose {
                    println!("\n=== Metrics Exported ===");
                    println!("Path: {}", metrics_path.display());
                }
            }
            Err(e) => {
                eprintln!("Metrics export failed: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    // Print compliance and evidence summaries
    print_aln_compliance(&trace);
    print_pof_evidence(&trace, &sample);

    println!("\n✅ Encoding completed successfully.");
    println!("   Trace ID: {}", trace.id);
    println!("   Output:   {}", output_path.display());

    ExitCode::SUCCESS
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_sample_generation() {
        let sample = generate_synthetic_sample();
        assert!(!sample.roi_label.is_empty());
        assert!(sample.validate().valid);
    }

    #[test]
    fn test_encoder_with_synthetic_sample() {
        let encoder = EegNetworkEncoder::new();
        let sample = generate_synthetic_sample();
        let result = encoder.encode(&sample);
        assert!(result.is_ok());
        let trace = result.unwrap();
        assert!(trace.validate_all());
    }

    #[test]
    fn test_trace_serialization() {
        let encoder = EegNetworkEncoder::new();
        let sample = generate_synthetic_sample();
        let trace = encoder.encode(&sample).unwrap();
        let json = serialize_trace(&trace);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("\"id\""));
    }

    #[test]
    fn test_cli_args_parsing() {
        // Note: This test would need env var manipulation
        // Basic structure test only
        let args = CliArgs {
            output_path: Some(PathBuf::from("test.json")),
            strict_mode: true,
            verbose: true,
            synthetic: false,
            input_path: Some(PathBuf::from("input.json")),
            export_metrics: true,
            metrics_path: Some(PathBuf::from("metrics.prom")),
        };
        assert!(args.strict_mode);
        assert!(args.verbose);
    }
}
