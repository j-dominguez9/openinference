use openinference_semantic_conventions::attributes::{
    DOCUMENT_CONTENT, DOCUMENT_METADATA, EMBEDDING_EMBEDDINGS, EMBEDDING_TEXT, EMBEDDING_VECTOR,
    IMAGE_URL, INPUT_IMAGES, INPUT_MIME_TYPE, INPUT_VALUE, LLM_CHOICES, LLM_INPUT_MESSAGES,
    LLM_INVOCATION_PARAMETERS, LLM_OUTPUT_MESSAGES, LLM_PROMPTS, LLM_TOOLS, MESSAGE_CONTENT,
    MESSAGE_CONTENT_IMAGE, MESSAGE_CONTENT_TEXT, MESSAGE_CONTENTS, OUTPUT_IMAGES, OUTPUT_MIME_TYPE,
    OUTPUT_VALUE, RERANKER_INPUT_DOCUMENTS, RERANKER_OUTPUT_DOCUMENTS, RERANKER_QUERY,
    RETRIEVAL_DOCUMENTS,
};
use opentelemetry::Value;

/// Replaces the value of an attribute that trace configuration hides.
pub const REDACTED_VALUE: &str = "__REDACTED__";

// Environment variables read by `TraceConfig::from_env`.

/// Sets [`TraceConfig::hide_llm_invocation_parameters`].
pub const ENV_HIDE_LLM_INVOCATION_PARAMETERS: &str = "OPENINFERENCE_HIDE_LLM_INVOCATION_PARAMETERS";
/// Sets [`TraceConfig::hide_llm_tools`].
pub const ENV_HIDE_LLM_TOOLS: &str = "OPENINFERENCE_HIDE_LLM_TOOLS";
/// Sets [`TraceConfig::hide_inputs`].
pub const ENV_HIDE_INPUTS: &str = "OPENINFERENCE_HIDE_INPUTS";
/// Sets [`TraceConfig::hide_outputs`].
pub const ENV_HIDE_OUTPUTS: &str = "OPENINFERENCE_HIDE_OUTPUTS";
/// Sets [`TraceConfig::hide_input_messages`].
pub const ENV_HIDE_INPUT_MESSAGES: &str = "OPENINFERENCE_HIDE_INPUT_MESSAGES";
/// Sets [`TraceConfig::hide_output_messages`].
pub const ENV_HIDE_OUTPUT_MESSAGES: &str = "OPENINFERENCE_HIDE_OUTPUT_MESSAGES";
/// Sets [`TraceConfig::hide_input_images`].
pub const ENV_HIDE_INPUT_IMAGES: &str = "OPENINFERENCE_HIDE_INPUT_IMAGES";
/// Sets [`TraceConfig::hide_input_text`].
pub const ENV_HIDE_INPUT_TEXT: &str = "OPENINFERENCE_HIDE_INPUT_TEXT";
/// Sets [`TraceConfig::hide_output_text`].
pub const ENV_HIDE_OUTPUT_TEXT: &str = "OPENINFERENCE_HIDE_OUTPUT_TEXT";
/// Sets [`TraceConfig::hide_embeddings_vectors`].
#[deprecated(note = "use `ENV_HIDE_EMBEDDINGS_VECTORS` instead")]
pub const ENV_HIDE_EMBEDDING_VECTORS: &str = "OPENINFERENCE_HIDE_EMBEDDING_VECTORS";
/// Sets [`TraceConfig::hide_embeddings_vectors`].
pub const ENV_HIDE_EMBEDDINGS_VECTORS: &str = "OPENINFERENCE_HIDE_EMBEDDINGS_VECTORS";
/// Sets [`TraceConfig::hide_embeddings_text`].
pub const ENV_HIDE_EMBEDDINGS_TEXT: &str = "OPENINFERENCE_HIDE_EMBEDDINGS_TEXT";
/// Sets [`TraceConfig::hide_prompts`].
pub const ENV_HIDE_PROMPTS: &str = "OPENINFERENCE_HIDE_PROMPTS";
/// Sets [`TraceConfig::hide_choices`].
pub const ENV_HIDE_CHOICES: &str = "OPENINFERENCE_HIDE_CHOICES";
/// Sets [`TraceConfig::hide_retrieval_documents`].
pub const ENV_HIDE_RETRIEVAL_DOCUMENTS: &str = "OPENINFERENCE_HIDE_RETRIEVAL_DOCUMENTS";
/// Sets [`TraceConfig::enable_genai_semconv`].
pub const ENV_ENABLE_GENAI_SEMCONV: &str = "OPENINFERENCE_ENABLE_GENAI_SEMCONV";
/// Sets [`TraceConfig::base64_image_max_length`].
pub const ENV_BASE64_IMAGE_MAX_LENGTH: &str = "OPENINFERENCE_BASE64_IMAGE_MAX_LENGTH";
/// Names the uploader for oversized base64 images. Not supported yet.
pub const ENV_BLOB_UPLOADER: &str = "OPENINFERENCE_BLOB_UPLOADER";

