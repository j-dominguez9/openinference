use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use async_openai::error::OpenAIError;
use async_openai::middleware::HttpRequestFactory;
use openinference_instrumentation::suppression::is_tracing_suppressed;
use openinference_instrumentation::tracer::OITracer;
use openinference_semantic_conventions::attributes::{
    LLM_MODEL_NAME, LLM_PROVIDER, LLM_SYSTEM, OPENINFERENCE_SPAN_KIND,
};
use openinference_semantic_conventions::enums::{LlmProvider, LlmSystem, SpanKind};
use opentelemetry::context::FutureExt;
use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use serde_json::Value;
use tower::{Layer, Service};

use crate::request::request_attributes;
use crate::response::response_attributes;
use crate::stream::TracedStream;

const CHAT_COMPLETIONS_SPAN_NAME: &str = "openai.chat.completions.create";

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// A [`tower::Layer`] that traces async-openai chat completions.
///
/// Install it with `Client::with_http_service`. Each chat completions
/// request, streaming or not, becomes an OpenInference LLM span; other
/// requests pass through untraced.
///
/// # Examples
///
/// ```no_run
/// use async_openai::Client;
/// use async_openai::config::OpenAIConfig;
/// use async_openai::middleware::ReqwestService;
/// use opentelemetry::global;
/// use openinference_instrumentation::trace_config::TraceConfig;
/// use openinference_instrumentation::tracer::OITracer;
/// use openinference_instrumentation_async_openai::OpenInferenceLayer;
///
/// let tracer = OITracer::new(global::tracer("my-app"), TraceConfig::from_env());
/// let service = tower::ServiceBuilder::new()
///     .layer(OpenInferenceLayer::new(tracer))
///     .service(ReqwestService::new(reqwest::Client::new()));
/// let client = Client::with_config(OpenAIConfig::default()).with_http_service(service);
/// ```
#[derive(Debug)]
pub struct OpenInferenceLayer<T> {
    tracer: Arc<OITracer<T>>,
}

impl<T> OpenInferenceLayer<T> {
    /// Traces requests with `tracer`.
    pub fn new(tracer: OITracer<T>) -> Self {
        Self {
            tracer: Arc::new(tracer),
        }
    }
}

// Written by hand: `derive(Clone)` would require `T: Clone`, but only the
// `Arc` is cloned.
impl<T> Clone for OpenInferenceLayer<T> {
    fn clone(&self) -> Self {
        Self {
            tracer: Arc::clone(&self.tracer),
        }
    }
}

impl<S, T> Layer<S> for OpenInferenceLayer<T> {
    type Service = OpenInferenceService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        OpenInferenceService {
            inner,
            tracer: Arc::clone(&self.tracer),
        }
    }
}

/// The service created by [`OpenInferenceLayer`].
#[derive(Debug)]
pub struct OpenInferenceService<S, T> {
    inner: S,
    tracer: Arc<OITracer<T>>,
}

impl<S: Clone, T> Clone for OpenInferenceService<S, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            tracer: Arc::clone(&self.tracer),
        }
    }
}

