//! Helpers that build indexed attribute keys.
//!
//! OTel span attributes are a flat key/value map, so lists (messages,
//! documents, tool calls, embeddings) are flattened into keys that carry an
//! index, e.g. `llm.input_messages.0.message.role`. These helpers build those
//! keys so callers don't format them by hand.

use crate::attributes::{
    ANNOTATIONS, COMPLETION_TEXT, EMBEDDING_EMBEDDINGS, EVALUATIONS, INPUT_IMAGES, LLM_CHOICES,
    LLM_INPUT_MESSAGES, LLM_OUTPUT_MESSAGES, LLM_PROMPTS, LLM_TOOLS, MESSAGE_CONTENT, MESSAGE_NAME,
    MESSAGE_ROLE, MESSAGE_TOOL_CALL_ID, MESSAGE_TOOL_CALLS, OUTPUT_IMAGES, PROMPT_TEXT,
    RETRIEVAL_DOCUMENTS, SESSION_ANNOTATIONS, SESSION_EVALUATIONS, TOOL_JSON_SCHEMA,
    TRACE_ANNOTATIONS, TRACE_EVALUATIONS,
};

/// Builds `{prefix}.{i}.{child}`, the shape shared by every helper below.
fn indexed_key(prefix: &str, i: usize, child: &str) -> String {
    format!("{prefix}.{i}.{child}")
}

/// Returns the key for a field of the `i`-th span annotation.
///
/// `child` is one of the `ANNOTATION_*` attributes.
///
/// # Examples
///
/// ```
/// use openinference_semantic_conventions::attributes::ANNOTATION_NAME;
/// use openinference_semantic_conventions::indexers::annotation_key;
///
/// assert_eq!(annotation_key(0, ANNOTATION_NAME), "annotations.0.annotation.name");
/// ```
pub fn annotation_key(i: usize, child: &str) -> String {
    indexed_key(ANNOTATIONS, i, child)
}

/// Returns the key for a field of the `i`-th span evaluation, e.g.
/// `evaluations.0.evaluation.score`.
///
/// `child` is one of the `EVALUATION_*` attributes.
pub fn evaluation_key(i: usize, child: &str) -> String {
    indexed_key(EVALUATIONS, i, child)
}

/// Returns the key for a field of the `i`-th trace-scoped annotation, e.g.
/// `trace.annotations.0.annotation.name`.
///
/// `child` is one of the `ANNOTATION_*` attributes.
pub fn trace_annotation_key(i: usize, child: &str) -> String {
    indexed_key(TRACE_ANNOTATIONS, i, child)
}

/// Returns the key for a field of the `i`-th trace-scoped evaluation, e.g.
/// `trace.evaluations.0.evaluation.score`.
///
/// `child` is one of the `EVALUATION_*` attributes.
pub fn trace_evaluation_key(i: usize, child: &str) -> String {
    indexed_key(TRACE_EVALUATIONS, i, child)
}

/// Returns the key for a field of the `i`-th session-scoped annotation, e.g.
/// `session.annotations.0.annotation.label`.
///
/// `child` is one of the `ANNOTATION_*` attributes. Session-scoped feedback
/// requires [`SESSION_ID`](crate::attributes::SESSION_ID) on the span.
pub fn session_annotation_key(i: usize, child: &str) -> String {
    indexed_key(SESSION_ANNOTATIONS, i, child)
}

/// Returns the key for a field of the `i`-th session-scoped evaluation, e.g.
/// `session.evaluations.0.evaluation.label`.
///
/// `child` is one of the `EVALUATION_*` attributes. Session-scoped feedback
/// requires [`SESSION_ID`](crate::attributes::SESSION_ID) on the span.
pub fn session_evaluation_key(i: usize, child: &str) -> String {
    indexed_key(SESSION_EVALUATIONS, i, child)
}

/// Returns the key for the role of the `i`-th input message, e.g.
/// `llm.input_messages.0.message.role`.
pub fn llm_input_message_role_key(i: usize) -> String {
    indexed_key(LLM_INPUT_MESSAGES, i, MESSAGE_ROLE)
}

/// Returns the key for the content of the `i`-th input message, e.g.
/// `llm.input_messages.0.message.content`.
pub fn llm_input_message_content_key(i: usize) -> String {
    indexed_key(LLM_INPUT_MESSAGES, i, MESSAGE_CONTENT)
}

