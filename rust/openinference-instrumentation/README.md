# openinference-instrumentation (Rust)

Core building blocks for OpenInference instrumentors in Rust: a tracer wrapper that masks sensitive span attributes, tracing suppression, and context attribute propagation (session, user, metadata, tags, prompt template). Mirrors the Python [`openinference-instrumentation`](../../python/openinference-instrumentation) package.

## Install

```bash
cargo add openinference-instrumentation openinference-semantic-conventions opentelemetry
```

Requires Rust 1.85+ (edition 2024).

## Wrap your tracer

`OITracer` wraps any OpenTelemetry tracer. Every span it starts records the current context attributes, and every attribute set on it is masked by a `TraceConfig` before it reaches the span, including the attributes passed when the span starts, so samplers never see hidden values.

```rust
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use openinference_instrumentation::trace_config::TraceConfig;
use openinference_instrumentation::tracer::OITracer;
use openinference_semantic_conventions::attributes::INPUT_VALUE;

let tracer = OITracer::new(global::tracer("my-app"), TraceConfig::from_env());

let mut span = tracer.start("chat");
span.set_attribute(KeyValue::new(INPUT_VALUE, "What is the weather?"));
span.end();
```

`OITracer` implements `opentelemetry::trace::Tracer`, so `start`, `start_with_context`, `in_span` and span builders all work as usual.

## Suppression

Skip tracing for work that shouldn't appear in the trace, such as evaluator code that itself calls an LLM:

```rust
use openinference_instrumentation::suppression::suppress_tracing;

{
    let _guard = suppress_tracing();
    // spans started here are not recorded
}
// tracing resumes when the guard is dropped
```

Bind the guard with `let _guard`; `let _ =` drops it immediately. The guard applies only to the current thread, so in async code pass a suppressed `Context` along instead. Instrumentors can check `is_tracing_suppressed(&cx)` to skip work entirely.

## Context attributes

OTel span attributes are not inherited from parent to child, so setting `session.id` on one span does not put it on the LLM spans beneath it. These helpers store the values on the OTel `Context`, and `OITracer` adds them to every span started in that context.

The `with_*` functions return a new `Context`, which works across threads and in async code:

```rust
use opentelemetry::Context;
use openinference_instrumentation::context_attributes::{
    with_metadata, with_session, with_tags, with_user,
};

let cx = with_session(&Context::current(), "session-abc");
let cx = with_user(&cx, "user-xyz");
let cx = with_metadata(&cx, r#"{"team":"platform"}"#);
let cx = with_tags(&cx, ["prod", "canary"]);

let span = tracer.start_with_context("chat", &cx);
// span carries session.id, user.id, metadata and tag.tags
```

The `using_*` functions attach the values to the current thread until the returned guard is dropped, like Python's `using_*` context managers:

```rust
use openinference_instrumentation::context_attributes::{using_prompt_template, using_session};

let _session = using_session("session-abc");
let _template = using_prompt_template(
    "Describe the weather in {city}",
    "v1.0",
    r#"{"city":"Johannesburg"}"#,
);
let span = tracer.start("chat");
```

Metadata and prompt template variables are JSON object strings. Empty values are skipped. For spans you create without `OITracer`, `context_attributes(&cx)` returns the attributes as `KeyValue`s.

## Masking sensitive data

`TraceConfig` controls which OpenInference attributes are recorded. Set it with environment variables, in code, or both:

```bash
export OPENINFERENCE_HIDE_INPUTS=true
export OPENINFERENCE_HIDE_OUTPUTS=true
```

```rust
use openinference_instrumentation::trace_config::TraceConfig;

// Code takes precedence; anything not set falls back to the environment.
let config = TraceConfig {
    hide_inputs: true,
    ..TraceConfig::from_env()
};
```

Environment variables (booleans accept `true` or `false`, in any case):

| Env var | What it does |
|---|---|
| `OPENINFERENCE_HIDE_INPUTS` | Redacts `input.value`, `llm.prompts` and the reranker query; drops `input.mime_type`, `llm.input_messages.*`, `llm.tools.*`, input images and reranker input documents. |
| `OPENINFERENCE_HIDE_OUTPUTS` | Redacts `output.value`, `llm.choices` and retrieval document content and metadata; drops `output.mime_type`, `llm.output_messages.*`, output images and reranker output documents. |
| `OPENINFERENCE_HIDE_INPUT_MESSAGES` | Drops `llm.input_messages.*`. |
| `OPENINFERENCE_HIDE_OUTPUT_MESSAGES` | Drops `llm.output_messages.*`. |
| `OPENINFERENCE_HIDE_INPUT_TEXT` / `_OUTPUT_TEXT` | Keeps message structure (roles, indices) and redacts only the message text. |
| `OPENINFERENCE_HIDE_INPUT_IMAGES` | Drops images in input messages. |
| `OPENINFERENCE_HIDE_PROMPTS` | Redacts `llm.prompts`. |
| `OPENINFERENCE_HIDE_CHOICES` | Redacts `llm.choices`. |
| `OPENINFERENCE_HIDE_RETRIEVAL_DOCUMENTS` | Redacts retrieval document content and metadata. |
| `OPENINFERENCE_HIDE_LLM_INVOCATION_PARAMETERS` | Drops `llm.invocation_parameters`. |
| `OPENINFERENCE_HIDE_LLM_TOOLS` | Drops `llm.tools.*`. |
| `OPENINFERENCE_HIDE_EMBEDDINGS_VECTORS` | Redacts embedding vectors. |
| `OPENINFERENCE_HIDE_EMBEDDINGS_TEXT` | Redacts embedding text. |
| `OPENINFERENCE_BASE64_IMAGE_MAX_LENGTH` | Redacts base64 image URLs longer than this many characters (default `32000`). |

Redacted values are replaced with `__REDACTED__`, so consumers can tell "hidden" from "never recorded"; dropped attributes don't appear on the span at all. The rules match Python's `TraceConfig.mask`, in the same order.

## Differences from Python

- Invalid environment values fall back to the default without a warning.
- `OPENINFERENCE_BLOB_UPLOADER` is not supported; oversized base64 images are always redacted.
- `OPENINFERENCE_ENABLE_GENAI_SEMCONV` is read but `gen_ai.*` attributes are not written yet.
- There is no decorator API (`@tracer.chain`, `@tracer.tool`, ...).
- Metadata and prompt template variables are taken as JSON strings (as in Go) and are not validated.

## Send the trace to Arize AX or Phoenix

This crate is exporter-agnostic. Pair it with whatever OTel exporter your runtime needs (e.g. [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp)):

- **Arize AX**: configure an OTLP/HTTP exporter pointed at `otlp.arize.com` with `space_id` + `api_key` headers.
- **Phoenix (self-hosted)**: configure OTLP/HTTP at `http://localhost:6006/v1/traces` (or your Phoenix host).