impl<S, T> Service<HttpRequestFactory> for OpenInferenceService<S, T>
where
    S: Service<HttpRequestFactory, Response = reqwest::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<OpenAIError>,
    T: Tracer + Send + Sync + 'static,
    T::Span: Send + Sync + 'static,
{
    type Response = reqwest::Response;
    type Error = OpenAIError;
    type Future = BoxFuture<Result<reqwest::Response, OpenAIError>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), OpenAIError>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, factory: HttpRequestFactory) -> Self::Future {
        // Tower requires calling the service that `poll_ready` readied, so
        // take it and leave a fresh clone in its place.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let tracer = Arc::clone(&self.tracer);
        // Read now: `call` runs while the caller's context is current.
        let parent_cx = Context::current();

        Box::pin(async move {
            let Some((mut attributes, streaming)) = start_attributes(&factory, &parent_cx).await
            else {
                return inner.call(factory).await.map_err(Into::into);
            };
            // The SDK keeps every value set for a key, so record the model
            // once, when the span ends.
            let requested_model = attributes
                .iter()
                .position(|kv| kv.key.as_str() == LLM_MODEL_NAME)
                .map(|i| attributes.remove(i));
            let span = tracer
                .span_builder(CHAT_COMPLETIONS_SPAN_NAME)
                .with_attributes(attributes)
                .start_with_context(tracer.as_ref(), &parent_cx);
            let mut span = LlmSpan::new(parent_cx.with_span(span), requested_model);

            // Make the LLM span current while the request runs, so inner
            // layers and HTTP tracing see it as their parent.
            let call = {
                let _guard = span.context().clone().attach();
                inner.call(factory)
            };
            let response = match call
                .with_context(span.context().clone())
                .await
                .map_err(Into::into)
            {
                Ok(response) => response,
                Err(err) => {
                    span.finish(Vec::new(), Status::error(err.to_string()));
                    return Err(err);
                }
            };

            let status = response.status();
            let version = response.version();
            let headers = response.headers().clone();
            let rebuild = |body: reqwest::Body| {
                let mut rebuilt = http::Response::new(body);
                *rebuilt.status_mut() = status;
                *rebuilt.version_mut() = version;
                *rebuilt.headers_mut() = headers;
                reqwest::Response::from(rebuilt)
            };

            // Pass a stream through as it arrives; the span ends with it.
            if streaming && status.is_success() {
                let stream = TracedStream::new(Box::pin(response.bytes_stream()), span);
                return Ok(rebuild(reqwest::Body::wrap_stream(stream)));
            }

            // Read the body to record it, then hand async-openai an
            // identical response.
            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    span.finish(Vec::new(), Status::error(err.to_string()));
                    return Err(OpenAIError::Reqwest(err));
                }
            };

            if status.is_success() {
                let attributes = serde_json::from_slice::<Value>(&bytes)
                    .map(|body| response_attributes(&body))
                    .unwrap_or_default();
                span.finish(attributes, Status::Ok);
            } else {
                span.finish(Vec::new(), Status::error(format!("HTTP {status}")));
            }

            Ok(rebuild(reqwest::Body::from(bytes)))
        })
    }
}

/// An LLM span in progress. If it's dropped before [`LlmSpan::finish`], for
/// example because an outer timeout cancelled the request, it ends as
/// cancelled.
pub(crate) struct LlmSpan {
    // The caller's context with the LLM span as its active span.
    cx: Context,
    requested_model: Option<KeyValue>,
    ended: bool,
}

impl LlmSpan {
    fn new(cx: Context, requested_model: Option<KeyValue>) -> Self {
        Self {
            cx,
            requested_model,
            ended: false,
        }
    }

    fn context(&self) -> &Context {
        &self.cx
    }

    /// Sets the final attributes and status, then ends the span. The
    /// requested model is recorded only if the response didn't name one.
    /// Later calls do nothing.
    pub(crate) fn finish(&mut self, mut attributes: Vec<KeyValue>, status: Status) {
        if self.ended {
            return;
        }
        self.ended = true;
        if !attributes
            .iter()
            .any(|kv| kv.key.as_str() == LLM_MODEL_NAME)
        {
            attributes.extend(self.requested_model.take());
        }
        let span = self.cx.span();
        span.set_attributes(attributes);
        span.set_status(status);
        span.end();
    }
}

impl Drop for LlmSpan {
    fn drop(&mut self) {
        self.finish(Vec::new(), Status::error("request cancelled"));
    }
}

/// Returns the attributes to start a span with and whether the response
/// streams, or `None` if the request shouldn't be traced.
async fn start_attributes(
    factory: &HttpRequestFactory,
    cx: &Context,
) -> Option<(Vec<KeyValue>, bool)> {
    if is_tracing_suppressed(cx) {
        return None;
    }
    // Build a copy to inspect; the real request is rebuilt from the factory.
    let request = factory.build().await.ok()?;
    if request.method() != reqwest::Method::POST
        || !request.url().path().ends_with("/chat/completions")
    {
        return None;
    }
    let body: Value = serde_json::from_slice(request.body()?.as_bytes()?).ok()?;
    let streaming = body["stream"].as_bool() == Some(true);

    let provider = provider_for_host(request.url().host_str().unwrap_or_default());
    let mut attributes = vec![
        KeyValue::new(OPENINFERENCE_SPAN_KIND, SpanKind::Llm.as_str()),
        KeyValue::new(LLM_SYSTEM, LlmSystem::Openai.as_str()),
        KeyValue::new(LLM_PROVIDER, provider.as_str()),
    ];
    attributes.extend(request_attributes(&body));
    Some((attributes, streaming))
}

