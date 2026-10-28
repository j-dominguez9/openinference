use openinference_semantic_conventions::{
    attributes::{
        INPUT_MIME_TYPE, INPUT_VALUE, LLM_INPUT_MESSAGES, LLM_INVOCATION_PARAMETERS, LLM_MODEL_NAME,
    },
    enums::MimeType,
    indexers::llm_tool_key,
};
use opentelemetry::KeyValue;
use serde_json::Value;

use crate::message::message_attributes;

/// Returns the OpenInference attributes for a chat completions request body.
pub(crate) fn request_attributes(body: &Value) -> Vec<KeyValue> {
    let mut attributes = Vec::new();
    if let Some(model) = body["model"].as_str() {
        attributes.push(KeyValue::new(LLM_MODEL_NAME, model.to_string()));
    }
    if let Some(object) = body.as_object() {
        // Copy only the parameters, not the messages and tools.
        let params: serde_json::Map<String, Value> = object
            .iter()
            .filter(|(key, _)| !matches!(key.as_str(), "messages" | "functions" | "tools"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        if !params.is_empty() {
            attributes.push(KeyValue::new(
                LLM_INVOCATION_PARAMETERS,
                Value::Object(params).to_string(),
            ));
        }
    }
    attributes.push(KeyValue::new(INPUT_VALUE, body.to_string()));
    attributes.push(KeyValue::new(INPUT_MIME_TYPE, MimeType::Json.as_str()));

    if let Some(messages) = body["messages"].as_array() {
        for (i, message) in messages.iter().enumerate() {
            let prefix = format!("{LLM_INPUT_MESSAGES}.{i}");
            attributes.extend(message_attributes(&prefix, message));
        }
    }

    // Each tool definition is recorded as its JSON schema.
    if let Some(tools) = body["tools"].as_array() {
        for (i, tool) in tools.iter().enumerate() {
            attributes.push(KeyValue::new(llm_tool_key(i), tool.to_string()));
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use openinference_semantic_conventions::indexers::{
        llm_input_message_content_key, llm_input_message_role_key,
    };
    use serde_json::json;

    /// Returns the value of `key`, failing the test if it appears more than once.
    fn get(attributes: &[KeyValue], key: &str) -> Option<opentelemetry::Value> {
        let mut matches = attributes.iter().filter(|kv| kv.key.as_str() == key);
        let found = matches.next().map(|kv| kv.value.clone());
        assert!(matches.next().is_none(), "{key} recorded more than once");
        found
    }

    #[test]
    fn model_is_recorded() {
        let body = json!({"model": "gpt-6-luna", "messages": []});
        assert_eq!(
            get(&request_attributes(&body), LLM_MODEL_NAME),
            Some("gpt-6-luna".into())
        );
    }

    #[test]
    fn missing_model_is_skipped() {
        let body = json!({"messages": []});
        assert_eq!(get(&request_attributes(&body), LLM_MODEL_NAME), None);
    }

    #[test]
    fn invocation_parameters_exclude_messages_functions_and_tools() {
        let body = json!({
            "model": "gpt-6-luna",
            "temperature": 0.2,
            "messages": [{"role": "user", "content": "Hi"}],
            "functions": [],
            "tools": [],
        });
        let params = get(&request_attributes(&body), LLM_INVOCATION_PARAMETERS)
            .expect("invocation parameters are recorded");
        // Compare parsed JSON so key order doesn't matter.
        let params: Value = serde_json::from_str(params.as_str().as_ref()).unwrap();
        assert_eq!(params, json!({"model": "gpt-6-luna", "temperature": 0.2}));
    }

    #[test]
    fn no_invocation_parameters_when_only_messages_are_sent() {
        let body = json!({"messages": [], "tools": []});
        assert_eq!(
            get(&request_attributes(&body), LLM_INVOCATION_PARAMETERS),
            None
        );
    }

    #[test]
    fn input_value_is_the_request_body_as_json() {
        let body = json!({"model": "gpt-6-luna", "messages": []});
        let attributes = request_attributes(&body);
        assert_eq!(get(&attributes, INPUT_VALUE), Some(body.to_string().into()));
        assert_eq!(
            get(&attributes, INPUT_MIME_TYPE),
            Some("application/json".into())
        );
    }

    #[test]
    fn messages_are_flattened_by_index() {
        let body = json!({
            "model": "gpt-6-luna",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hi"},
            ],
        });
        let attributes = request_attributes(&body);
        let expected = [
            (llm_input_message_role_key(0), "system"),
            (llm_input_message_content_key(0), "You are helpful."),
            (llm_input_message_role_key(1), "user"),
            (llm_input_message_content_key(1), "Hi"),
        ];
        for (key, value) in expected {
            assert_eq!(get(&attributes, &key), Some(value.into()), "{key}");
        }
    }

    #[test]
    fn content_parts_are_not_recorded_as_plain_content() {
        let body = json!({
            "messages": [{
                "role": "user",
                "content": [{"type": "text", "text": "Hi"}],
            }],
        });
        let attributes = request_attributes(&body);
        assert_eq!(
            get(&attributes, &llm_input_message_role_key(0)),
            Some("user".into())
        );
        assert_eq!(get(&attributes, &llm_input_message_content_key(0)), None);
    }

    #[test]
    fn tools_are_recorded_as_json_schemas() {
        let tool = json!({"type": "function", "function": {"name": "get_weather"}});
        let body = json!({"messages": [], "tools": [tool]});
        let attributes = request_attributes(&body);
        let recorded = get(&attributes, &llm_tool_key(0)).expect("tool is recorded");
        let recorded: Value = serde_json::from_str(recorded.as_str().as_ref()).unwrap();
        assert_eq!(recorded, tool);
    }

    #[test]
    fn non_object_body_only_records_input_value() {
        let body = json!("not an object");
        let keys: Vec<String> = request_attributes(&body)
            .iter()
            .map(|kv| kv.key.to_string())
            .collect();
        assert_eq!(keys, [INPUT_VALUE, INPUT_MIME_TYPE]);
    }
}
