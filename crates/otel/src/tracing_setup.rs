use anyhow::Result;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};
use opentelemetry_sdk::Resource;
use std::collections::HashMap;
use std::env;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const DEFAULT_OTLP_TIMEOUT_MS: u64 = 10_000;
const OTEL_METER_NAME: &str = "nettoolskit-cli.metrics";
static OTEL_TRACER_PROVIDER: OnceLock<Mutex<Option<SdkTracerProvider>>> = OnceLock::new();
static OTEL_METER_PROVIDER: OnceLock<Mutex<Option<SdkMeterProvider>>> = OnceLock::new();
static OTEL_METRIC_EXPORTER: OnceLock<Mutex<Option<OtlpMetricExporter>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtlpProtocolKind {
    Grpc,
    HttpBinary,
}

#[derive(Debug, Clone)]
struct OtlpExportConfig {
    endpoint: String,
    protocol: OtlpProtocolKind,
    timeout_ms: u64,
}

#[derive(Debug)]
struct OtlpMetricExporter {
    meter: Meter,
    counters: HashMap<String, Counter<u64>>,
    gauges: HashMap<String, Gauge<f64>>,
    timings_ms: HashMap<String, Histogram<f64>>,
}

impl OtlpMetricExporter {
    fn new(meter: Meter) -> Self {
        Self {
            meter,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            timings_ms: HashMap::new(),
        }
    }

    fn add_counter(&mut self, name: &str, value: u64) {
        let Some(metric_name) = sanitize_otlp_metric_name(name) else {
            return;
        };

        if !self.counters.contains_key(&metric_name) {
            let counter = self.meter.u64_counter(metric_name.clone()).build();
            self.counters.insert(metric_name.clone(), counter);
        }

        if let Some(counter) = self.counters.get(&metric_name) {
            counter.add(value, &[]);
        }
    }

    fn record_gauge(&mut self, name: &str, value: f64) {
        if !value.is_finite() {
            return;
        }

        let Some(metric_name) = sanitize_otlp_metric_name(name) else {
            return;
        };

        if !self.gauges.contains_key(&metric_name) {
            let gauge = self.meter.f64_gauge(metric_name.clone()).build();
            self.gauges.insert(metric_name.clone(), gauge);
        }

        if let Some(gauge) = self.gauges.get(&metric_name) {
            gauge.record(value, &[]);
        }
    }

    fn record_timing_ms(&mut self, name: &str, value_ms: f64) {
        if !value_ms.is_finite() {
            return;
        }

        let Some(metric_name) = sanitize_otlp_metric_name(name) else {
            return;
        };

        if !self.timings_ms.contains_key(&metric_name) {
            let histogram = self
                .meter
                .f64_histogram(metric_name.clone())
                .with_unit("ms")
                .build();
            self.timings_ms.insert(metric_name.clone(), histogram);
        }

        if let Some(histogram) = self.timings_ms.get(&metric_name) {
            histogram.record(value_ms, &[]);
        }
    }
}

/// Tracing configuration for different environments
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Enable verbose (debug-level) log output.
    pub verbose: bool,
    /// Requested base log level (off, error, warn, info, debug, trace).
    pub log_level: String,
    /// Emit log records in structured JSON format.
    pub json_format: bool,
    /// Include the source-file path in each log line.
    pub with_file: bool,
    /// Include source line numbers in each log line.
    pub with_line_numbers: bool,
    /// OpenTelemetry service name used for resource identification.
    pub service_name: String,
    /// Semantic version of the service reported to the telemetry backend.
    pub service_version: String,
    /// When `true`, tracing output is routed through the interactive UI writer.
    pub interactive_mode: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            log_level: "info".to_string(),
            json_format: false,
            with_file: false,
            with_line_numbers: true,
            service_name: "nettoolskit-cli".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            interactive_mode: false,
        }
    }
}

