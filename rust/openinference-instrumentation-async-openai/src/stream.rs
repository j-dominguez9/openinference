use std::collections::BTreeMap;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use opentelemetry::trace::Status;
use serde_json::{Map, Value, json};

use crate::layer::LlmSpan;
use crate::response::response_attributes;

pub(crate) type ByteStream = Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>;

/// Passes a streaming chat completions body through unchanged, and ends the
/// span with the accumulated response once the stream finishes or is dropped.
pub(crate) struct TracedStream {
    inner: ByteStream,
    accumulator: ChatCompletionAccumulator,
    span: LlmSpan,
}

impl TracedStream {
    pub(crate) fn new(inner: ByteStream, span: LlmSpan) -> Self {
        Self {
            inner,
            accumulator: ChatCompletionAccumulator::default(),
            span,
        }
    }

    fn end(&mut self, status: Status) {
        let attributes = response_attributes(&self.accumulator.response());
        self.span.finish(attributes, status);
    }
}

impl Stream for TracedStream {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = self.inner.as_mut().poll_next(cx);
        match &poll {
            Poll::Ready(Some(Ok(bytes))) => {
                self.accumulator.push(bytes);
                // async-openai stops reading at `[DONE]`, so end the span here
                // rather than waiting for the body to close.
                if self.accumulator.done {
                    let status = self.accumulator.status();
                    self.end(status);
                }
            }
            Poll::Ready(Some(Err(err))) => {
                let status = Status::error(err.to_string());
                self.end(status);
            }
            Poll::Ready(None) => {
                self.accumulator.flush();
                let status = self.accumulator.status();
                self.end(status);
            }
            Poll::Pending => {}
        }
        poll
    }
}

impl Drop for TracedStream {
    fn drop(&mut self) {
        // The caller stopped reading early: record what arrived.
        self.end(Status::Unset);
    }
}

/// Rebuilds a chat completion from the server-sent events of a streaming
/// response.
#[derive(Debug, Default)]
pub(crate) struct ChatCompletionAccumulator {
    // Bytes of an event line that hasn't fully arrived yet.
    buffer: Vec<u8>,
    done: bool,
    // The message of an error event sent in the stream.
    error: Option<String>,
    model: Option<String>,
    choices: BTreeMap<u64, ChoiceAccumulator>,
    usage: Option<Value>,
}