/// Controls which OpenInference attributes are recorded, so sensitive data
/// can be kept out of traces. The default records everything.
///
/// [`from_env`](Self::from_env) reads the `OPENINFERENCE_*` environment
/// variables. To override single settings in code, use struct update syntax;
/// the fields you set win over the environment.
///
/// # Examples
///
/// ```
/// use openinference_instrumentation::trace_config::TraceConfig;
///
/// let config = TraceConfig {
///     hide_inputs: true,
///     ..TraceConfig::from_env()
/// };
/// assert!(config.hide_inputs);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TraceConfig {
    /// Hides the LLM invocation parameters.
    pub hide_llm_invocation_parameters: bool,
    /// Hides the tool definitions sent to the LLM.
    pub hide_llm_tools: bool,
    /// Hides all input data: values, messages, prompts, tools and images.
    pub hide_inputs: bool,
    /// Hides all output data: values, messages, choices, images and retrieved documents.
    pub hide_outputs: bool,
    /// Hides input messages.
    pub hide_input_messages: bool,
    /// Hides output messages.
    pub hide_output_messages: bool,
    /// Hides input images.
    pub hide_input_images: bool,
    /// Hides the text of input messages, keeping their structure.
    pub hide_input_text: bool,
    /// Hides the text of output messages, keeping their structure.
    pub hide_output_text: bool,
    /// Hides embedding vectors.
    pub hide_embeddings_vectors: bool,
    /// Hides embedding text.
    pub hide_embeddings_text: bool,
    /// Hides completions API prompts.
    pub hide_prompts: bool,
    /// Hides completions API choices.
    pub hide_choices: bool,
    /// Hides the content of retrieved documents.
    pub hide_retrieval_documents: bool,
    /// Also emits OpenTelemetry GenAI semantic conventions.
    pub enable_genai_semconv: bool,
    /// Longest base64-encoded image, in characters, kept on a span.
    pub base64_image_max_length: usize,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            hide_llm_invocation_parameters: false,
            hide_llm_tools: false,
            hide_inputs: false,
            hide_outputs: false,
            hide_input_messages: false,
            hide_output_messages: false,
            hide_input_images: false,
            hide_input_text: false,
            hide_output_text: false,
            hide_embeddings_vectors: false,
            hide_embeddings_text: false,
            hide_prompts: false,
            hide_choices: false,
            hide_retrieval_documents: false,
            enable_genai_semconv: false,
            base64_image_max_length: 32_000,
        }
    }
}