/// Initialize comprehensive tracing for NetToolsKit CLI with structured logging.
pub fn init_tracing(verbose: bool) -> Result<()> {
    let config = TracingConfig {
        verbose,
        log_level: if verbose {
            "debug".to_string()
        } else {
            "info".to_string()
        },
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Initialize tracing with full configuration.
pub fn init_tracing_with_config(config: TracingConfig) -> Result<()> {
    let filter = create_env_filter(&config)?;

    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        verbose = config.verbose,
        "Initializing structured logging and tracing"
    );

    // Optional OTLP tracer (enabled via environment variables).
    let otlp_tracer = build_otlp_tracer(&config)?;
    configure_otlp_metrics_exporter(&config)?;

    let registry = tracing_subscriber::registry().with(filter);

    if config.interactive_mode {
        if let Some(tracer) = otlp_tracer {
            registry
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_file(config.with_file)
                        .with_line_number(config.with_line_numbers)
                        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                        .with_writer(nettoolskit_ui::UiWriter::new)
                        .compact(),
                )
                .try_init()?;
        } else {
            registry
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_file(config.with_file)
                        .with_line_number(config.with_line_numbers)
                        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                        .with_writer(nettoolskit_ui::UiWriter::new)
                        .compact(),
                )
                .try_init()?;
        }
    } else if config.json_format {
        warn!("JSON format requested but not available in current setup, using pretty format");
        if let Some(tracer) = otlp_tracer {
            registry
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_file(config.with_file)
                        .with_line_number(config.with_line_numbers)
                        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                        .with_writer(std::io::stderr)
                        .pretty(),
                )
                .try_init()?;
        } else {
            registry
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_file(config.with_file)
                        .with_line_number(config.with_line_numbers)
                        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                        .with_writer(std::io::stderr)
                        .pretty(),
                )
                .try_init()?;
        }
    } else if let Some(tracer) = otlp_tracer {
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(
                fmt::layer()
                    .with_target(false)
                    .with_file(config.with_file)
                    .with_line_number(config.with_line_numbers)
                    .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                    .with_writer(std::io::stderr)
                    .compact(),
            )
            .try_init()?;
    } else {
        registry
            .with(
                fmt::layer()
                    .with_target(false)
                    .with_file(config.with_file)
                    .with_line_number(config.with_line_numbers)
                    .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                    .with_writer(std::io::stderr)
                    .compact(),
            )
            .try_init()?;
    }

    info!(config = ?config, "Tracing initialized successfully");
    Ok(())
}

/// Initialize tracing with custom filter string.
pub fn init_tracing_with_filter(filter: &str) -> Result<()> {
    let filter = EnvFilter::try_new(filter)?;
    let otlp_tracer = build_otlp_tracer(&TracingConfig::default())?;
    configure_otlp_metrics_exporter(&TracingConfig::default())?;

    info!(
        filter = %filter,
        "Initializing tracing with custom filter"
    );

    let base = tracing_subscriber::registry().with(filter);
    if let Some(tracer) = otlp_tracer {
        base.with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                    .with_writer(std::io::stderr),
            )
            .try_init()?;
    } else {
        base.with(
            fmt::layer()
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                .with_writer(std::io::stderr),
        )
        .try_init()?;
    }

    Ok(())
}

/// Create environment filter based on configuration.
fn create_env_filter(config: &TracingConfig) -> Result<EnvFilter> {
    if let Ok(env_filter) = env::var("RUST_LOG") {
        info!(filter = %env_filter, "Using RUST_LOG environment filter");
        return Ok(EnvFilter::try_new(env_filter)?);
    }

    let default_filter = default_filter_for_level(&config.log_level, config.verbose);

    info!(
        filter = %default_filter,
        requested_level = %config.log_level,
        "Using default filter configuration"
    );
    Ok(EnvFilter::try_new(default_filter)?)
}

