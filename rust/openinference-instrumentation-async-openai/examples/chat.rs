//! Sends one real chat completion through `OpenInferenceLayer` and exports the
//! span to Arize AX.
//!
//! Run from `rust/`:
//!
//! ```bash
//! OPENAI_API_KEY=... ARIZE_SPACE_ID=... ARIZE_API_KEY=... \
//!   cargo run -p openinference-instrumentation-async-openai --example chat
//! ```
//!
//! Add `-- --stream` to stream the response, or `-- --tools` to run a
//! tool-calling cycle (model → tool → model) under an agent span. Set
//! `OPENAI_MODEL` to use a model other than `gpt-6-luna`.

use std::collections::HashMap;
use std::env;
use std::future::poll_fn;
use std::pin::Pin;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::middleware::ReqwestService;
use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage, ChatCompletionStreamOptions, ChatCompletionTool,
    ChatCompletionTools, CreateChatCompletionRequestArgs, FunctionObject, ReasoningEffort,
};
use futures_core::Stream;
use openinference_instrumentation::context_attributes::{with_session, with_tags};
use openinference_instrumentation::trace_config::TraceConfig;
use openinference_instrumentation::tracer::OITracer;
use openinference_instrumentation_async_openai::OpenInferenceLayer;
use openinference_semantic_conventions::attributes::{
    INPUT_VALUE, OPENINFERENCE_SPAN_KIND, OUTPUT_VALUE, TOOL_NAME,
};
use openinference_semantic_conventions::enums::SpanKind;
use opentelemetry::context::FutureExt;
use opentelemetry::trace::TracerProvider;
use opentelemetry::trace::{Span, Status, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use serde_json::json;

const PROJECT_NAME: &str = "openinference-rust-async-openai-example";

fn main() {
    // The OTel SDK reports export failures (e.g. a rejected API key) only
    // through its internal logs, so print them.
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
        .init();

    // The OTLP exporter uses a blocking HTTP client, which can't be created or
    // shut down inside the async runtime, so it lives outside it.
    let provider = tracer_provider();
    let tracer = OITracer::new(
        provider.tracer("async-openai-example"),
        TraceConfig::from_env(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("failed to start the async runtime");
    if env::args().any(|arg| arg == "--tools") {
        // A second tracer for the spans the example creates itself.
        let agent_tracer = OITracer::new(
            provider.tracer("async-openai-example"),
            TraceConfig::from_env(),
        );
        runtime.block_on(weather_agent(tracer, agent_tracer));
    } else {
        let stream = env::args().any(|arg| arg == "--stream");
        runtime.block_on(chat(tracer, stream));
    }
    drop(runtime);

    // Flushes the batch exporter before the process exits.
    if let Err(err) = provider.shutdown() {
        eprintln!("failed to export spans: {err}");
    }
}

fn traced_client(tracer: OITracer<SdkTracer>) -> Client<OpenAIConfig> {
    let service = tower::ServiceBuilder::new()
        .layer(OpenInferenceLayer::new(tracer))
        .service(ReqwestService::new(reqwest::Client::new()));
    Client::with_config(OpenAIConfig::default()).with_http_service(service)
}

fn model() -> String {
    env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-6-luna".to_string())
}

async fn chat(tracer: OITracer<SdkTracer>, stream: bool) {
    let client = traced_client(tracer);
    let model = model();
    let mut request = CreateChatCompletionRequestArgs::default();
    request
        .model(model)
        .messages([ChatCompletionRequestUserMessage::from(
            "In one sentence, what is observability?",
        )
        .into()]);
    if stream {
        // Ask for a final chunk with token counts.
        request
            .stream(true)
            .stream_options(ChatCompletionStreamOptions {
                include_usage: Some(true),
                include_obfuscation: None,
            });
    }
    let request = request.build().expect("valid request");

    // Context attributes travel with the future, so they work across `.await`.
    let session = if stream {
        "rust-openai-stream-session"
    } else {
        "rust-openai-session"
    };
    let cx = with_tags(&with_session(&Context::new(), session), ["rust", "openai"]);

    if stream {
        let mut chunks = match client
            .chat()
            .create_stream(request)
            .with_context(cx.clone())
            .await
        {
            Ok(chunks) => chunks,
            Err(err) => return eprintln!("OpenAI request failed: {err}"),
        };
        while let Some(chunk) = poll_fn(|task| Pin::new(&mut chunks).poll_next(task)).await {
            match chunk {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        print!("{}", choice.delta.content.as_deref().unwrap_or_default());
                    }
                }
                Err(err) => return eprintln!("\nstream failed: {err}"),
            }
        }
        println!();
        return;
    }

    match client.chat().create(request).with_context(cx).await {
        Ok(response) => {
            let text = response.choices[0]
                .message
                .content
                .as_deref()
                .unwrap_or_default();
            println!("{text}");
        }
        Err(err) => eprintln!("OpenAI request failed: {err}"),
    }
}

/// Runs one tool-calling cycle under an AGENT span: the model asks for the
/// weather, the example runs `get_weather` in a TOOL span, and the model
/// answers with the result. Both LLM calls are traced by the layer.
async fn weather_agent(tracer: OITracer<SdkTracer>, agent_tracer: OITracer<SdkTracer>) {
    const QUESTION: &str = "What's the weather in Paris? Use the tool.";
    let client = traced_client(tracer);

    let cx = with_tags(
        &with_session(&Context::new(), "rust-openai-tools-session"),
        ["rust", "openai", "tools"],
    );
    let mut agent = agent_tracer.start_with_context("weather-agent", &cx);
    agent.set_attributes([
        KeyValue::new(OPENINFERENCE_SPAN_KIND, SpanKind::Agent.as_str()),
        KeyValue::new(INPUT_VALUE, QUESTION),
    ]);
    // Spans started under `cx` become children of the agent span.
    let cx = cx.with_span(agent);

    match run_weather_agent(&client, &agent_tracer, QUESTION)
        .with_context(cx.clone())
        .await
    {
        Ok(answer) => {
            println!("{answer}");
            cx.span().set_attribute(KeyValue::new(OUTPUT_VALUE, answer));
            cx.span().set_status(Status::Ok);
        }
        Err(err) => {
            eprintln!("OpenAI request failed: {err}");
            cx.span().set_status(Status::error(err.to_string()));
        }
    }
    cx.span().end();
}

async fn run_weather_agent(
    client: &Client<OpenAIConfig>,
    tracer: &OITracer<SdkTracer>,
    question: &str,
) -> Result<String, OpenAIError> {
    let tools = vec![ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: "get_weather".into(),
            description: Some("Returns the current weather for a city.".into()),
            parameters: Some(json!({
                "type": "object",
                "properties": {"city": {"type": "string"}},
                "required": ["city"],
            })),
            strict: None,
        },
    })];
    let mut messages: Vec<ChatCompletionRequestMessage> =
        vec![ChatCompletionRequestUserMessage::from(question).into()];

    // 1. The model decides to call the tool.
    let request = CreateChatCompletionRequestArgs::default()
        .model(model())
        .messages(messages.clone())
        .tools(tools.clone())
        // Chat completions only allows function tools without reasoning.
        .reasoning_effort(ReasoningEffort::None)
        .build()?;
    let first = client.chat().create(request).await?;
    let message = &first.choices[0].message;
    let Some(tool_calls) = message.tool_calls.clone() else {
        return Ok(message.content.clone().unwrap_or_default());
    };

    // 2. Run each tool call and send the results back.
    messages.push(
        ChatCompletionRequestAssistantMessageArgs::default()
            .tool_calls(tool_calls.clone())
            .build()?
            .into(),
    );
    for call in tool_calls {
        if let ChatCompletionMessageToolCalls::Function(call) = call {
            let result = traced_get_weather(tracer, &call.function.arguments);
            messages.push(
                ChatCompletionRequestToolMessage {
                    content: result.into(),
                    tool_call_id: call.id,
                }
                .into(),
            );
        }
    }

    // 3. The model answers using the tool results.
    let request = CreateChatCompletionRequestArgs::default()
        .model(model())
        .messages(messages)
        .tools(tools)
        .reasoning_effort(ReasoningEffort::None)
        .build()?;
    let second = client.chat().create(request).await?;
    Ok(second.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default())
}

/// A stand-in weather tool, traced as a TOOL span.
fn traced_get_weather(tracer: &OITracer<SdkTracer>, arguments: &str) -> String {
    let mut span = tracer.start("get_weather");
    let city = serde_json::from_str::<serde_json::Value>(arguments)
        .ok()
        .and_then(|args| args["city"].as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string());
    let result = json!({"city": city, "forecast": "sunny", "temperature_c": 21}).to_string();
    span.set_attributes([
        KeyValue::new(OPENINFERENCE_SPAN_KIND, SpanKind::Tool.as_str()),
        KeyValue::new(TOOL_NAME, "get_weather"),
        KeyValue::new(INPUT_VALUE, arguments.to_string()),
        KeyValue::new(OUTPUT_VALUE, result.clone()),
    ]);
    span.end();
    result
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