impl TraceConfig {
    /// Reads each setting from its environment variable, falling back to the
    /// default when the variable is unset or invalid. Parsing matches the
    /// Python package.
    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    // Takes the environment as a function so tests can pass a fake one;
    // `std::env::set_var` is unsafe while other test threads run.
    #[allow(deprecated)]
    fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let d = Self::default();
        Self {
            hide_llm_invocation_parameters: env_bool(
                &lookup,
                ENV_HIDE_LLM_INVOCATION_PARAMETERS,
                d.hide_llm_invocation_parameters,
            ),
            hide_llm_tools: env_bool(&lookup, ENV_HIDE_LLM_TOOLS, d.hide_llm_tools),
            hide_inputs: env_bool(&lookup, ENV_HIDE_INPUTS, d.hide_inputs),
            hide_outputs: env_bool(&lookup, ENV_HIDE_OUTPUTS, d.hide_outputs),
            hide_input_messages: env_bool(&lookup, ENV_HIDE_INPUT_MESSAGES, d.hide_input_messages),
            hide_output_messages: env_bool(
                &lookup,
                ENV_HIDE_OUTPUT_MESSAGES,
                d.hide_output_messages,
            ),
            hide_input_images: env_bool(&lookup, ENV_HIDE_INPUT_IMAGES, d.hide_input_images),
            hide_input_text: env_bool(&lookup, ENV_HIDE_INPUT_TEXT, d.hide_input_text),
            hide_output_text: env_bool(&lookup, ENV_HIDE_OUTPUT_TEXT, d.hide_output_text),
            hide_embeddings_vectors: env_bool(
                &lookup,
                ENV_HIDE_EMBEDDINGS_VECTORS,
                d.hide_embeddings_vectors,
            ) || env_bool(&lookup, ENV_HIDE_EMBEDDING_VECTORS, false),
            hide_embeddings_text: env_bool(
                &lookup,
                ENV_HIDE_EMBEDDINGS_TEXT,
                d.hide_embeddings_text,
            ),
            hide_prompts: env_bool(&lookup, ENV_HIDE_PROMPTS, d.hide_prompts),
            hide_choices: env_bool(&lookup, ENV_HIDE_CHOICES, d.hide_choices),
            hide_retrieval_documents: env_bool(
                &lookup,
                ENV_HIDE_RETRIEVAL_DOCUMENTS,
                d.hide_retrieval_documents,
            ),
            enable_genai_semconv: env_bool(
                &lookup,
                ENV_ENABLE_GENAI_SEMCONV,
                d.enable_genai_semconv,
            ),
            base64_image_max_length: env_usize(
                &lookup,
                ENV_BASE64_IMAGE_MAX_LENGTH,
                d.base64_image_max_length,
            ),
        }
    }
}

impl TraceConfig {
    /// Returns what to record for the attribute `key`: `value` unchanged,
    /// [`REDACTED_VALUE`] in its place, or `None` to drop the attribute.
    ///
    /// The rules and their order follow Python's `TraceConfig.mask`; the
    /// first rule that matches wins.
    pub fn mask(&self, key: &str, value: Value) -> Option<Value> {
        let redacted = || Some(Value::from(REDACTED_VALUE));

        if self.hide_llm_invocation_parameters && key == LLM_INVOCATION_PARAMETERS {
            return None;
        }

        if self.hides_llm_tools(key) {
            return None;
        }

        if self.hide_inputs && key == INPUT_VALUE {
            return redacted();
        }

        if self.hide_inputs && key == INPUT_MIME_TYPE {
            return None;
        }

        if self.hide_inputs && key == RERANKER_QUERY {
            return redacted();
        }

        if self.hide_inputs && is_within(key, RERANKER_INPUT_DOCUMENTS) {
            return None;
        }

        if self.hide_outputs && key == OUTPUT_VALUE {
            return redacted();
        }

        if self.hide_outputs && key == OUTPUT_MIME_TYPE {
            return None;
        }

        if self.hide_outputs && is_within(key, RERANKER_OUTPUT_DOCUMENTS) {
            return None;
        }

        if self.hides_retrieval_documents(key) {
            return redacted();
        }

        if self.hides_input_messages(key) {
            return None;
        }

        if self.hides_prompts(key) {
            return redacted();
        }

        if self.hides_output_messages(key) {
            return None;
        }

        if self.hides_choices(key) {
            return redacted();
        }

        if self.hides_input_text(key) {
            return redacted();
        }

        if self.hides_output_text(key) {
            return redacted();
        }

        if (self.hide_inputs || self.hide_input_images) && is_input_image(key) {
            return None;
        }

        if self.hide_outputs && is_output_image(key) {
            return None;
        }

        if is_image_url(key) {
            if let Value::String(s) = &value {
                let url = s.as_str();
                // Count characters, not bytes, to match Python's len().
                if is_base64_url(url) && url.chars().count() > self.base64_image_max_length {
                    // Python can hand oversized images to a blob uploader; Rust has none yet.
                    return redacted();
                }
            }
            return Some(value);
        }

        if self.hide_embeddings_vectors
            && key.contains(EMBEDDING_EMBEDDINGS)
            && key.contains(EMBEDDING_VECTOR)
        {
            return redacted();
        }

        if self.hide_embeddings_text
            && key.contains(EMBEDDING_EMBEDDINGS)
            && key.contains(EMBEDDING_TEXT)
        {
            return redacted();
        }

        Some(value)
    }

