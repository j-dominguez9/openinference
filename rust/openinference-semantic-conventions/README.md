# openinference-semantic-conventions (Rust)

Attribute keys and value constants for OpenInference span kinds, messages, tools, embeddings, retrieval, cost/token counts, and prompts — the Rust port of [`openinference-semantic-conventions`](../../python/openinference-semantic-conventions) (Python) / `openinference-semantic-conventions` (Go / Java / JS).

Keys are exported as `&'static str` constants so they work directly with [`opentelemetry::KeyValue`](https://docs.rs/opentelemetry/latest/opentelemetry/struct.KeyValue.html). The crate has no dependencies.

## Install

```bash
cargo add openinference-semantic-conventions
```

Requires Rust 1.85+ (edition 2024).

## Use

The crate has three modules:

- `attributes`: attribute keys, e.g. `LLM_MODEL_NAME` → `"llm.model_name"`
- `enums`: well-known values, e.g. `SpanKind::Llm.as_str()` → `"LLM"`
- `indexers`: builders for indexed keys, e.g. `llm_input_message_role_key(0)` → `"llm.input_messages.0.message.role"`

```rust
use opentelemetry::KeyValue;
use openinference_semantic_conventions::attributes::{
    LLM_MODEL_NAME, LLM_PROVIDER, OPENINFERENCE_SPAN_KIND,
};
use openinference_semantic_conventions::enums::{LlmProvider, SpanKind};

span.set_attributes([
    KeyValue::new(OPENINFERENCE_SPAN_KIND, SpanKind::Llm.as_str()),
    KeyValue::new(LLM_MODEL_NAME, "gpt-4o"),
    KeyValue::new(LLM_PROVIDER, LlmProvider::Openai.as_str()),
]);
```

The enums cover the well-known values only. The spec allows other values for attributes such as `llm.provider`, so pass a plain string when a vendor isn't listed.

Indexed attributes (array-valued, like message lists) are produced by helper functions that return a `String`:

```rust
use opentelemetry::KeyValue;
use openinference_semantic_conventions::attributes::LLM_TOKEN_COUNT_PROMPT;
use openinference_semantic_conventions::indexers::{
    llm_input_message_content_key, llm_input_message_role_key,
};

span.set_attributes([
    KeyValue::new(llm_input_message_role_key(0), "system"),
    KeyValue::new(llm_input_message_content_key(0), "You are a helpful assistant."),
    KeyValue::new(llm_input_message_role_key(1), "user"),
    KeyValue::new(llm_input_message_content_key(1), user_input),
    KeyValue::new(LLM_TOKEN_COUNT_PROMPT, 42),
]);
```

## Send the trace to Arize AX or Phoenix

This crate is exporter-agnostic — pair it with whatever OTel exporter your runtime needs (e.g. [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp)):

- **Arize AX**: configure an OTLP/HTTP exporter pointed at `otlp.arize.com` with `space_id` + `api_key` headers.
- **Phoenix (self-hosted)**: configure OTLP/HTTP at `http://localhost:6006/v1/traces` (or your Phoenix host).

## Drift policy

The Python package is the source of truth. New attribute keys MUST be added to `python/openinference-semantic-conventions/` first, then ported here. Two layers catch drift in CI:

- **`scripts/check_rust_semconv.py`** parses the Python source and asserts every wire-format key value is present in the Rust crate. Canonical cross-language check.
- **Unit tests** in `src/enums.rs` and `src/indexers.rs` assert each value and helper against its wire-format string. Cheaper Rust-side typo guard.