fn default_filter_for_level(level: &str, verbose: bool) -> &'static str {
    match level.trim().to_ascii_lowercase().as_str() {
        "off" => "off",
        "error" => "nettoolskit=error,error",
        "warn" | "warning" => "nettoolskit=warn,warn",
        "info" => "nettoolskit=info,warn",
        "debug" => {
            "nettoolskit=debug,nettoolskit_cli=debug,nettoolskit_commands=debug,nettoolskit_ui=debug,info"
        }
        "trace" => {
            "nettoolskit=trace,nettoolskit_cli=trace,nettoolskit_commands=trace,nettoolskit_ui=trace,debug"
        }
        _ => {
            if verbose {
                "nettoolskit=debug,nettoolskit_cli=debug,nettoolskit_commands=debug,nettoolskit_ui=debug,info"
            } else {
                "nettoolskit=info,warn"
            }
        }
    }
}

/// Initialize minimal tracing for production.
pub fn init_production_tracing() -> Result<()> {
    let config = TracingConfig {
        verbose: false,
        json_format: true,
        with_file: false,
        with_line_numbers: false,
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Initialize development tracing with full details.
pub fn init_development_tracing() -> Result<()> {
    let config = TracingConfig {
        verbose: true,
        log_level: "debug".to_string(),
        json_format: false,
        with_file: true,
        with_line_numbers: true,
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Gracefully shutdown tracing.
pub fn shutdown_tracing() {
    shutdown_tracer_provider();
    shutdown_meter_provider();
    debug!("Tracing subsystem shutdown completed");
}

pub(crate) fn record_otlp_counter(name: &str, value: u64) {
    with_otlp_metric_exporter(|exporter| exporter.add_counter(name, value));
}

pub(crate) fn record_otlp_gauge(name: &str, value: f64) {
    with_otlp_metric_exporter(|exporter| exporter.record_gauge(name, value));
}

pub(crate) fn record_otlp_timing(name: &str, duration: Duration) {
    let value_ms = duration.as_secs_f64() * 1000.0;
    with_otlp_metric_exporter(|exporter| exporter.record_timing_ms(name, value_ms));
}

fn with_otlp_metric_exporter<F>(mut action: F)
where
    F: FnMut(&mut OtlpMetricExporter),
{
    let Some(storage) = OTEL_METRIC_EXPORTER.get() else {
        return;
    };

    let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(exporter) = guard.as_mut() {
        action(exporter);
    }
}

fn build_otlp_tracer(config: &TracingConfig) -> Result<Option<Tracer>> {
    let Some(otlp) = resolve_otlp_trace_export_config() else {
        return Ok(None);
    };

    info!(
        endpoint = %otlp.endpoint,
        protocol = ?otlp.protocol,
        timeout_ms = otlp.timeout_ms,
        "OpenTelemetry OTLP trace export enabled"
    );

    let timeout = Duration::from_millis(otlp.timeout_ms);
    let exporter = match otlp.protocol {
        OtlpProtocolKind::Grpc => opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp.endpoint.clone())
            .with_timeout(timeout)
            .build()?,
        OtlpProtocolKind::HttpBinary => opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(otlp.endpoint.clone())
            .with_timeout(timeout)
            .build()?,
    };

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(build_resource(config))
        .build();

    let tracer = provider.tracer(config.service_name.clone());
    register_tracer_provider(provider.clone());
    global::set_tracer_provider(provider);

    Ok(Some(tracer))
}

fn configure_otlp_metrics_exporter(config: &TracingConfig) -> Result<()> {
    let Some(otlp) = resolve_otlp_metric_export_config() else {
        shutdown_meter_provider();
        return Ok(());
    };

    info!(
        endpoint = %otlp.endpoint,
        protocol = ?otlp.protocol,
        timeout_ms = otlp.timeout_ms,
        "OpenTelemetry OTLP metrics export enabled"
    );

    let timeout = Duration::from_millis(otlp.timeout_ms);
    let exporter = match otlp.protocol {
        OtlpProtocolKind::Grpc => opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(otlp.endpoint)
            .with_timeout(timeout)
            .build()?,
        OtlpProtocolKind::HttpBinary => opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(otlp.endpoint)
            .with_timeout(timeout)
            .build()?,
    };

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(build_resource(config))
        .build();

    register_meter_provider(provider.clone());
    global::set_meter_provider(provider);
    install_metric_exporter_state();
    Ok(())
}

fn build_resource(config: &TracingConfig) -> Resource {
    Resource::builder_empty()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
        ])
        .build()
}

fn register_tracer_provider(provider: SdkTracerProvider) {
    let storage = OTEL_TRACER_PROVIDER.get_or_init(|| Mutex::new(None));
    let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(old_provider) = guard.replace(provider) {
        let _ = old_provider.shutdown();
    }
}

fn register_meter_provider(provider: SdkMeterProvider) {
    let storage = OTEL_METER_PROVIDER.get_or_init(|| Mutex::new(None));
    let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(old_provider) = guard.replace(provider) {
        let _ = old_provider.shutdown();
    }
}

fn install_metric_exporter_state() {
    let storage = OTEL_METRIC_EXPORTER.get_or_init(|| Mutex::new(None));
    let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(OtlpMetricExporter::new(global::meter(OTEL_METER_NAME)));
}

fn shutdown_tracer_provider() {
    if let Some(storage) = OTEL_TRACER_PROVIDER.get() {
        let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(provider) = guard.take() {
            let _ = provider.shutdown();
        }
    }
}

fn shutdown_meter_provider() {
    if let Some(storage) = OTEL_METER_PROVIDER.get() {
        let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(provider) = guard.take() {
            let _ = provider.shutdown();
        }
    }

    if let Some(storage) = OTEL_METRIC_EXPORTER.get() {
        let mut guard = storage.lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
}

fn resolve_otlp_trace_export_config() -> Option<OtlpExportConfig> {
    resolve_otlp_export_config(
        &[
            "NTK_OTLP_TRACES_ENDPOINT",
            "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
            "NTK_OTLP_ENDPOINT",
            "OTEL_EXPORTER_OTLP_ENDPOINT",
        ],
        &[
            "NTK_OTLP_TRACES_PROTOCOL",
            "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL",
            "NTK_OTLP_PROTOCOL",
            "OTEL_EXPORTER_OTLP_PROTOCOL",
        ],
        &[
            "NTK_OTLP_TRACES_TIMEOUT_MS",
            "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT",
            "NTK_OTLP_TIMEOUT_MS",
            "OTEL_EXPORTER_OTLP_TIMEOUT",
        ],
    )
}

fn resolve_otlp_metric_export_config() -> Option<OtlpExportConfig> {
    resolve_otlp_export_config(
        &[
            "NTK_OTLP_METRICS_ENDPOINT",
            "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
            "NTK_OTLP_ENDPOINT",
            "OTEL_EXPORTER_OTLP_ENDPOINT",
        ],
        &[
            "NTK_OTLP_METRICS_PROTOCOL",
            "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL",
            "NTK_OTLP_PROTOCOL",
            "OTEL_EXPORTER_OTLP_PROTOCOL",
        ],
        &[
            "NTK_OTLP_METRICS_TIMEOUT_MS",
            "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT",
            "NTK_OTLP_TIMEOUT_MS",
            "OTEL_EXPORTER_OTLP_TIMEOUT",
        ],
    )
}

fn resolve_otlp_export_config(
    endpoint_keys: &[&str],
    protocol_keys: &[&str],
    timeout_keys: &[&str],
) -> Option<OtlpExportConfig> {
    let endpoint = first_non_empty_env(endpoint_keys)?;
    let protocol = first_non_empty_env(protocol_keys)
        .as_deref()
        .map(parse_otlp_protocol)
        .unwrap_or(OtlpProtocolKind::Grpc);
    let timeout_ms = first_non_empty_env(timeout_keys)
        .as_deref()
        .map(parse_otlp_timeout_ms)
        .unwrap_or(DEFAULT_OTLP_TIMEOUT_MS);

    Some(OtlpExportConfig {
        endpoint,
        protocol,
        timeout_ms,
    })
}

fn first_non_empty_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn parse_otlp_protocol(value: &str) -> OtlpProtocolKind {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "grpc" => OtlpProtocolKind::Grpc,
        "http" | "http/protobuf" | "http-protobuf" | "httpbinary" => OtlpProtocolKind::HttpBinary,
        _ => {
            warn!(
                protocol = %value,
                "Unsupported OTLP protocol value, defaulting to gRPC"
            );
            OtlpProtocolKind::Grpc
        }
    }
}

fn parse_otlp_timeout_ms(value: &str) -> u64 {
    match value.trim().parse::<u64>() {
        Ok(ms) if ms > 0 => ms,
        _ => {
            warn!(
                timeout = %value,
                default_timeout = DEFAULT_OTLP_TIMEOUT_MS,
                "Invalid OTLP timeout value, using default"
            );
            DEFAULT_OTLP_TIMEOUT_MS
        }
    }
}

fn sanitize_otlp_metric_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::with_capacity(trimmed.len());
    let mut previous_separator = false;

    for raw in trimmed.chars() {
        let ch = raw.to_ascii_lowercase();
        let allowed = ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-';
        if allowed {
            normalized.push(ch);
            previous_separator = false;
        } else if !previous_separator {
            normalized.push('_');
            previous_separator = true;
        }
    }

    let stripped = normalized
        .trim_matches(|ch: char| ch == '_' || ch == '.' || ch == '-')
        .to_string();
    if stripped.is_empty() {
        return None;
    }

    let starts_with_digit = stripped
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit());
    if starts_with_digit {
        Some(format!("ntk_{stripped}"))
    } else {
        Some(stripped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_vars<F>(entries: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        let previous: Vec<(&str, Option<String>)> = entries
            .iter()
            .map(|(key, _)| (*key, std::env::var(key).ok()))
            .collect();

        for (key, value) in entries {
            match value {
                Some(raw) => std::env::set_var(key, raw),
                None => std::env::remove_var(key),
            }
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        for (key, value) in previous {
            if let Some(raw) = value {
                std::env::set_var(key, raw);
            } else {
                std::env::remove_var(key);
            }
        }

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    #[test]
    fn parse_otlp_protocol_supports_grpc_and_http() {
        assert_eq!(parse_otlp_protocol("grpc"), OtlpProtocolKind::Grpc);
        assert_eq!(
            parse_otlp_protocol("http/protobuf"),
            OtlpProtocolKind::HttpBinary
        );
        assert_eq!(parse_otlp_protocol("http"), OtlpProtocolKind::HttpBinary);
    }

    #[test]
    fn parse_otlp_protocol_defaults_to_grpc() {
        assert_eq!(
            parse_otlp_protocol("unknown-protocol"),
            OtlpProtocolKind::Grpc
        );
    }

    #[test]
    fn parse_otlp_protocol_supports_http_aliases() {
        assert_eq!(
            parse_otlp_protocol("httpbinary"),
            OtlpProtocolKind::HttpBinary
        );
        assert_eq!(
            parse_otlp_protocol("http-protobuf"),
            OtlpProtocolKind::HttpBinary
        );
    }

    #[test]
    fn parse_otlp_timeout_uses_default_when_invalid() {
        assert_eq!(parse_otlp_timeout_ms("invalid"), DEFAULT_OTLP_TIMEOUT_MS);
        assert_eq!(parse_otlp_timeout_ms("0"), DEFAULT_OTLP_TIMEOUT_MS);
    }

    #[test]
    fn parse_otlp_timeout_parses_positive_values() {
        assert_eq!(parse_otlp_timeout_ms("15000"), 15_000);
        assert_eq!(parse_otlp_timeout_ms(" 250 "), 250);
    }

    #[test]
    fn default_filter_for_level_maps_known_values() {
        assert_eq!(default_filter_for_level("off", false), "off");
        assert_eq!(
            default_filter_for_level("warn", false),
            "nettoolskit=warn,warn"
        );
        assert_eq!(
            default_filter_for_level("info", false),
            "nettoolskit=info,warn"
        );
        assert!(default_filter_for_level("debug", false).contains("nettoolskit=debug"));
        assert!(default_filter_for_level("trace", false).contains("nettoolskit=trace"));
    }

    #[test]
    fn default_filter_for_level_uses_verbose_fallback_when_unknown() {
        assert!(default_filter_for_level("custom", true).contains("nettoolskit=debug"));
        assert_eq!(
            default_filter_for_level("custom", false),
            "nettoolskit=info,warn"
        );
    }

    #[test]
    fn first_non_empty_env_returns_first_non_blank_entry() {
        with_env_vars(
            &[
                ("NTK_TEST_ENV_A", Some("  ")),
                ("NTK_TEST_ENV_B", Some("value-b")),
            ],
            || {
                let value = first_non_empty_env(&["NTK_TEST_ENV_A", "NTK_TEST_ENV_B"]);
                assert_eq!(value, Some("value-b".to_string()));
            },
        );
    }

    #[test]
    fn resolve_otlp_export_config_returns_none_without_endpoint() {
        with_env_vars(
            &[
                ("NTK_TEST_OTLP_ENDPOINT", None),
                ("NTK_TEST_OTLP_PROTOCOL", Some("http")),
                ("NTK_TEST_OTLP_TIMEOUT", Some("5000")),
            ],
            || {
                let config = resolve_otlp_export_config(
                    &["NTK_TEST_OTLP_ENDPOINT"],
                    &["NTK_TEST_OTLP_PROTOCOL"],
                    &["NTK_TEST_OTLP_TIMEOUT"],
                );
                assert!(config.is_none());
            },
        );
    }

    #[test]
    fn resolve_otlp_export_config_uses_defaults_for_missing_optional_fields() {
        with_env_vars(
            &[
                ("NTK_TEST_OTLP_ENDPOINT", Some("http://localhost:4317")),
                ("NTK_TEST_OTLP_PROTOCOL", None),
                ("NTK_TEST_OTLP_TIMEOUT", None),
            ],
            || {
                let config = resolve_otlp_export_config(
                    &["NTK_TEST_OTLP_ENDPOINT"],
                    &["NTK_TEST_OTLP_PROTOCOL"],
                    &["NTK_TEST_OTLP_TIMEOUT"],
                )
                .expect("endpoint should enable OTLP config");
                assert_eq!(config.endpoint, "http://localhost:4317");
                assert_eq!(config.protocol, OtlpProtocolKind::Grpc);
                assert_eq!(config.timeout_ms, DEFAULT_OTLP_TIMEOUT_MS);
            },
        );
    }

    #[test]
    fn resolve_otlp_trace_export_config_uses_priority_order() {
        with_env_vars(
            &[
                (
                    "NTK_OTLP_TRACES_ENDPOINT",
                    Some("http://localhost:4318/v1/traces"),
                ),
                ("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", Some("http://ignored")),
                ("NTK_OTLP_TRACES_PROTOCOL", Some("http/protobuf")),
                ("OTEL_EXPORTER_OTLP_TRACES_TIMEOUT", Some("9000")),
            ],
            || {
                let config =
                    resolve_otlp_trace_export_config().expect("trace config should resolve");
                assert_eq!(config.endpoint, "http://localhost:4318/v1/traces");
                assert_eq!(config.protocol, OtlpProtocolKind::HttpBinary);
                assert_eq!(config.timeout_ms, 9000);
            },
        );
    }

    #[test]
    fn resolve_otlp_metric_export_config_uses_priority_order() {
        with_env_vars(
            &[
                (
                    "NTK_OTLP_METRICS_ENDPOINT",
                    Some("http://localhost:4318/v1/metrics"),
                ),
                (
                    "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
                    Some("http://ignored"),
                ),
                ("NTK_OTLP_METRICS_PROTOCOL", Some("grpc")),
                ("NTK_OTLP_METRICS_TIMEOUT_MS", Some("7000")),
            ],
            || {
                let config =
                    resolve_otlp_metric_export_config().expect("metric config should resolve");
                assert_eq!(config.endpoint, "http://localhost:4318/v1/metrics");
                assert_eq!(config.protocol, OtlpProtocolKind::Grpc);
                assert_eq!(config.timeout_ms, 7000);
            },
        );
    }

    #[test]
    fn create_env_filter_prefers_rust_log_environment() {
        with_env_vars(&[("RUST_LOG", Some("nettoolskit=trace"))], || {
            let filter = create_env_filter(&TracingConfig::default())
                .expect("RUST_LOG should create a valid filter");
            assert!(format!("{filter}").contains("nettoolskit=trace"));
        });
    }

    #[test]
    fn create_env_filter_uses_default_when_rust_log_not_set() {
        with_env_vars(&[("RUST_LOG", None)], || {
            let config = TracingConfig {
                verbose: false,
                log_level: "warn".to_string(),
                ..Default::default()
            };
            let filter = create_env_filter(&config).expect("default filter should be valid");
            assert!(format!("{filter}").contains("nettoolskit=warn"));
        });
    }

    #[test]
    fn sanitize_otlp_metric_name_normalizes_symbols() {
        assert_eq!(
            sanitize_otlp_metric_name(" runtime/Command Latency "),
            Some("runtime_command_latency".to_string())
        );
        assert_eq!(
            sanitize_otlp_metric_name("1st.metric"),
            Some("ntk_1st.metric".to_string())
        );
    }

    #[test]
    fn sanitize_otlp_metric_name_rejects_empty_inputs() {
        assert_eq!(sanitize_otlp_metric_name(""), None);
        assert_eq!(sanitize_otlp_metric_name("   "), None);
        assert_eq!(sanitize_otlp_metric_name("___"), None);
    }

    #[test]
    fn sanitize_otlp_metric_name_collapses_repeated_separators() {
        assert_eq!(
            sanitize_otlp_metric_name(" latency//////p95 "),
            Some("latency_p95".to_string())
        );
    }

    #[test]
    fn otlp_metric_exporter_records_counter_gauge_and_timing() {
        shutdown_meter_provider();
        install_metric_exporter_state();

        record_otlp_counter("runtime/command count", 1);
        record_otlp_gauge("runtime/queue depth", 3.0);
        record_otlp_timing("runtime/latency", Duration::from_millis(15));
        record_otlp_gauge("runtime/invalid", f64::NAN);

        let storage = OTEL_METRIC_EXPORTER
            .get()
            .expect("metric exporter storage should exist");
        let guard = storage.lock().unwrap_or_else(|error| error.into_inner());
        let exporter = guard.as_ref().expect("metric exporter should be installed");
        assert!(!exporter.counters.is_empty());
        assert!(!exporter.gauges.is_empty());
        assert!(!exporter.timings_ms.is_empty());

        drop(guard);
        shutdown_meter_provider();
    }

    #[test]
    fn shutdown_meter_provider_clears_exporter_state() {
        install_metric_exporter_state();
        shutdown_meter_provider();
        let storage = OTEL_METRIC_EXPORTER
            .get()
            .expect("metric exporter storage should exist");
        let guard = storage.lock().unwrap_or_else(|error| error.into_inner());
        assert!(guard.is_none());
    }

    #[test]
    fn build_resource_includes_service_identity_attributes() {
        let config = TracingConfig {
            service_name: "ntk-tests".to_string(),
            service_version: "9.9.9".to_string(),
            ..Default::default()
        };
        let resource = build_resource(&config);
        let attrs: Vec<(String, String)> = resource
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        assert!(attrs
            .iter()
            .any(|(key, value)| key == "service.name" && value.contains("ntk-tests")));
        assert!(attrs
            .iter()
            .any(|(key, value)| key == "service.version" && value.contains("9.9.9")));
    }
}