    // Python matches these prefixes with `in`, so any key containing one
    // counts.

    fn hides_llm_tools(&self, key: &str) -> bool {
        (self.hide_inputs || self.hide_llm_tools) && key.contains(LLM_TOOLS)
    }

    fn hides_input_messages(&self, key: &str) -> bool {
        (self.hide_inputs || self.hide_input_messages) && key.contains(LLM_INPUT_MESSAGES)
    }

    fn hides_prompts(&self, key: &str) -> bool {
        (self.hide_inputs || self.hide_prompts) && key.contains(LLM_PROMPTS)
    }

    fn hides_output_messages(&self, key: &str) -> bool {
        (self.hide_outputs || self.hide_output_messages) && key.contains(LLM_OUTPUT_MESSAGES)
    }

    fn hides_choices(&self, key: &str) -> bool {
        (self.hide_outputs || self.hide_choices) && key.contains(LLM_CHOICES)
    }

    fn hides_retrieval_documents(&self, key: &str) -> bool {
        (self.hide_outputs || self.hide_retrieval_documents)
            && is_within(key, RETRIEVAL_DOCUMENTS)
            && (key.ends_with(DOCUMENT_CONTENT) || key.ends_with(DOCUMENT_METADATA))
    }

    fn hides_input_text(&self, key: &str) -> bool {
        self.hide_input_text
            && key.contains(LLM_INPUT_MESSAGES)
            && ((key.contains(MESSAGE_CONTENT) && !key.contains(MESSAGE_CONTENTS))
                || key.contains(MESSAGE_CONTENT_TEXT))
    }

    fn hides_output_text(&self, key: &str) -> bool {
        self.hide_output_text
            && key.contains(LLM_OUTPUT_MESSAGES)
            && ((key.contains(MESSAGE_CONTENT) && !key.contains(MESSAGE_CONTENTS))
                || key.contains(MESSAGE_CONTENT_TEXT))
    }
}

/// True for `namespace` itself and any key indexed under it, such as
/// `reranker.input_documents.0.document.id`.
fn is_within(key: &str, namespace: &str) -> bool {
    key.strip_prefix(namespace)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// True for an image in an input message or under `input.images`.
fn is_input_image(key: &str) -> bool {
    (key.contains(LLM_INPUT_MESSAGES) && key.contains(MESSAGE_CONTENT_IMAGE))
        || is_within(key, INPUT_IMAGES)
}

/// True for an image in an output message or under `output.images`.
fn is_output_image(key: &str) -> bool {
    (key.contains(LLM_OUTPUT_MESSAGES) && key.contains(MESSAGE_CONTENT_IMAGE))
        || is_within(key, OUTPUT_IMAGES)
}

fn is_image_url(key: &str) -> bool {
    key.ends_with(IMAGE_URL) && (is_input_image(key) || is_output_image(key))
}

/// True for a base64-encoded image data URL.
fn is_base64_url(url: &str) -> bool {
    url.starts_with("data:image/") && url.contains("base64")
}

fn env_bool(lookup: &impl Fn(&str) -> Option<String>, name: &str, default: bool) -> bool {
    let Some(value) = lookup(name) else {
        return default;
    };

    match value.to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => default,
    }
}

