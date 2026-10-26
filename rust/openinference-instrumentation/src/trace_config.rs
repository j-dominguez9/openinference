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

    match value.trim().parse::<usize>() {
        Ok(n) => n,
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
        // Python's int() ignores surrounding whitespace; anything that
        // isn't a non-negative integer falls back to the default.
        let cases = [
            ("100", 100),
            (" 100 ", 100),
            ("0", 0),
            ("abc", 32_000),
            ("-5", 32_000),
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
}