/// Returns the key for the name of the `i`-th input message, e.g.
/// `llm.input_messages.0.message.name`.
///
/// Used on `tool` and `function` role messages to name the tool or function
/// that produced them.
pub fn llm_input_message_name_key(i: usize) -> String {
    indexed_key(LLM_INPUT_MESSAGES, i, MESSAGE_NAME)
}

/// Returns the key for the tool call id of the `i`-th input message, e.g.
/// `llm.input_messages.0.message.tool_call_id`.
///
/// Set on a tool-result message to link it to the tool call it answers.
pub fn llm_input_message_tool_call_id_key(i: usize) -> String {
    indexed_key(LLM_INPUT_MESSAGES, i, MESSAGE_TOOL_CALL_ID)
}

/// Returns the key for the role of the `i`-th output message, e.g.
/// `llm.output_messages.0.message.role`.
pub fn llm_output_message_role_key(i: usize) -> String {
    indexed_key(LLM_OUTPUT_MESSAGES, i, MESSAGE_ROLE)
}

/// Returns the key for the content of the `i`-th output message, e.g.
/// `llm.output_messages.0.message.content`.
pub fn llm_output_message_content_key(i: usize) -> String {
    indexed_key(LLM_OUTPUT_MESSAGES, i, MESSAGE_CONTENT)
}

/// Returns the key for a field of the `j`-th tool call on the `i`-th output
/// message, e.g. `llm.output_messages.0.message.tool_calls.1.tool_call.id`.
///
/// `child` is one of the `TOOL_CALL_*` attributes.
pub fn llm_output_message_tool_call_key(i: usize, j: usize, child: &str) -> String {
    indexed_key(
        LLM_OUTPUT_MESSAGES,
        i,
        &indexed_key(MESSAGE_TOOL_CALLS, j, child),
    )
}

/// Returns the key for a field of the `j`-th tool call on the `i`-th input
/// message, e.g. `llm.input_messages.0.message.tool_calls.1.tool_call.id`.
///
/// `child` is one of the `TOOL_CALL_*` attributes. Input messages carry tool
/// calls when earlier assistant turns are replayed to the model.
pub fn llm_input_message_tool_call_key(i: usize, j: usize, child: &str) -> String {
    indexed_key(
        LLM_INPUT_MESSAGES,
        i,
        &indexed_key(MESSAGE_TOOL_CALLS, j, child),
    )
}

/// Returns the key for the text of the `i`-th prompt sent to a completions
/// API, e.g. `llm.prompts.0.prompt.text`.
pub fn llm_prompt_key(i: usize) -> String {
    indexed_key(LLM_PROMPTS, i, PROMPT_TEXT)
}

/// Returns the key for the text of the `i`-th choice returned by a
/// completions API, e.g. `llm.choices.0.completion.text`.
pub fn llm_choice_key(i: usize) -> String {
    indexed_key(LLM_CHOICES, i, COMPLETION_TEXT)
}

/// Returns the key for the JSON schema of the `i`-th tool advertised to the
/// model, e.g. `llm.tools.0.tool.json_schema`.
pub fn llm_tool_key(i: usize) -> String {
    indexed_key(LLM_TOOLS, i, TOOL_JSON_SCHEMA)
}

/// Returns the key for a field of the `i`-th retrieved document, e.g.
/// `retrieval.documents.0.document.content`.
///
/// `child` is one of the `DOCUMENT_*` attributes.
pub fn retrieval_document_key(i: usize, child: &str) -> String {
    indexed_key(RETRIEVAL_DOCUMENTS, i, child)
}

/// Returns the key for a field of the `i`-th embedding, e.g.
/// `embedding.embeddings.0.embedding.text`.
///
/// `child` is [`EMBEDDING_TEXT`](crate::attributes::EMBEDDING_TEXT) or
/// [`EMBEDDING_VECTOR`](crate::attributes::EMBEDDING_VECTOR).
pub fn embedding_key(i: usize, child: &str) -> String {
    indexed_key(EMBEDDING_EMBEDDINGS, i, child)
}