/// Azure OpenAI hosts report `azure`; every other host reports `openai`.
fn provider_for_host(host: &str) -> LlmProvider {
    let host = host.to_ascii_lowercase();
    let azure_suffixes = [
        ".openai.azure.com",
        ".services.ai.azure.com",
        ".cognitiveservices.azure.com",
    ];
    if azure_suffixes.iter().any(|suffix| host.ends_with(suffix)) {
        LlmProvider::Azure
    } else {
        LlmProvider::Openai
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_openai::Client;
    use async_openai::config::OpenAIConfig;
    use async_openai::types::chat::{
        ChatCompletionRequestUserMessage, ChatCompletionTool, ChatCompletionTools,
        CreateChatCompletionRequestArgs, FunctionObject,
    };
    use openinference_instrumentation::context_attributes::with_session;
    use openinference_instrumentation::suppression::suppress_tracing;
    use openinference_instrumentation::trace_config::{REDACTED_VALUE, TraceConfig};
    use openinference_semantic_conventions::attributes::{
        INPUT_VALUE, LLM_FINISH_REASON, LLM_INVOCATION_PARAMETERS, LLM_TOKEN_COUNT_TOTAL,
        OUTPUT_VALUE, SESSION_ID, TOOL_CALL_FUNCTION_ARGUMENTS_JSON, TOOL_CALL_FUNCTION_NAME,
        TOOL_CALL_ID,
    };
    use openinference_semantic_conventions::indexers::{
        llm_output_message_content_key, llm_output_message_tool_call_key, llm_tool_key,
    };
    use opentelemetry::context::FutureExt;
    use opentelemetry::trace::{Status, TracerProvider};
    use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracer, SdkTracerProvider, SpanData};
    use serde_json::json;
    use tower::{ServiceBuilder, service_fn};

    const RESPONSE: &str = r#"{
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "created": 0,
        "model": "gpt-6-luna-2026-09-01",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hi!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
    }"#;

    /// A client whose HTTP calls are answered with `status` and `body`,
    /// traced through `OpenInferenceLayer` into an in-memory exporter.
    fn client(
        config: TraceConfig,
        status: u16,
        body: &'static str,
    ) -> (Client<OpenAIConfig>, InMemorySpanExporter) {
        let mock = service_fn(move |_factory: HttpRequestFactory| async move {
            Ok::<_, OpenAIError>(reqwest::Response::from(
                http::Response::builder()
                    .status(status)
                    .header("content-type", "application/json")
                    .body(reqwest::Body::from(body))
                    .unwrap(),
            ))
        });
        client_with(config, mock)
    }

    /// A client whose HTTP calls go to `mock`, traced through
    /// `OpenInferenceLayer` into an in-memory exporter.
    fn client_with<S>(config: TraceConfig, mock: S) -> (Client<OpenAIConfig>, InMemorySpanExporter)
    where
        S: Service<HttpRequestFactory, Response = reqwest::Response, Error = OpenAIError>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer: OITracer<SdkTracer> = OITracer::new(provider.tracer("test"), config);
        let service = ServiceBuilder::new()
            .layer(OpenInferenceLayer::new(tracer))
            .service(mock);
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base("http://example.test/v1")
                .with_api_key("test-key"),
        )
        .with_http_service(service);
        (client, exporter)
    }

    fn chat_request(stream: bool) -> async_openai::types::chat::CreateChatCompletionRequest {
        CreateChatCompletionRequestArgs::default()
            .model("gpt-6-luna")
            .messages([ChatCompletionRequestUserMessage::from("Hi").into()])
            .stream(stream)
            .build()
            .unwrap()
    }

    /// Returns the value of `key`, failing the test if it was recorded more
    /// than once (the SDK keeps every value set for a key).
    fn attribute(span: &SpanData, key: &str) -> Option<opentelemetry::Value> {
        let mut matches = span.attributes.iter().filter(|kv| kv.key.as_str() == key);
        let found = matches.next().map(|kv| kv.value.clone());
        assert!(matches.next().is_none(), "{key} recorded more than once");
        found
    }

    fn spans(exporter: &InMemorySpanExporter) -> Vec<SpanData> {
        exporter.get_finished_spans().unwrap()
    }

    #[tokio::test]
    async fn chat_completion_is_traced() {
        let (client, exporter) = client(TraceConfig::default(), 200, RESPONSE);
        let response = client.chat().create(chat_request(false)).await.unwrap();
        assert_eq!(response.choices[0].message.content.as_deref(), Some("Hi!"));

        let spans = spans(&exporter);
        assert_eq!(spans.len(), 1);
        let span = &spans[0];
        assert_eq!(span.name, CHAT_COMPLETIONS_SPAN_NAME);
        assert_eq!(span.status, Status::Ok);
        assert_eq!(attribute(span, OPENINFERENCE_SPAN_KIND), Some("LLM".into()));
        assert_eq!(attribute(span, LLM_PROVIDER), Some("openai".into()));
        assert_eq!(attribute(span, LLM_SYSTEM), Some("openai".into()));
        // The response's model replaces the requested one.
        assert_eq!(
            attribute(span, LLM_MODEL_NAME),
            Some("gpt-6-luna-2026-09-01".into())
        );
        assert_eq!(attribute(span, LLM_TOKEN_COUNT_TOTAL), Some(7.into()));
        let input: Value =
            serde_json::from_str(attribute(span, INPUT_VALUE).unwrap().as_str().as_ref()).unwrap();
        assert_eq!(input["model"], json!("gpt-6-luna"));
    }

    const TOOL_CALL_RESPONSE: &str = r#"{
        "id": "chatcmpl-2",
        "object": "chat.completion",
        "created": 0,
        "model": "gpt-6-luna-2026-09-01",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {"prompt_tokens": 20, "completion_tokens": 10, "total_tokens": 30}
    }"#;

    fn tool_request() -> async_openai::types::chat::CreateChatCompletionRequest {
        let get_weather = ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObject {
                name: "get_weather".into(),
                description: Some("Current weather for a city".into()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {"city": {"type": "string"}},
                })),
                strict: None,
            },
        });
        CreateChatCompletionRequestArgs::default()
            .model("gpt-6-luna")
            .messages([ChatCompletionRequestUserMessage::from("Weather in Paris?").into()])
            .tools(vec![get_weather])
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn tool_definitions_and_tool_calls_are_traced() {
        let (client, exporter) = client(TraceConfig::default(), 200, TOOL_CALL_RESPONSE);
        let response = client.chat().create(tool_request()).await.unwrap();
        let tool_calls = response.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);

        let span = &spans(&exporter)[0];
        // The tool definition, as sent.
        let schema = attribute(span, &llm_tool_key(0)).expect("tool definition is recorded");
        let schema: Value = serde_json::from_str(schema.as_str().as_ref()).unwrap();
        assert_eq!(schema["type"], json!("function"));
        assert_eq!(schema["function"]["name"], json!("get_weather"));
        assert_eq!(
            schema["function"]["parameters"]["properties"]["city"]["type"],
            json!("string")
        );
        // The tool call the model made.
        let expected = [
            (TOOL_CALL_ID, "call_1"),
            (TOOL_CALL_FUNCTION_NAME, "get_weather"),
            (TOOL_CALL_FUNCTION_ARGUMENTS_JSON, r#"{"city":"Paris"}"#),
        ];
        for (child, value) in expected {
            let key = llm_output_message_tool_call_key(0, 0, child);
            assert_eq!(attribute(span, &key), Some(value.into()), "{key}");
        }
        assert_eq!(
            attribute(span, LLM_FINISH_REASON),
            Some("tool_calls".into())
        );
        // `tools` goes in its own attribute, not the invocation parameters.
        let params = attribute(span, LLM_INVOCATION_PARAMETERS).unwrap();
        let params: Value = serde_json::from_str(params.as_str().as_ref()).unwrap();
        assert_eq!(params.get("tools"), None);
    }

    #[tokio::test]
    async fn hide_llm_tools_drops_only_the_tool_definitions() {
        let config = TraceConfig {
            hide_llm_tools: true,
            ..TraceConfig::default()
        };
        let (client, exporter) = client(config, 200, TOOL_CALL_RESPONSE);
        client.chat().create(tool_request()).await.unwrap();

        let span = &spans(&exporter)[0];
        assert!(
            !span
                .attributes
                .iter()
                .any(|kv| kv.key.as_str().starts_with("llm.tools.")),
            "no tool definitions are recorded"
        );
        let key = llm_output_message_tool_call_key(0, 0, TOOL_CALL_FUNCTION_NAME);
        assert_eq!(attribute(span, &key), Some("get_weather".into()));
    }

    #[tokio::test]
    async fn trace_config_masks_the_span() {
        let config = TraceConfig {
            hide_inputs: true,
            hide_outputs: true,
            ..TraceConfig::default()
        };
        let (client, exporter) = client(config, 200, RESPONSE);
        client.chat().create(chat_request(false)).await.unwrap();

        let span = &spans(&exporter)[0];
        assert_eq!(attribute(span, INPUT_VALUE), Some(REDACTED_VALUE.into()));
        assert_eq!(attribute(span, OUTPUT_VALUE), Some(REDACTED_VALUE.into()));
    }

    #[tokio::test]
    async fn context_attributes_are_recorded() {
        let (client, exporter) = client(TraceConfig::default(), 200, RESPONSE);
        let cx = with_session(&Context::new(), "session-1");
        client
            .chat()
            .create(chat_request(false))
            .with_context(cx)
            .await
            .unwrap();

        assert_eq!(
            attribute(&spans(&exporter)[0], SESSION_ID),
            Some("session-1".into())
        );
    }

    #[tokio::test]
    async fn suppressed_requests_are_not_traced() {
        let (client, exporter) = client(TraceConfig::default(), 200, RESPONSE);
        {
            let _guard = suppress_tracing();
            client.chat().create(chat_request(false)).await.unwrap();
        }
        assert!(spans(&exporter).is_empty());
    }

    const STREAM: &str = concat!(
        "data: {\"id\":\"c\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-6-luna-2026-09-01\",",
        "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hel\"},\"finish_reason\":null}]}\n\n",
        "data: {\"id\":\"c\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-6-luna-2026-09-01\",",
        "\"choices\":[{\"index\":0,\"delta\":{\"content\":\"lo\"},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"id\":\"c\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-6-luna-2026-09-01\",",
        "\"choices\":[],\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":2,\"total_tokens\":7}}\n\n",
        "data: [DONE]\n\n",
    );

    #[tokio::test]
    async fn streaming_responses_are_traced() {
        use futures_core::Stream;
        use std::future::poll_fn;

        let (client, exporter) = client(TraceConfig::default(), 200, STREAM);
        let mut stream = client
            .chat()
            .create_stream(chat_request(true))
            .await
            .unwrap();
        let mut text = String::new();
        while let Some(chunk) = poll_fn(|cx| Pin::new(&mut stream).poll_next(cx)).await {
            if let Some(delta) = chunk.unwrap().choices.first() {
                text.push_str(delta.delta.content.as_deref().unwrap_or_default());
            }
        }
        assert_eq!(text, "Hello");

        let spans = spans(&exporter);
        assert_eq!(spans.len(), 1);
        let span = &spans[0];
        assert_eq!(span.status, Status::Ok);
        assert_eq!(
            attribute(span, &llm_output_message_content_key(0)),
            Some("Hello".into())
        );
        assert_eq!(
            attribute(span, LLM_MODEL_NAME),
            Some("gpt-6-luna-2026-09-01".into())
        );
        assert_eq!(attribute(span, LLM_TOKEN_COUNT_TOTAL), Some(7.into()));
    }

    #[tokio::test]
    async fn requested_model_is_kept_when_the_response_has_none() {
        let (client, exporter) = client(
            TraceConfig::default(),
            401,
            r#"{"error": {"message": "x"}}"#,
        );
        let _ = client.chat().create(chat_request(false)).await;
        assert_eq!(
            attribute(&spans(&exporter)[0], LLM_MODEL_NAME),
            Some("gpt-6-luna".into())
        );
    }

    #[tokio::test]
    async fn http_errors_mark_the_span_as_failed() {
        let error = r#"{"error": {"message": "bad key", "type": "invalid_request_error"}}"#;
        let (client, exporter) = client(TraceConfig::default(), 401, error);
        let result = client.chat().create(chat_request(false)).await;
        assert!(result.is_err());

        let span = &spans(&exporter)[0];
        assert_eq!(span.status, Status::error("HTTP 401 Unauthorized"));
        assert_eq!(attribute(span, OUTPUT_VALUE), None);
    }

    #[tokio::test]
    async fn cancelled_requests_end_the_span_as_cancelled() {
        let never_answers = service_fn(|_factory: HttpRequestFactory| {
            std::future::pending::<Result<reqwest::Response, OpenAIError>>()
        });
        let (client, exporter) = client_with(TraceConfig::default(), never_answers);
        let chat = client.chat();
        tokio::select! {
            biased;
            _ = chat.create(chat_request(false)) => panic!("the mock never answers"),
            // Polled after the request has started, then drops it.
            _ = std::future::ready(()) => {}
        }

        let spans = spans(&exporter);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].status, Status::error("request cancelled"));
        assert_eq!(
            attribute(&spans[0], LLM_MODEL_NAME),
            Some("gpt-6-luna".into())
        );
    }

    #[tokio::test]
    async fn inner_services_run_inside_the_llm_span() {
        use opentelemetry::trace::{SpanId, TraceContextExt};
        use std::sync::Mutex;

        let seen = Arc::new(Mutex::new(SpanId::INVALID));
        let seen_by_mock = Arc::clone(&seen);
        let mock = service_fn(move |_factory: HttpRequestFactory| {
            // Recorded when the inner service is called, as a tracing layer would.
            *seen_by_mock.lock().unwrap() = Context::current().span().span_context().span_id();
            async move {
                Ok::<_, OpenAIError>(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body(reqwest::Body::from(RESPONSE))
                        .unwrap(),
                ))
            }
        });
        let (client, exporter) = client_with(TraceConfig::default(), mock);
        client.chat().create(chat_request(false)).await.unwrap();

        let span = &spans(&exporter)[0];
        assert_eq!(*seen.lock().unwrap(), span.span_context.span_id());
    }

    const STREAM_CUT_OFF: &str = concat!(
        "data: {\"id\":\"c\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-6-luna\",",
        "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hel\"},\"finish_reason\":null}]}\n\n",
    );

    #[tokio::test]
    async fn streams_that_stop_early_are_marked_as_errors() {
        use futures_core::Stream;
        use std::future::poll_fn;

        let (client, exporter) = client(TraceConfig::default(), 200, STREAM_CUT_OFF);
        let mut stream = client
            .chat()
            .create_stream(chat_request(true))
            .await
            .unwrap();
        while poll_fn(|cx| Pin::new(&mut stream).poll_next(cx))
            .await
            .is_some()
        {}

        let span = &spans(&exporter)[0];
        assert_eq!(
            span.status,
            Status::error("stream ended before the response finished")
        );
        assert_eq!(
            attribute(span, &llm_output_message_content_key(0)),
            Some("Hel".into())
        );
    }

    #[test]
    fn azure_hosts_report_the_azure_provider() {
        assert_eq!(
            provider_for_host("my-resource.openai.azure.com"),
            LlmProvider::Azure
        );
        assert_eq!(
            provider_for_host("x.cognitiveservices.azure.com"),
            LlmProvider::Azure
        );
        assert_eq!(provider_for_host("api.openai.com"), LlmProvider::Openai);
        assert_eq!(provider_for_host("localhost"), LlmProvider::Openai);
    }
}