#[derive(Debug, Default)]
struct ChoiceAccumulator {
    role: Option<String>,
    content: Option<String>,
    tool_calls: BTreeMap<u64, ToolCallAccumulator>,
    // The legacy single function call: name and arguments.
    function_call: Option<(Option<String>, String)>,
    finish_reason: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCallAccumulator {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ChatCompletionAccumulator {
    /// Adds the next bytes of the stream. Events may be split across calls.
    pub(crate) fn push(&mut self, bytes: &[u8]) {
        // Only the new bytes are searched, so a long event split across many
        // chunks is scanned once.
        let mut rest = bytes;
        while let Some(end) = rest.iter().position(|&b| b == b'\n') {
            self.buffer.extend_from_slice(&rest[..end]);
            let line = std::mem::take(&mut self.buffer);
            self.push_line(&line);
            rest = &rest[end + 1..];
        }
        self.buffer.extend_from_slice(rest);
    }

    /// Processes a final event that wasn't followed by a newline.
    pub(crate) fn flush(&mut self) {
        let line = std::mem::take(&mut self.buffer);
        if !line.is_empty() {
            self.push_line(&line);
        }
    }

    /// The span status for the stream so far: an error if the server sent
    /// one or the stream stopped before the model finished.
    pub(crate) fn status(&self) -> Status {
        if let Some(message) = &self.error {
            return Status::error(message.clone());
        }
        let finished = self
            .choices
            .values()
            .any(|choice| choice.finish_reason.is_some());
        if self.done || finished {
            Status::Ok
        } else {
            Status::error("stream ended before the response finished")
        }
    }

    fn push_line(&mut self, line: &[u8]) {
        let Ok(line) = std::str::from_utf8(line) else {
            return;
        };
        let Some(data) = line.trim_end().strip_prefix("data:") else {
            return;
        };
        let data = data.trim_start();
        if data == "[DONE]" {
            self.done = true;
            return;
        }
        if let Ok(chunk) = serde_json::from_str::<Value>(data) {
            self.push_chunk(&chunk);
        }
    }

    fn push_chunk(&mut self, chunk: &Value) {
        if chunk["error"].is_object() {
            let message = chunk["error"]["message"].as_str().unwrap_or("stream error");
            self.error = Some(message.to_string());
        }
        if let Some(model) = chunk["model"].as_str() {
            self.model = Some(model.to_string());
        }
        if chunk["usage"].is_object() {
            self.usage = Some(chunk["usage"].clone());
        }
        for choice in chunk["choices"].as_array().into_iter().flatten() {
            let index = choice["index"].as_u64().unwrap_or(0);
            let accumulated = self.choices.entry(index).or_default();
            let delta = &choice["delta"];
            if let Some(role) = delta["role"].as_str() {
                accumulated.role = Some(role.to_string());
            }
            if let Some(text) = delta["content"].as_str() {
                accumulated
                    .content
                    .get_or_insert_with(String::new)
                    .push_str(text);
            }
            for call in delta["tool_calls"].as_array().into_iter().flatten() {
                let index = call["index"].as_u64().unwrap_or(0);
                let tool_call = accumulated.tool_calls.entry(index).or_default();
                if let Some(id) = call["id"].as_str() {
                    tool_call.id = Some(id.to_string());
                }
                if let Some(name) = call["function"]["name"].as_str() {
                    tool_call.name = Some(name.to_string());
                }
                if let Some(arguments) = call["function"]["arguments"].as_str() {
                    tool_call.arguments.push_str(arguments);
                }
            }
            let function_call = &delta["function_call"];
            if function_call.is_object() {
                let (name, arguments) = accumulated.function_call.get_or_insert_default();
                if let Some(delta_name) = function_call["name"].as_str() {
                    *name = Some(delta_name.to_string());
                }
                if let Some(delta_arguments) = function_call["arguments"].as_str() {
                    arguments.push_str(delta_arguments);
                }
            }
            if let Some(reason) = choice["finish_reason"].as_str() {
                accumulated.finish_reason = Some(reason.to_string());
            }
        }
    }

    /// The response so far, in the shape of a non-streaming response, so
    /// `response_attributes` can read it.
    pub(crate) fn response(&self) -> Value {
        let choices: Vec<Value> = self
            .choices
            .iter()
            .map(|(index, choice)| {
                let mut message = Map::new();
                if let Some(role) = &choice.role {
                    message.insert("role".into(), role.as_str().into());
                }
                if let Some(content) = &choice.content {
                    message.insert("content".into(), content.as_str().into());
                }
                if !choice.tool_calls.is_empty() {
                    let tool_calls = choice
                        .tool_calls
                        .values()
                        .map(|call| {
                            json!({
                                "id": call.id,
                                "type": "function",
                                "function": {"name": call.name, "arguments": call.arguments},
                            })
                        })
                        .collect();
                    message.insert("tool_calls".into(), Value::Array(tool_calls));
                }
                if let Some((name, arguments)) = &choice.function_call {
                    message.insert(
                        "function_call".into(),
                        json!({"name": name, "arguments": arguments}),
                    );
                }
                json!({
                    "index": index,
                    "message": message,
                    "finish_reason": choice.finish_reason,
                })
            })
            .collect();

        let mut response = Map::new();
        if let Some(model) = &self.model {
            response.insert("model".into(), model.as_str().into());
        }
        response.insert("choices".into(), Value::Array(choices));
        if let Some(usage) = &self.usage {
            response.insert("usage".into(), usage.clone());
        }
        Value::Object(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accumulate(events: &[&str]) -> ChatCompletionAccumulator {
        let mut accumulator = ChatCompletionAccumulator::default();
        for event in events {
            accumulator.push(format!("data: {event}\n\n").as_bytes());
        }
        accumulator
    }

    #[test]
    fn text_deltas_are_joined() {
        let accumulator = accumulate(&[
            r#"{"model":"gpt-6-luna","choices":[{"index":0,"delta":{"role":"assistant","content":""}}]}"#,
            r#"{"model":"gpt-6-luna","choices":[{"index":0,"delta":{"content":"Hel"}}]}"#,
            r#"{"model":"gpt-6-luna","choices":[{"index":0,"delta":{"content":"lo"},"finish_reason":"stop"}]}"#,
            "[DONE]",
        ]);
        assert!(accumulator.done);
        assert_eq!(
            accumulator.response(),
            json!({
                "model": "gpt-6-luna",
                "choices": [{
                    "index": 0,
                    "message": {"role": "assistant", "content": "Hello"},
                    "finish_reason": "stop",
                }],
            })
        );
    }

    #[test]
    fn tool_call_arguments_are_joined() {
        let accumulator = accumulate(&[
            r#"{"choices":[{"index":0,"delta":{"role":"assistant","tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"get_weather","arguments":""}}]}}]}"#,
            r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"city\":"}}]}}]}"#,
            r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"Paris\"}"}}]},"finish_reason":"tool_calls"}]}"#,
        ]);
        assert_eq!(
            accumulator.response()["choices"][0]["message"]["tool_calls"],
            json!([{
                "id": "call_1",
                "type": "function",
                "function": {"name": "get_weather", "arguments": r#"{"city":"Paris"}"#},
            }])
        );
    }

    #[test]
    fn usage_from_the_final_chunk_is_kept() {
        let accumulator = accumulate(&[
            r#"{"choices":[{"index":0,"delta":{"content":"Hi"}}],"usage":null}"#,
            r#"{"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":1,"total_tokens":6}}"#,
        ]);
        assert_eq!(
            accumulator.response()["usage"],
            json!({"prompt_tokens": 5, "completion_tokens": 1, "total_tokens": 6})
        );
    }

    #[test]
    fn events_split_across_chunks_are_reassembled() {
        let mut accumulator = ChatCompletionAccumulator::default();
        let event = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"}}]}\n\n";
        let (first, second) = event.split_at(20);
        accumulator.push(first.as_bytes());
        accumulator.push(second.as_bytes());
        assert_eq!(
            accumulator.response()["choices"][0]["message"]["content"],
            json!("Hi")
        );
    }

    #[test]
    fn legacy_function_call_deltas_are_joined() {
        let accumulator = accumulate(&[
            r#"{"choices":[{"index":0,"delta":{"role":"assistant","function_call":{"name":"get_weather","arguments":""}}}]}"#,
            r#"{"choices":[{"index":0,"delta":{"function_call":{"arguments":"{\"city\":"}}}]}"#,
            r#"{"choices":[{"index":0,"delta":{"function_call":{"arguments":"\"Paris\"}"}},"finish_reason":"function_call"}]}"#,
        ]);
        assert_eq!(
            accumulator.response()["choices"][0]["message"]["function_call"],
            json!({"name": "get_weather", "arguments": r#"{"city":"Paris"}"#})
        );
    }

    #[test]
    fn a_final_event_without_a_newline_is_flushed() {
        let mut accumulator = ChatCompletionAccumulator::default();
        accumulator.push(br#"data: {"choices":[],"usage":{"total_tokens":6}}"#);
        assert_eq!(accumulator.response().get("usage"), None);
        accumulator.flush();
        assert_eq!(accumulator.response()["usage"], json!({"total_tokens": 6}));
    }

    #[test]
    fn an_event_split_into_single_bytes_is_reassembled() {
        let mut accumulator = ChatCompletionAccumulator::default();
        for byte in b"data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"}}]}\n" {
            accumulator.push(&[*byte]);
        }
        assert_eq!(
            accumulator.response()["choices"][0]["message"]["content"],
            json!("Hi")
        );
    }

    #[test]
    fn status_is_ok_once_the_model_finishes() {
        let finished =
            accumulate(&[r#"{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#]);
        assert_eq!(finished.status(), Status::Ok);
        assert_eq!(accumulate(&["[DONE]"]).status(), Status::Ok);
    }

    #[test]
    fn status_is_an_error_when_the_stream_stops_early() {
        let cut_off = accumulate(&[r#"{"choices":[{"index":0,"delta":{"content":"Hel"}}]}"#]);
        assert_eq!(
            cut_off.status(),
            Status::error("stream ended before the response finished")
        );
    }

    #[test]
    fn status_reports_an_error_event() {
        let failed = accumulate(&[
            r#"{"choices":[{"index":0,"delta":{"content":"Hel"}}]}"#,
            r#"{"error":{"message":"server overloaded","type":"server_error"}}"#,
            "[DONE]",
        ]);
        assert_eq!(failed.status(), Status::error("server overloaded"));
    }

    #[test]
    fn choices_are_kept_separate() {
        let accumulator = accumulate(&[
            r#"{"choices":[{"index":0,"delta":{"content":"A"}},{"index":1,"delta":{"content":"B"}}]}"#,
        ]);
        let response = accumulator.response();
        assert_eq!(response["choices"][0]["message"]["content"], json!("A"));
        assert_eq!(response["choices"][1]["message"]["content"], json!("B"));
    }
}