/// Returns the key for a field of the `i`-th input image, e.g.
/// `input.images.0.image.url`.
///
/// `child` is usually [`IMAGE_URL`](crate::attributes::IMAGE_URL).
pub fn input_image_key(i: usize, child: &str) -> String {
    indexed_key(INPUT_IMAGES, i, child)
}

/// Returns the key for a field of the `i`-th output image, e.g.
/// `output.images.0.image.url`.
///
/// `child` is usually [`IMAGE_URL`](crate::attributes::IMAGE_URL).
pub fn output_image_key(i: usize, child: &str) -> String {
    indexed_key(OUTPUT_IMAGES, i, child)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{
        ANNOTATION_LABEL, ANNOTATION_NAME, ANNOTATION_SCORE, DOCUMENT_CONTENT, EMBEDDING_TEXT,
        EVALUATION_LABEL, EVALUATION_SCORE, IMAGE_URL, TOOL_CALL_FUNCTION_NAME, TOOL_CALL_ID,
    };

    // Each test uses a nonzero index that differs from the other indexes in
    // the same call, so a helper that ignores or swaps an index fails.

    #[test]
    fn llm_input_message_tool_call_key_nests_both_indexes() {
        assert_eq!(
            llm_input_message_tool_call_key(2, 1, TOOL_CALL_ID),
            "llm.input_messages.2.message.tool_calls.1.tool_call.id"
        );
    }

    #[test]
    fn llm_output_message_tool_call_key_nests_both_indexes() {
        assert_eq!(
            llm_output_message_tool_call_key(2, 1, TOOL_CALL_FUNCTION_NAME),
            "llm.output_messages.2.message.tool_calls.1.tool_call.function.name"
        );
    }

    #[test]
    fn evaluation_key_inserts_index_and_child() {
        assert_eq!(
            evaluation_key(5, EVALUATION_SCORE),
            "evaluations.5.evaluation.score"
        );
    }

    #[test]
    fn trace_annotation_key_inserts_index_and_child() {
        assert_eq!(
            trace_annotation_key(2, ANNOTATION_NAME),
            "trace.annotations.2.annotation.name"
        );
    }

    #[test]
    fn trace_evaluation_key_inserts_index_and_child() {
        assert_eq!(
            trace_evaluation_key(1, EVALUATION_SCORE),
            "trace.evaluations.1.evaluation.score"
        );
    }

    #[test]
    fn session_annotation_key_inserts_index_and_child() {
        assert_eq!(
            session_annotation_key(4, ANNOTATION_LABEL),
            "session.annotations.4.annotation.label"
        );
    }

    #[test]
    fn llm_input_message_role_key_inserts_index() {
        assert_eq!(
            llm_input_message_role_key(3),
            "llm.input_messages.3.message.role"
        );
    }

    #[test]
    fn remaining_helpers_build_expected_keys() {
        let cases = [
            (
                annotation_key(6, ANNOTATION_SCORE),
                "annotations.6.annotation.score",
            ),
            (
                session_evaluation_key(7, EVALUATION_LABEL),
                "session.evaluations.7.evaluation.label",
            ),
            (
                llm_input_message_content_key(8),
                "llm.input_messages.8.message.content",
            ),
            (
                llm_input_message_name_key(9),
                "llm.input_messages.9.message.name",
            ),
            (
                llm_input_message_tool_call_id_key(10),
                "llm.input_messages.10.message.tool_call_id",
            ),
            (
                llm_output_message_role_key(11),
                "llm.output_messages.11.message.role",
            ),
            (
                llm_output_message_content_key(12),
                "llm.output_messages.12.message.content",
            ),
            (llm_prompt_key(13), "llm.prompts.13.prompt.text"),
            (llm_choice_key(14), "llm.choices.14.completion.text"),
            (llm_tool_key(15), "llm.tools.15.tool.json_schema"),
            (
                retrieval_document_key(16, DOCUMENT_CONTENT),
                "retrieval.documents.16.document.content",
            ),
            (
                embedding_key(17, EMBEDDING_TEXT),
                "embedding.embeddings.17.embedding.text",
            ),
            (input_image_key(18, IMAGE_URL), "input.images.18.image.url"),
            (
                output_image_key(19, IMAGE_URL),
                "output.images.19.image.url",
            ),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }
}
