use openinference_semantic_conventions::attributes::{
    IMAGE_URL, MESSAGE_CONTENT, MESSAGE_CONTENT_IMAGE, MESSAGE_CONTENT_TEXT, MESSAGE_CONTENT_TYPE,
    MESSAGE_CONTENTS, MESSAGE_FUNCTION_CALL_ARGUMENTS_JSON, MESSAGE_FUNCTION_CALL_NAME,
    MESSAGE_NAME, MESSAGE_ROLE, MESSAGE_TOOL_CALL_ID, MESSAGE_TOOL_CALLS,
    TOOL_CALL_FUNCTION_ARGUMENTS_JSON, TOOL_CALL_FUNCTION_NAME, TOOL_CALL_ID,
};
use opentelemetry::KeyValue;
use serde_json::Value;

/// Returns the attributes for one chat message, keyed under `prefix`, e.g.
/// `llm.input_messages.0`. Request and response messages share this format.
pub(crate) fn message_attributes(prefix: &str, message: &Value) -> Vec<KeyValue> {
    let mut attributes = Vec::new();
    let mut push = |child: &str, value: &Value| {
        if let Some(value) = value.as_str() {
            attributes.push(KeyValue::new(
                format!("{prefix}.{child}"),
                value.to_string(),
            ));
        }
    };

    push(MESSAGE_ROLE, &message["role"]);
    push(MESSAGE_NAME, &message["name"]);
    push(MESSAGE_TOOL_CALL_ID, &message["tool_call_id"]);

    // Content is either a string or a list of parts, such as text and images.
    match &message["content"] {
        Value::String(_) => push(MESSAGE_CONTENT, &message["content"]),
        Value::Array(parts) => {
            for (j, part) in parts.iter().enumerate() {
                let part_prefix = format!("{MESSAGE_CONTENTS}.{j}");
                match part["type"].as_str() {
                    Some("text") => {
                        push(
                            &format!("{part_prefix}.{MESSAGE_CONTENT_TYPE}"),
                            &"text".into(),
                        );
                        push(
                            &format!("{part_prefix}.{MESSAGE_CONTENT_TEXT}"),
                            &part["text"],
                        );
                    }
                    Some("image_url") => {
                        push(
                            &format!("{part_prefix}.{MESSAGE_CONTENT_TYPE}"),
                            &"image".into(),
                        );
                        // `image_url` is either `{"url": ...}` or a bare string.
                        let url = match &part["image_url"] {
                            Value::String(_) => &part["image_url"],
                            image => &image["url"],
                        };
                        push(
                            &format!("{part_prefix}.{MESSAGE_CONTENT_IMAGE}.{IMAGE_URL}"),
                            url,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    // Legacy single function call.
    push(
        MESSAGE_FUNCTION_CALL_NAME,
        &message["function_call"]["name"],
    );
    push(
        MESSAGE_FUNCTION_CALL_ARGUMENTS_JSON,
        &message["function_call"]["arguments"],
    );

    if let Some(tool_calls) = message["tool_calls"].as_array() {
        for (j, tool_call) in tool_calls.iter().enumerate() {
            let call_prefix = format!("{MESSAGE_TOOL_CALLS}.{j}");
            push(&format!("{call_prefix}.{TOOL_CALL_ID}"), &tool_call["id"]);
            push(
                &format!("{call_prefix}.{TOOL_CALL_FUNCTION_NAME}"),
                &tool_call["function"]["name"],
            );
            push(
                &format!("{call_prefix}.{TOOL_CALL_FUNCTION_ARGUMENTS_JSON}"),
                &tool_call["function"]["arguments"],
            );
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn attributes(message: Value) -> Vec<(String, String)> {
        message_attributes("m", &message)
            .into_iter()
            .map(|kv| (kv.key.to_string(), kv.value.as_str().into_owned()))
            .collect()
    }

    fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
        expected
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn text_message() {
        let message = json!({"role": "user", "content": "Hi", "name": "ana"});
        assert_eq!(
            attributes(message),
            pairs(&[
                ("m.message.role", "user"),
                ("m.message.name", "ana"),
                ("m.message.content", "Hi"),
            ])
        );
    }

    #[test]
    fn content_parts_are_recorded_as_contents() {
        let message = json!({"role": "user", "content": [
            {"type": "text", "text": "What is this?"},
            {"type": "image_url", "image_url": {"url": "https://x.test/cat.png"}},
            {"type": "image_url", "image_url": "https://x.test/dog.png"},
            {"type": "input_audio", "input_audio": {}},
        ]});
        assert_eq!(
            attributes(message),
            pairs(&[
                ("m.message.role", "user"),
                ("m.message.contents.0.message_content.type", "text"),
                ("m.message.contents.0.message_content.text", "What is this?"),
                ("m.message.contents.1.message_content.type", "image"),
                (
                    "m.message.contents.1.message_content.image.image.url",
                    "https://x.test/cat.png",
                ),
                ("m.message.contents.2.message_content.type", "image"),
                (
                    "m.message.contents.2.message_content.image.image.url",
                    "https://x.test/dog.png",
                ),
            ])
        );
    }

    #[test]
    fn assistant_tool_calls() {
        let message = json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": {"name": "get_weather", "arguments": "{}"},
            }],
        });
        assert_eq!(
            attributes(message),
            pairs(&[
                ("m.message.role", "assistant"),
                ("m.message.tool_calls.0.tool_call.id", "call_1"),
                (
                    "m.message.tool_calls.0.tool_call.function.name",
                    "get_weather"
                ),
                ("m.message.tool_calls.0.tool_call.function.arguments", "{}"),
            ])
        );
    }

    #[test]
    fn tool_result_message() {
        let message = json!({"role": "tool", "tool_call_id": "call_1", "content": "Sunny"});
        assert_eq!(
            attributes(message),
            pairs(&[
                ("m.message.role", "tool"),
                ("m.message.tool_call_id", "call_1"),
                ("m.message.content", "Sunny"),
            ])
        );
    }

    #[test]
    fn legacy_function_call() {
        let message = json!({
            "role": "assistant",
            "function_call": {"name": "get_weather", "arguments": "{}"},
        });
        assert_eq!(
            attributes(message),
            pairs(&[
                ("m.message.role", "assistant"),
                ("m.message.function_call_name", "get_weather"),
                ("m.message.function_call_arguments_json", "{}"),
            ])
        );
    }
}
