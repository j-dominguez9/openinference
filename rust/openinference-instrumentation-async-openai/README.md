# openinference-instrumentation-async-openai (Rust)

OpenInference instrumentation for [`async-openai`](https://crates.io/crates/async-openai). Chat completions, streaming or not, are recorded as OpenInference LLM spans with messages, tool calls, token counts and invocation parameters.

The instrumentation is a [`tower`](https://crates.io/crates/tower) layer installed with `async-openai`'s `Client::with_http_service`, so it sees each HTTP request and response without changing how you call the client.

## Install

```bash
cargo add openinference-instrumentation-async-openai openinference-instrumentation opentelemetry tower reqwest
cargo add async-openai --features middleware,chat-completion
```

Requires Rust 1.85+ (edition 2024) and `async-openai` 0.42+ with its `middleware` feature.

## Use

Wrap your tracer in `OITracer`, add `OpenInferenceLayer` to the client's HTTP service, and use the client as usual:

```rust
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::middleware::ReqwestService;
use opentelemetry::global;
use openinference_instrumentation::trace_config::TraceConfig;
use openinference_instrumentation::tracer::OITracer;
use openinference_instrumentation_async_openai::OpenInferenceLayer;

let tracer = OITracer::new(global::tracer("my-app"), TraceConfig::from_env());
let service = tower::ServiceBuilder::new()
    .layer(OpenInferenceLayer::new(tracer))
    .service(ReqwestService::new(reqwest::Client::new()));
let client = Client::with_config(OpenAIConfig::default()).with_http_service(service);

// client.chat().create(...) and client.chat().create_stream(...) are now traced.
```

`ReqwestService` sends requests without `async-openai`'s default retries. To keep retries, add `async_openai::middleware::retry::OpenAIRetryLayer` below `OpenInferenceLayer`; each attempt then shares one span.

### Context attributes

Session, user, metadata and tag attributes from [`openinference-instrumentation`](../openinference-instrumentation) are added to every span. In async code, attach them to the call's future:

```rust
use opentelemetry::Context;
use opentelemetry::context::FutureExt;
use openinference_instrumentation::context_attributes::{with_session, with_user};

let cx = with_user(&with_session(&Context::current(), "session-abc"), "user-xyz");
let response = client.chat().create(request).with_context(cx).await?;
```

### Masking and suppression

Masking follows the `TraceConfig` passed to `OITracer`, including the `OPENINFERENCE_HIDE_*` environment variables, and requests made while tracing is suppressed create no span. See the [`openinference-instrumentation` README](../openinference-instrumentation/README.md).

## What is recorded

Spans are named `openai.chat.completions.create` and carry:

- `openinference.span.kind` = `LLM`, `llm.system` = `openai`, and `llm.provider` = `openai` (or `azure` for Azure OpenAI hosts)
- `llm.model_name`: the model the response reports, or the requested one if the request failed
- `llm.invocation_parameters`: every request field except `messages`, `functions` and `tools`
- `input.value` / `output.value`: the request and response bodies as JSON
- `llm.input_messages.*` / `llm.output_messages.*`: roles, text, content parts (text and images), names, tool calls and tool call IDs
- `llm.tools.*`: the tool definitions sent with the request
- `llm.token_count.*`: prompt, completion and total, plus cached, reasoning and audio details
- `llm.finish_reason`, and an error status for failed requests

Streaming responses are passed through as they arrive and the span ends when the stream finishes, with the output reassembled from the deltas. Set `stream_options.include_usage` to record token counts for streams.

Other endpoints (embeddings, the Responses API, ...) pass through untraced for now.

## Example

[`examples/chat.rs`](examples/chat.rs) sends one chat completion and exports the span to Arize AX:

```bash
cd rust
OPENAI_API_KEY=... ARIZE_SPACE_ID=... ARIZE_API_KEY=... \
  cargo run -p openinference-instrumentation-async-openai --example chat
```

## Send the trace to Arize AX or Phoenix

Pair the tracer with whatever OTel exporter your runtime needs (e.g. [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp)):

- **Arize AX**: configure an OTLP/HTTP exporter pointed at `https://otlp.arize.com/v1/traces` with `space_id` + `api_key` headers.
- **Phoenix (self-hosted)**: configure OTLP/HTTP at `http://localhost:6006/v1/traces` (or your Phoenix host).

`opentelemetry-otlp` needs a TLS feature such as `reqwest-rustls` to reach an `https://` endpoint, and reports export failures only through its internal logs, so install a `tracing` subscriber to see them.
