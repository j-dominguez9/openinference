//! Sends one hand-built LLM span to Arize AX through `OITracer`, with context
//! attributes and input masking applied. No LLM API key is needed.
//!
//! Run from `rust/`:
//!
//! ```bash
//! ARIZE_SPACE_ID=... ARIZE_API_KEY=... cargo run --example basic
//! ```

use std::collections::HashMap;
use std::env;

use openinference_instrumentation::context_attributes::{using_session, using_tags};
use openinference_instrumentation::trace_config::TraceConfig;
use openinference_instrumentation::tracer::OITracer;
use openinference_semantic_conventions::attributes::{
    INPUT_MIME_TYPE, INPUT_VALUE, LLM_MODEL_NAME, LLM_PROVIDER, LLM_TOKEN_COUNT_COMPLETION,
    LLM_TOKEN_COUNT_PROMPT, OPENINFERENCE_SPAN_KIND, OUTPUT_VALUE,
};
use openinference_semantic_conventions::enums::{LlmProvider, SpanKind};
use opentelemetry::KeyValue;
use opentelemetry::trace::{Span, Tracer, TracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;

const PROJECT_NAME: &str = "openinference-rust-basic-example";

fn main() {
    // The OTel SDK reports export failures (e.g. a rejected API key) only
    // through its internal logs, so print them.
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    let provider = tracer_provider();
    let config = TraceConfig {
        hide_inputs: true,
        ..TraceConfig::from_env()
    };
    let tracer = OITracer::new(provider.tracer("basic-example"), config);

    {
        let _session = using_session("rust-example-session");
        let _tags = using_tags(["rust", "example"]);

        let mut span = tracer.start("chat");
        span.set_attributes([
            KeyValue::new(OPENINFERENCE_SPAN_KIND, SpanKind::Llm.as_str()),
            KeyValue::new(LLM_PROVIDER, LlmProvider::Openai.as_str()),
            KeyValue::new(LLM_MODEL_NAME, "gpt-4o"),
            // Hidden by `hide_inputs`: the value is redacted, the MIME type dropped.
            KeyValue::new(INPUT_VALUE, "What is observability?"),
            KeyValue::new(INPUT_MIME_TYPE, "text/plain"),
            KeyValue::new(OUTPUT_VALUE, "Understanding a system from its outputs."),
            KeyValue::new(LLM_TOKEN_COUNT_PROMPT, 5),
            KeyValue::new(LLM_TOKEN_COUNT_COMPLETION, 7),
        ]);
        span.end();
    }

    // Flushes the batch exporter before the process exits.
    if let Err(err) = provider.shutdown() {
        eprintln!("failed to export spans: {err}");
    }
}

fn tracer_provider() -> SdkTracerProvider {
    let headers = HashMap::from([
        ("space_id".to_string(), required_env("ARIZE_SPACE_ID")),
        ("api_key".to_string(), required_env("ARIZE_API_KEY")),
    ]);
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint("https://otlp.arize.com/v1/traces")
        .with_headers(headers)
        .build()
        .expect("failed to build the OTLP exporter");
    let resource = Resource::builder()
        .with_service_name(PROJECT_NAME)
        .with_attribute(KeyValue::new("openinference.project.name", PROJECT_NAME))
        .build();

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("environment variable {name} is required"))
}
