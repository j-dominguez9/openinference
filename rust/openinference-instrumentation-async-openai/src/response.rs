use openinference_semantic_conventions::{
    attributes::{
        LLM_FINISH_REASON, LLM_MODEL_NAME, LLM_OUTPUT_MESSAGES, LLM_TOKEN_COUNT_COMPLETION,
        LLM_TOKEN_COUNT_COMPLETION_DETAILS_AUDIO, LLM_TOKEN_COUNT_COMPLETION_DETAILS_REASONING,
        LLM_TOKEN_COUNT_PROMPT, LLM_TOKEN_COUNT_PROMPT_DETAILS_AUDIO,
        LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_READ, LLM_TOKEN_COUNT_TOTAL, OUTPUT_MIME_TYPE,
        OUTPUT_VALUE,
    },
    enums::MimeType,
};
use opentelemetry::KeyValue;
use serde_json::Value;

use crate::message::message_attributes;

/// Returns the OpenInference attributes for a chat completions response body.
pub(crate) fn response_attributes(body: &Value) -> Vec<KeyValue> {
    let mut attributes = Vec::new();
    if let Some(model) = body["model"].as_str() {
        attributes.push(KeyValue::new(LLM_MODEL_NAME, model.to_string()));
    }
    attributes.push(KeyValue::new(OUTPUT_VALUE, body.to_string()));
    attributes.push(KeyValue::new(OUTPUT_MIME_TYPE, MimeType::Json.as_str()));

    if let Some(choices) = body["choices"].as_array() {
        for (i, choice) in choices.iter().enumerate() {
            let prefix = format!("{LLM_OUTPUT_MESSAGES}.{i}");
            attributes.extend(message_attributes(&prefix, &choice["message"]));
        }
    }

    if let Some(reason) = body["choices"][0]["finish_reason"].as_str() {
        attributes.push(KeyValue::new(LLM_FINISH_REASON, reason.to_string()));
    }

    let usage = &body["usage"];
    let token_counts = [
        (LLM_TOKEN_COUNT_PROMPT, &usage["prompt_tokens"]),
        (LLM_TOKEN_COUNT_COMPLETION, &usage["completion_tokens"]),
        (LLM_TOKEN_COUNT_TOTAL, &usage["total_tokens"]),
        (
            LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_READ,
            &usage["prompt_tokens_details"]["cached_tokens"],
        ),
        (
            LLM_TOKEN_COUNT_PROMPT_DETAILS_AUDIO,
            &usage["prompt_tokens_details"]["audio_tokens"],
        ),
        (
            LLM_TOKEN_COUNT_COMPLETION_DETAILS_REASONING,
            &usage["completion_tokens_details"]["reasoning_tokens"],
        ),
        (
            LLM_TOKEN_COUNT_COMPLETION_DETAILS_AUDIO,
            &usage["completion_tokens_details"]["audio_tokens"],
        ),
    ];
    for (key, value) in token_counts {
        if let Some(count) = value.as_i64() {
            attributes.push(KeyValue::new(key, count));
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use openinference_semantic_conventions::attributes::{
        TOOL_CALL_FUNCTION_ARGUMENTS_JSON, TOOL_CALL_FUNCTION_NAME, TOOL_CALL_ID,
    };
    use openinference_semantic_conventions::indexers::{
        llm_output_message_content_key, llm_output_message_role_key,
        llm_output_message_tool_call_key,
    };
    use serde_json::json;

    /// Returns the value of `key`, failing the test if it appears more than once.
    fn get(attributes: &[KeyValue], key: &str) -> Option<opentelemetry::Value> {
        let mut matches = attributes.iter().filter(|kv| kv.key.as_str() == key);
        let found = matches.next().map(|kv| kv.value.clone());
        assert!(matches.next().is_none(), "{key} recorded more than once");
        found
    }

    fn chat_response() -> Value {
        json!({
            "model": "gpt-6-luna-2026-09-01",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hi!"},
                "finish_reason": "stop",
            }],
            "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7},
        })
    }

    #[test]
    fn model_and_output_value_are_recorded() {
        let body = chat_response();
        let attributes = response_attributes(&body);
        assert_eq!(
            get(&attributes, LLM_MODEL_NAME),
            Some("gpt-6-luna-2026-09-01".into())
        );
        assert_eq!(
            get(&attributes, OUTPUT_VALUE),
            Some(body.to_string().into())
        );
        assert_eq!(
            get(&attributes, OUTPUT_MIME_TYPE),
            Some("application/json".into())
        );
    }

    #[test]
    fn choice_messages_are_flattened_by_index() {
        let attributes = response_attributes(&chat_response());
        assert_eq!(
            get(&attributes, &llm_output_message_role_key(0)),
            Some("assistant".into())
        );
        assert_eq!(
            get(&attributes, &llm_output_message_content_key(0)),
            Some("Hi!".into())
        );
    }

    #[test]
    fn every_choice_is_recorded() {
        let body = json!({"choices": [
            {"message": {"role": "assistant", "content": "A"}},
            {"message": {"role": "assistant", "content": "B"}},
        ]});
        let attributes = response_attributes(&body);
        assert_eq!(
            get(&attributes, &llm_output_message_content_key(0)),
            Some("A".into())
        );
        assert_eq!(
            get(&attributes, &llm_output_message_content_key(1)),
            Some("B".into())
        );
    }

    #[test]
    fn finish_reason_comes_from_the_first_choice() {
        let attributes = response_attributes(&chat_response());
        assert_eq!(get(&attributes, LLM_FINISH_REASON), Some("stop".into()));
    }

    #[test]
    fn token_counts_are_recorded_as_integers() {
        let attributes = response_attributes(&chat_response());
        assert_eq!(get(&attributes, LLM_TOKEN_COUNT_PROMPT), Some(5.into()));
        assert_eq!(get(&attributes, LLM_TOKEN_COUNT_COMPLETION), Some(2.into()));
        assert_eq!(get(&attributes, LLM_TOKEN_COUNT_TOTAL), Some(7.into()));
    }

    #[test]
    fn token_details_are_recorded() {
        let body = json!({"usage": {
            "prompt_tokens": 15,
            "completion_tokens": 81,
            "total_tokens": 96,
            "prompt_tokens_details": {"cached_tokens": 4, "audio_tokens": 0},
            "completion_tokens_details": {"reasoning_tokens": 47, "audio_tokens": 0},
        }});
        let attributes = response_attributes(&body);
        let expected = [
            (LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_READ, 4),
            (LLM_TOKEN_COUNT_PROMPT_DETAILS_AUDIO, 0),
            (LLM_TOKEN_COUNT_COMPLETION_DETAILS_REASONING, 47),
            (LLM_TOKEN_COUNT_COMPLETION_DETAILS_AUDIO, 0),
        ];
        for (key, count) in expected {
            assert_eq!(get(&attributes, key), Some(count.into()), "{key}");
        }
    }

    #[test]
    fn missing_usage_is_skipped() {
        let body = json!({"choices": []});
        assert_eq!(
            get(&response_attributes(&body), LLM_TOKEN_COUNT_PROMPT),
            None
        );
    }

    #[test]
    fn tool_calls_are_flattened_by_message_and_call_index() {
        let body = json!({"choices": [{
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [
                    {"id": "call_1", "type": "function",
                     "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}},
                    {"id": "call_2", "type": "function",
                     "function": {"name": "get_time", "arguments": "{}"}},
                ],
            },
            "finish_reason": "tool_calls",
        }]});
        let attributes = response_attributes(&body);
        let expected = [
            (0, TOOL_CALL_ID, "call_1"),
            (0, TOOL_CALL_FUNCTION_NAME, "get_weather"),
            (0, TOOL_CALL_FUNCTION_ARGUMENTS_JSON, r#"{"city":"Paris"}"#),
            (1, TOOL_CALL_ID, "call_2"),
            (1, TOOL_CALL_FUNCTION_NAME, "get_time"),
            (1, TOOL_CALL_FUNCTION_ARGUMENTS_JSON, "{}"),
        ];
        for (j, child, value) in expected {
            let key = llm_output_message_tool_call_key(0, j, child);
            assert_eq!(get(&attributes, &key), Some(value.into()), "{key}");
        }
        // `content: null` records no content attribute.
        assert_eq!(get(&attributes, &llm_output_message_content_key(0)), None);
        assert_eq!(
            get(&attributes, LLM_FINISH_REASON),
            Some("tool_calls".into())
        );
    }
}