fn env_usize(lookup: &impl Fn(&str) -> Option<String>, name: &str, default: usize) -> usize {
    let Some(value) = lookup(name) else {
        return default;
    };

    // Python's int() accepts negatives, and a negative maximum redacts every
    // image; 0 does the same here, since an empty string is never base64.
    // Python also accepts `1_000`; Rust doesn't, and falls back to the default.
    match value.trim().parse::<i64>() {
        Ok(n) => usize::try_from(n).unwrap_or(0),
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Builds a config from a fake environment holding only `vars`.
    fn config_with(vars: &[(&str, &str)]) -> TraceConfig {
        let env: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        TraceConfig::from_lookup(|name| env.get(name).cloned())
    }

    #[test]
    fn default_matches_python_defaults() {
        let d = TraceConfig::default();
        assert!(!d.hide_inputs);
        assert!(!d.enable_genai_semconv);
        assert_eq!(d.base64_image_max_length, 32_000);
    }

    #[test]
    fn empty_env_gives_defaults() {
        assert_eq!(config_with(&[]), TraceConfig::default());
    }

    /// Reads one boolean field, so a test table can name fields.
    type Field = fn(&TraceConfig) -> bool;

    // Each env var must set its own field and no other.
    #[test]
    fn each_bool_env_var_sets_its_field() {
        let cases: [(&str, Field); 15] = [
            (ENV_HIDE_LLM_INVOCATION_PARAMETERS, |c| {
                c.hide_llm_invocation_parameters
            }),
            (ENV_HIDE_LLM_TOOLS, |c| c.hide_llm_tools),
            (ENV_HIDE_INPUTS, |c| c.hide_inputs),
            (ENV_HIDE_OUTPUTS, |c| c.hide_outputs),
            (ENV_HIDE_INPUT_MESSAGES, |c| c.hide_input_messages),
            (ENV_HIDE_OUTPUT_MESSAGES, |c| c.hide_output_messages),
            (ENV_HIDE_INPUT_IMAGES, |c| c.hide_input_images),
            (ENV_HIDE_INPUT_TEXT, |c| c.hide_input_text),
            (ENV_HIDE_OUTPUT_TEXT, |c| c.hide_output_text),
            (ENV_HIDE_EMBEDDINGS_VECTORS, |c| c.hide_embeddings_vectors),
            (ENV_HIDE_EMBEDDINGS_TEXT, |c| c.hide_embeddings_text),
            (ENV_HIDE_PROMPTS, |c| c.hide_prompts),
            (ENV_HIDE_CHOICES, |c| c.hide_choices),
            (ENV_HIDE_RETRIEVAL_DOCUMENTS, |c| c.hide_retrieval_documents),
            (ENV_ENABLE_GENAI_SEMCONV, |c| c.enable_genai_semconv),
        ];

        for (name, field) in cases {
            let config = config_with(&[(name, "true")]);
            assert!(field(&config), "{name} did not set its field");
            assert_eq!(
                cases.iter().filter(|(_, f)| f(&config)).count(),
                1,
                "{name} set more than one field"
            );
        }
    }

    #[test]
    fn bool_parsing_matches_python() {
        // Python accepts only "true"/"false" in any case; anything else
        // falls back to the default.
        let cases = [
            ("true", true),
            ("TRUE", true),
            ("True", true),
            ("false", false),
            ("FALSE", false),
            ("1", false),
            ("yes", false),
            (" true", false),
            ("", false),
        ];

        for (value, expected) in cases {
            let config = config_with(&[(ENV_HIDE_INPUTS, value)]);
            assert_eq!(config.hide_inputs, expected, "parsed {value:?} wrong");
        }
    }

    #[test]
    fn invalid_bool_keeps_a_true_default() {
        let lookup = |_: &str| Some("not-a-bool".to_string());
        assert!(env_bool(&lookup, ENV_HIDE_INPUTS, true));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_embedding_vectors_var_still_hides() {
        let config = config_with(&[(ENV_HIDE_EMBEDDING_VECTORS, "true")]);
        assert!(config.hide_embeddings_vectors);

        // Python ORs the two flags, so either one being true hides vectors.
        let config = config_with(&[
            (ENV_HIDE_EMBEDDINGS_VECTORS, "false"),
            (ENV_HIDE_EMBEDDING_VECTORS, "true"),
        ]);
        assert!(config.hide_embeddings_vectors);
    }

    #[test]
    fn base64_image_max_length_parsing_matches_python() {
        // Python's int() ignores surrounding whitespace, and a negative value
        // redacts every image, which 0 also does. Anything that isn't an
        // integer falls back to the default.
        let cases = [
            ("100", 100),
            (" 100 ", 100),
            ("0", 0),
            ("abc", 32_000),
            ("-5", 0),
            ("+5", 5),
            ("1.5", 32_000),
            ("", 32_000),
        ];

        for (value, expected) in cases {
            let config = config_with(&[(ENV_BASE64_IMAGE_MAX_LENGTH, value)]);
            assert_eq!(
                config.base64_image_max_length, expected,
                "parsed {value:?} wrong"
            );
        }
    }

    fn text(s: &'static str) -> Value {
        Value::from(s)
    }

    #[test]
    fn default_config_keeps_every_attribute() {
        let config = TraceConfig::default();
        for key in [LLM_INVOCATION_PARAMETERS, INPUT_VALUE, INPUT_MIME_TYPE] {
            assert_eq!(
                config.mask(key, text("x")),
                Some(text("x")),
                "{key} changed"
            );
        }
    }

    #[test]
    fn hide_llm_invocation_parameters_drops_them() {
        let config = TraceConfig {
            hide_llm_invocation_parameters: true,
            ..TraceConfig::default()
        };
        assert_eq!(config.mask(LLM_INVOCATION_PARAMETERS, text("{}")), None);
        assert_eq!(config.mask(INPUT_VALUE, text("hi")), Some(text("hi")));
    }

    #[test]
    fn hide_inputs_redacts_value_and_drops_mime_type_and_messages() {
        let config = TraceConfig {
            hide_inputs: true,
            ..TraceConfig::default()
        };
        assert_eq!(
            config.mask(INPUT_VALUE, text("hi")),
            Some(text(REDACTED_VALUE))
        );
        assert_eq!(config.mask(INPUT_MIME_TYPE, text("text/plain")), None);
        assert_eq!(
            config.mask("llm.input_messages.0.message.content", text("hi")),
            None
        );
    }

    #[test]
    fn hide_input_messages_keeps_input_value() {
        let config = TraceConfig {
            hide_input_messages: true,
            ..TraceConfig::default()
        };
        assert_eq!(
            config.mask("llm.input_messages.0.message.role", text("user")),
            None
        );
        assert_eq!(config.mask(INPUT_VALUE, text("hi")), Some(text("hi")));
    }

    /// Turns on one flag, so a test table can name flags.
    type Flag = fn(&mut TraceConfig);

    #[derive(Debug, Clone, Copy)]
    enum Expect {
        Keep,
        Redact,
        Drop,
    }

    #[test]
    fn mask_rules_match_python() {
        use Expect::{Drop, Keep, Redact};

        let cases: [(&str, Flag, &str, Expect); 42] = [
            (
                "tools",
                |c| c.hide_llm_tools = true,
                "llm.tools.0.tool.json_schema",
                Drop,
            ),
            (
                "tools",
                |c| c.hide_inputs = true,
                "llm.tools.0.tool.json_schema",
                Drop,
            ),
            ("tools", |c| c.hide_llm_tools = true, INPUT_VALUE, Keep),
            ("reranker", |c| c.hide_inputs = true, RERANKER_QUERY, Redact),
            (
                "reranker",
                |c| c.hide_inputs = true,
                "reranker.input_documents.0.document.id",
                Drop,
            ),
            (
                "reranker",
                |c| c.hide_inputs = true,
                RERANKER_INPUT_DOCUMENTS,
                Drop,
            ),
            // is_within needs a `.` after the namespace, not just a shared prefix.
            (
                "reranker",
                |c| c.hide_inputs = true,
                "reranker.input_documents_count",
                Keep,
            ),
            ("outputs", |c| c.hide_outputs = true, OUTPUT_VALUE, Redact),
            ("outputs", |c| c.hide_outputs = true, OUTPUT_MIME_TYPE, Drop),
            (
                "outputs",
                |c| c.hide_outputs = true,
                "reranker.output_documents.1.document.score",
                Drop,
            ),
            (
                "retrieval",
                |c| c.hide_retrieval_documents = true,
                "retrieval.documents.0.document.content",
                Redact,
            ),
            (
                "retrieval",
                |c| c.hide_retrieval_documents = true,
                "retrieval.documents.0.document.metadata",
                Redact,
            ),
            (
                "retrieval",
                |c| c.hide_retrieval_documents = true,
                "retrieval.documents.0.document.id",
                Keep,
            ),
            (
                "retrieval",
                |c| c.hide_outputs = true,
                "retrieval.documents.0.document.content",
                Redact,
            ),
            (
                "prompts",
                |c| c.hide_prompts = true,
                "llm.prompts.0.prompt.text",
                Redact,
            ),
            (
                "prompts",
                |c| c.hide_inputs = true,
                "llm.prompts.0.prompt.text",
                Redact,
            ),
            (
                "output messages",
                |c| c.hide_output_messages = true,
                "llm.output_messages.0.message.content",
                Drop,
            ),
            (
                "output messages",
                |c| c.hide_outputs = true,
                "llm.output_messages.0.message.role",
                Drop,
            ),
            (
                "output messages",
                |c| c.hide_output_messages = true,
                OUTPUT_VALUE,
                Keep,
            ),
            (
                "choices",
                |c| c.hide_choices = true,
                "llm.choices.0.completion.text",
                Redact,
            ),
            (
                "choices",
                |c| c.hide_outputs = true,
                "llm.choices.0.completion.text",
                Redact,
            ),
            ("choices", |c| c.hide_choices = true, OUTPUT_VALUE, Keep),
            (
                "input text",
                |c| c.hide_input_text = true,
                "llm.input_messages.0.message.content",
                Redact,
            ),
            (
                "input text",
                |c| c.hide_input_text = true,
                "llm.input_messages.0.message.contents.0.message_content.text",
                Redact,
            ),
            (
                "input text",
                |c| c.hide_input_text = true,
                "llm.input_messages.0.message.contents.0.message_content.image.image.url",
                Keep,
            ),
            (
                "input text",
                |c| c.hide_input_text = true,
                "llm.input_messages.0.message.role",
                Keep,
            ),
            (
                "output text",
                |c| c.hide_output_text = true,
                "llm.output_messages.0.message.content",
                Redact,
            ),
            (
                "output text",
                |c| c.hide_output_text = true,
                "llm.output_messages.0.message.contents.0.message_content.text",
                Redact,
            ),
            (
                "output text",
                |c| c.hide_output_text = true,
                "llm.output_messages.0.message.contents.0.message_content.image.image.url",
                Keep,
            ),
            (
                "output text",
                |c| c.hide_output_text = true,
                "llm.output_messages.0.message.role",
                Keep,
            ),
            (
                "input images",
                |c| c.hide_input_images = true,
                "llm.input_messages.0.message.contents.1.message_content.image.image.url",
                Drop,
            ),
            (
                "input images",
                |c| c.hide_input_images = true,
                "input.images.0.image.url",
                Drop,
            ),
            (
                "input images",
                |c| c.hide_input_images = true,
                "llm.input_messages.0.message.contents.0.message_content.text",
                Keep,
            ),
            (
                "input images",
                |c| c.hide_input_images = true,
                "llm.output_messages.0.message.contents.0.message_content.image.image.url",
                Keep,
            ),
            // Only the image rule catches span-level input images under hide_inputs.
            (
                "input images",
                |c| c.hide_inputs = true,
                "input.images.0.image.url",
                Drop,
            ),
            (
                "output images",
                |c| c.hide_outputs = true,
                "output.images.0.image.url",
                Drop,
            ),
            (
                "output images",
                |c| c.hide_outputs = true,
                "llm.output_messages.0.message.contents.0.message_content.image.image.url",
                Drop,
            ),
            (
                "embeddings",
                |c| c.hide_embeddings_vectors = true,
                "embedding.embeddings.0.embedding.vector",
                Redact,
            ),
            (
                "embeddings",
                |c| c.hide_embeddings_vectors = true,
                "embedding.embeddings.0.embedding.text",
                Keep,
            ),
            (
                "embeddings",
                |c| c.hide_embeddings_text = true,
                "embedding.embeddings.0.embedding.text",
                Redact,
            ),
            (
                "embeddings",
                |c| c.hide_embeddings_text = true,
                "embedding.embeddings.0.embedding.vector",
                Keep,
            ),
            (
                "embeddings",
                |c| c.hide_embeddings_text = true,
                "embedding.model_name",
                Keep,
            ),
        ];

        for (rule, set_flag, key, expect) in cases {
            // With no flags set, every key passes through unchanged.
            assert_eq!(
                TraceConfig::default().mask(key, text("x")),
                Some(text("x")),
                "default config changed {key}"
            );

            let mut config = TraceConfig::default();
            set_flag(&mut config);
            let expected = match expect {
                Keep => Some(text("x")),
                Redact => Some(text(REDACTED_VALUE)),
                Drop => None,
            };
            assert_eq!(
                config.mask(key, text("x")),
                expected,
                "{rule}: {key} should {expect:?}"
            );
        }
    }

    #[test]
    fn oversized_base64_images_are_redacted() {
        let config = TraceConfig {
            base64_image_max_length: 30,
            ..TraceConfig::default()
        };
        let long = "A".repeat(40);
        let image_key = "input.images.0.image.url";

        let cases = [
            (image_key, format!("data:image/png;base64,{long}"), true),
            (image_key, "data:image/png;base64,AAAA".to_string(), false),
            (image_key, format!("https://example.com/{long}.png"), false),
            // Each condition of is_base64_url must hold on its own.
            (image_key, format!("data:text/plain;base64,{long}"), false),
            (image_key, format!("data:image/png,{long}"), false),
            // Only image URL keys are checked.
            (INPUT_VALUE, format!("data:image/png;base64,{long}"), false),
            (
                "llm.output_messages.0.message.contents.0.message_content.image.image.url",
                format!("data:image/png;base64,{long}"),
                true,
            ),
        ];

        for (key, url, redact) in cases {
            let value = Value::from(url.clone());
            let expected = if redact {
                Value::from(REDACTED_VALUE)
            } else {
                value.clone()
            };
            assert_eq!(config.mask(key, value), Some(expected), "{key} = {url}");
        }
    }

    #[test]
    fn negative_max_length_redacts_every_base64_image() {
        let config = config_with(&[(ENV_BASE64_IMAGE_MAX_LENGTH, "-1")]);
        let tiny = Value::from("data:image/png;base64,A");
        assert_eq!(
            config.mask("input.images.0.image.url", tiny),
            Some(Value::from(REDACTED_VALUE))
        );
    }

    #[test]
    fn image_length_counts_characters_not_bytes() {
        // 25 characters but 26 bytes, because `é` takes two bytes in UTF-8.
        let url = "data:image/é;base64,AAAAA";
        assert_eq!(url.chars().count(), 25);
        let config = TraceConfig {
            base64_image_max_length: 25,
            ..TraceConfig::default()
        };
        assert_eq!(
            config.mask("input.images.0.image.url", Value::from(url)),
            Some(Value::from(url))
        );
    }
}
