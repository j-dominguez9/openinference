// Feedback collections. These are key prefixes: each entry is flattened with
// an index, e.g. `annotations.0.annotation.name`.

/// Span-scoped feedback using annotation terminology, flattened as
/// `annotations.<i>.annotation.*`.
pub const ANNOTATIONS: &str = "annotations";
/// Span-scoped feedback using evaluation terminology, flattened as
/// `evaluations.<i>.evaluation.*`.
pub const EVALUATIONS: &str = "evaluations";
/// Trace-scoped feedback using annotation terminology, flattened as
/// `trace.annotations.<i>.annotation.*`.
pub const TRACE_ANNOTATIONS: &str = "trace.annotations";
/// Trace-scoped feedback using evaluation terminology, flattened as
/// `trace.evaluations.<i>.evaluation.*`.
pub const TRACE_EVALUATIONS: &str = "trace.evaluations";
/// Session-scoped feedback using annotation terminology, flattened as
/// `session.annotations.<i>.annotation.*`. Requires [`SESSION_ID`].
pub const SESSION_ANNOTATIONS: &str = "session.annotations";
/// Session-scoped feedback using evaluation terminology, flattened as
/// `session.evaluations.<i>.evaluation.*`. Requires [`SESSION_ID`].
pub const SESSION_EVALUATIONS: &str = "session.evaluations";

// Span-level attributes.

/// The kind of operation the span represents (LLM, CHAIN, TOOL, etc.).
///
/// Set it to a [`SpanKind`](crate::enums::SpanKind) value.
pub const OPENINFERENCE_SPAN_KIND: &str = "openinference.span.kind";

/// The input to the operation.
///
/// A plain string by default. If the value is a JSON string, set
/// [`INPUT_MIME_TYPE`] to `application/json`.
pub const INPUT_VALUE: &str = "input.value";
/// The MIME type of [`INPUT_VALUE`].
///
/// Set it to a [`MimeType`](crate::enums::MimeType) value.
pub const INPUT_MIME_TYPE: &str = "input.mime_type";
/// Prefix for the span-kind-independent list of input images, flattened with
/// an index, e.g. `input.images.0.image.url`.
pub const INPUT_IMAGES: &str = "input.images";

/// The output of the operation.
///
/// A plain string by default. If the value is a JSON string, set
/// [`OUTPUT_MIME_TYPE`] to `application/json`.
pub const OUTPUT_VALUE: &str = "output.value";
/// The MIME type of [`OUTPUT_VALUE`].
///
/// Set it to a [`MimeType`](crate::enums::MimeType) value.
pub const OUTPUT_MIME_TYPE: &str = "output.mime_type";
/// Prefix for the span-kind-independent list of output images, flattened with
/// an index, e.g. `output.images.0.image.url`.
pub const OUTPUT_IMAGES: &str = "output.images";

/// A JSON-encoded map of user-defined key-value pairs.
pub const METADATA: &str = "metadata";

/// A list of custom categorical tags.
pub const TAG_TAGS: &str = "tag.tags";

/// Groups spans into a multi-turn conversation.
pub const SESSION_ID: &str = "session.id";

/// Identifies the user the span ran on behalf of.
pub const USER_ID: &str = "user.id";

/// The name of the agent.
///
/// Agents performing the same function should share a name.
pub const AGENT_NAME: &str = "agent.name";

/// The id of a node in the agent's execution graph.
///
/// Combined with [`GRAPH_NODE_PARENT_ID`], this lets the Arize UI render the
/// agent's execution graph.
pub const GRAPH_NODE_ID: &str = "graph.node.id";
/// A human-readable name for the node in the execution graph.
pub const GRAPH_NODE_NAME: &str = "graph.node.name";
/// The id of the parent node in the execution graph.
///
/// Leaving it unset or empty marks the span as the root node.
pub const GRAPH_NODE_PARENT_ID: &str = "graph.node.parent_id";

// Feedback object attributes, nested under ANNOTATIONS, EVALUATIONS and their
// trace- and session-scoped variants.

/// The criterion or metric name of an annotation.
pub const ANNOTATION_NAME: &str = "annotation.name";
/// The numeric result of an annotation.
pub const ANNOTATION_SCORE: &str = "annotation.score";
/// The categorical result of an annotation.
pub const ANNOTATION_LABEL: &str = "annotation.label";
/// The reason or evidence for an annotation's result.
pub const ANNOTATION_EXPLANATION: &str = "annotation.explanation";
/// The kind of judge that produced an annotation.
///
/// Set it to an [`AnnotatorKind`](crate::enums::AnnotatorKind) value, or a
/// custom value.
pub const ANNOTATION_ANNOTATOR_KIND: &str = "annotation.annotator_kind";
/// A stable result ID for the same name and target.
pub const ANNOTATION_IDENTIFIER: &str = "annotation.identifier";
/// Additional result or judge data, as a JSON object string.
pub const ANNOTATION_METADATA: &str = "annotation.metadata";

/// The criterion or metric name of an evaluation.
pub const EVALUATION_NAME: &str = "evaluation.name";
/// The numeric result of an evaluation.
pub const EVALUATION_SCORE: &str = "evaluation.score";
/// The categorical result of an evaluation.
pub const EVALUATION_LABEL: &str = "evaluation.label";
/// The reason or evidence for an evaluation's result.
pub const EVALUATION_EXPLANATION: &str = "evaluation.explanation";
/// The kind of judge that produced an evaluation.
///
/// Set it to an [`AnnotatorKind`](crate::enums::AnnotatorKind) value, or a
/// custom value.
pub const EVALUATION_ANNOTATOR_KIND: &str = "evaluation.annotator_kind";
/// A stable result ID for the same name and target.
pub const EVALUATION_IDENTIFIER: &str = "evaluation.identifier";
/// Additional result or judge data, as a JSON object string.
pub const EVALUATION_METADATA: &str = "evaluation.metadata";

// LLM-span attributes, set when the span represents an LLM API call.

/// The name of the language model being used.
pub const LLM_MODEL_NAME: &str = "llm.model_name";

/// The model requested by the caller, as sent in the request.
///
/// May differ from [`LLM_RESPONSE_MODEL_NAME`] when the provider routes the
/// request to a different model (e.g. classifier-triggered fallback).
pub const LLM_REQUEST_MODEL_NAME: &str = "llm.request.model_name";

/// The model that actually generated the response, as reported by the
/// provider.
///
/// May differ from [`LLM_REQUEST_MODEL_NAME`] when the provider routes the
/// request to a different model.
pub const LLM_RESPONSE_MODEL_NAME: &str = "llm.response.model_name";

/// The hosting provider of the model, e.g. `azure`.
///
/// Use an [`LlmProvider`](crate::enums::LlmProvider) value when one applies.
pub const LLM_PROVIDER: &str = "llm.provider";
/// The AI product as identified by the client or server instrumentation.
///
/// Use an [`LlmSystem`](crate::enums::LlmSystem) value when one applies.
pub const LLM_SYSTEM: &str = "llm.system";
/// The parameters used to invoke the model, as a JSON string.
pub const LLM_INVOCATION_PARAMETERS: &str = "llm.invocation_parameters";

/// Details of a function call made by the model, as a JSON string.
///
/// For models and APIs that support function calling, e.g. the function name
/// and its arguments.
pub const LLM_FUNCTION_CALL: &str = "llm.function_call";
/// The reason the model stopped generating tokens, e.g. `stop` or `length`.
pub const LLM_FINISH_REASON: &str = "llm.finish_reason";

/// Prefix for the messages sent to a chat API, flattened with an index, e.g.
/// `llm.input_messages.0.message.role`.
///
/// Each message's fields are the `MESSAGE_*` attributes, such as
/// [`MESSAGE_ROLE`].
pub const LLM_INPUT_MESSAGES: &str = "llm.input_messages";
/// Prefix for the messages received from a chat API, flattened with an index,
/// e.g. `llm.output_messages.0.message.role`.
///
/// Each message's fields are the `MESSAGE_*` attributes, such as
/// [`MESSAGE_ROLE`].
pub const LLM_OUTPUT_MESSAGES: &str = "llm.output_messages";

/// Prefix for the prompts sent to a completions API (as opposed to a chat
/// API), flattened with an index, e.g. `llm.prompts.0.prompt.text`.
///
/// See [`PROMPT_TEXT`].
pub const LLM_PROMPTS: &str = "llm.prompts";
/// Prefix for the text choices returned by a completions API, flattened with
/// an index, e.g. `llm.choices.0.completion.text`.
///
/// See [`COMPLETION_TEXT`].
pub const LLM_CHOICES: &str = "llm.choices";

/// The template used to generate the prompt, e.g. using `{variable}`
/// placeholders.
pub const LLM_PROMPT_TEMPLATE: &str = "llm.prompt_template.template";
/// The variables applied to [`LLM_PROMPT_TEMPLATE`], as a JSON string of
/// key-value pairs.
pub const LLM_PROMPT_TEMPLATE_VARIABLES: &str = "llm.prompt_template.variables";
/// The version of [`LLM_PROMPT_TEMPLATE`].
pub const LLM_PROMPT_TEMPLATE_VERSION: &str = "llm.prompt_template.version";

/// Prefix for the tools advertised to the model, flattened with an index, e.g.
/// `llm.tools.0.tool.json_schema`.
///
/// Each tool's fields are [`TOOL_NAME`], [`TOOL_DESCRIPTION`] and
/// [`TOOL_JSON_SCHEMA`].
pub const LLM_TOOLS: &str = "llm.tools";

// Decision-span attributes, set when the span represents a call to a decision
// model: a model that takes state plus typed questions and returns a typed,
// probabilistic answer per question instead of generated text. They mirror the
// `llm.*` identification attributes and share their well-known values. See
// https://github.com/Arize-ai/openinference/blob/main/spec/decision_spans.md.

/// The name of the decision model being used.
///
/// Mirrors [`LLM_MODEL_NAME`] for DECISION spans.
pub const DECISION_MODEL_NAME: &str = "decision.model_name";
/// The decision model requested by the caller, as sent in the request.
///
/// May differ from [`DECISION_RESPONSE_MODEL_NAME`] when the provider resolves
/// an alias (e.g. `jev-latest`) or routes the request to a different model.
pub const DECISION_REQUEST_MODEL_NAME: &str = "decision.request.model_name";
/// The decision model that actually produced the response, as reported by the
/// provider.
///
/// May differ from [`DECISION_REQUEST_MODEL_NAME`].
pub const DECISION_RESPONSE_MODEL_NAME: &str = "decision.response.model_name";

/// Who hosts the decision model that answered, e.g. `typesafe` when calling
/// TypeSafe directly, or the cloud or self-hosting provider for a model served
/// elsewhere.
///
/// Distinct from [`DECISION_SYSTEM`], which names the API shape. Use a
/// [`DecisionProvider`](crate::enums::DecisionProvider) value when one
/// applies.
pub const DECISION_PROVIDER: &str = "decision.provider";
/// The decision API ecosystem the call conforms to, i.e. which request and
/// response shape the client speaks.
///
/// For example, `typesafe` for the TypeSafe System One / Jev API and `openai`
/// for the OpenAI Decisions API. A self-hosted vLLM server answering the
/// Jev-compatible `/v1/systemone` shape is still `typesafe`. Distinct from
/// [`DECISION_PROVIDER`], which says who hosts the model. Use a
/// [`DecisionSystem`](crate::enums::DecisionSystem) value when one applies.
pub const DECISION_SYSTEM: &str = "decision.system";

/// The number of input tokens consumed by a decision model call: the state,
/// questions and candidate options.
///
/// Maps to `usage.input_tokens` in TypeSafe System One responses.
pub const DECISION_TOKEN_COUNT_INPUT: &str = "decision.token_count.input";
/// The number of output tokens produced by a decision model call.
///
/// Decision models emit typed answers rather than text, so this is typically
/// small. Maps to `usage.output_tokens` in TypeSafe System One responses.
pub const DECISION_TOKEN_COUNT_OUTPUT: &str = "decision.token_count.output";

// Token-count attributes for LLM spans. Values are integer token counts.

/// The number of tokens in the prompt.
pub const LLM_TOKEN_COUNT_PROMPT: &str = "llm.token_count.prompt";
/// Prefix for additional prompt token-count details.
///
/// Each detail is a separate attribute under this prefix, e.g.
/// [`LLM_TOKEN_COUNT_PROMPT_DETAILS_AUDIO`].
pub const LLM_TOKEN_COUNT_PROMPT_DETAILS: &str = "llm.token_count.prompt_details";
/// The number of audio tokens in the prompt.
pub const LLM_TOKEN_COUNT_PROMPT_DETAILS_AUDIO: &str = "llm.token_count.prompt_details.audio";
/// The number of input tokens in the prompt that were cached.
pub const LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_INPUT: &str =
    "llm.token_count.prompt_details.cache_input";
/// The number of prompt tokens read from the cache.
pub const LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_READ: &str =
    "llm.token_count.prompt_details.cache_read";
/// The number of prompt tokens written to the cache.
pub const LLM_TOKEN_COUNT_PROMPT_DETAILS_CACHE_WRITE: &str =
    "llm.token_count.prompt_details.cache_write";
/// The number of tokens in the completion.
pub const LLM_TOKEN_COUNT_COMPLETION: &str = "llm.token_count.completion";
/// The number of audio tokens in the completion.
pub const LLM_TOKEN_COUNT_COMPLETION_DETAILS_AUDIO: &str =
    "llm.token_count.completion_details.audio";
/// The number of tokens used for reasoning steps in the completion.
pub const LLM_TOKEN_COUNT_COMPLETION_DETAILS_REASONING: &str =
    "llm.token_count.completion_details.reasoning";
/// The total number of tokens, prompt and completion combined.
pub const LLM_TOKEN_COUNT_TOTAL: &str = "llm.token_count.total";

// Cost attributes for LLM spans. Values are floats in USD.

/// The total cost of all input tokens sent to the model, including system
/// messages, user messages and any other input.
pub const LLM_COST_PROMPT: &str = "llm.cost.prompt";
/// Prefix for additional prompt cost details.
///
/// Each detail is a separate attribute under this prefix, e.g.
/// [`LLM_COST_PROMPT_DETAILS_AUDIO`].
pub const LLM_COST_PROMPT_DETAILS: &str = "llm.cost.prompt_details";
/// The cost of audio tokens in the prompt.
pub const LLM_COST_PROMPT_DETAILS_AUDIO: &str = "llm.cost.prompt_details.audio";
/// The cost of input tokens in the prompt that were cached.
pub const LLM_COST_PROMPT_DETAILS_CACHE_INPUT: &str = "llm.cost.prompt_details.cache_input";
/// The cost of prompt tokens read from the cache.
pub const LLM_COST_PROMPT_DETAILS_CACHE_READ: &str = "llm.cost.prompt_details.cache_read";
/// The cost of prompt tokens written to the cache.
pub const LLM_COST_PROMPT_DETAILS_CACHE_WRITE: &str = "llm.cost.prompt_details.cache_write";
/// The total cost of tokens used as input to the model.
///
/// May differ from [`LLM_COST_PROMPT`] when there are additional processing
/// steps.
pub const LLM_COST_PROMPT_DETAILS_INPUT: &str = "llm.cost.prompt_details.input";
/// The total cost of all output tokens generated by the model, including the
/// main response and any additional output.
pub const LLM_COST_COMPLETION: &str = "llm.cost.completion";
/// Prefix for additional completion cost details.
///
/// Each detail is a separate attribute under this prefix, e.g.
/// [`LLM_COST_COMPLETION_DETAILS_AUDIO`].
pub const LLM_COST_COMPLETION_DETAILS: &str = "llm.cost.completion_details";
/// The cost of audio tokens in the completion.
pub const LLM_COST_COMPLETION_DETAILS_AUDIO: &str = "llm.cost.completion_details.audio";
/// The total cost of tokens generated as output by the model.
///
/// May differ from [`LLM_COST_COMPLETION`] when there are additional
/// processing steps.
pub const LLM_COST_COMPLETION_DETAILS_OUTPUT: &str = "llm.cost.completion_details.output";
/// The cost of reasoning steps in the completion.
pub const LLM_COST_COMPLETION_DETAILS_REASONING: &str = "llm.cost.completion_details.reasoning";
/// The total cost of the call, prompt and completion combined.
pub const LLM_COST_TOTAL: &str = "llm.cost.total";

// Embedding-span attributes, set when the span represents an embedding call.

/// Prefix for the list of embeddings, flattened with an index, e.g.
/// `embedding.embeddings.0.embedding.text`.
///
/// Each embedding's fields are [`EMBEDDING_TEXT`] and [`EMBEDDING_VECTOR`].
pub const EMBEDDING_EMBEDDINGS: &str = "embedding.embeddings";
/// The parameters used to invoke the embedding model, excluding the input, as
/// a JSON string.
pub const EMBEDDING_INVOCATION_PARAMETERS: &str = "embedding.invocation_parameters";
/// The name of the embedding model.
pub const EMBEDDING_MODEL_NAME: &str = "embedding.model_name";
/// The text represented by an embedding.
pub const EMBEDDING_TEXT: &str = "embedding.text";
/// The embedding vector, as a list of floats.
pub const EMBEDDING_VECTOR: &str = "embedding.vector";

// Tool attributes. Used on TOOL spans for the tool being invoked, and under
// LLM_TOOLS for each tool advertised to the model.

/// The name of the tool: the identifier the model uses to call it.
pub const TOOL_NAME: &str = "tool.name";
/// The description of the tool's purpose, which the model uses to decide
/// whether to call it.
pub const TOOL_DESCRIPTION: &str = "tool.description";
/// The parameters definition for invoking the tool, as a JSON string.
pub const TOOL_PARAMETERS: &str = "tool.parameters";
/// The identifier for the result of the tool call, matching [`TOOL_CALL_ID`].
pub const TOOL_ID: &str = "tool.id";
/// The JSON schema of the tool's input.
///
/// The OpenAI tool-calling format is recommended.
pub const TOOL_JSON_SCHEMA: &str = "tool.json_schema";

// Retrieval-span attributes, set when the span represents a retrieval call.

/// Prefix for the retrieved documents, flattened with an index, e.g.
/// `retrieval.documents.0.document.content`.
///
/// Each document's fields are the `DOCUMENT_*` attributes, such as
/// [`DOCUMENT_CONTENT`].
pub const RETRIEVAL_DOCUMENTS: &str = "retrieval.documents";

// Reranker-span attributes, set when the span represents a reranker call.

/// Prefix for the documents passed to the reranker, flattened with an index.
///
/// Each document's fields are the `DOCUMENT_*` attributes.
pub const RERANKER_INPUT_DOCUMENTS: &str = "reranker.input_documents";
/// Prefix for the documents returned by the reranker, flattened with an index.
///
/// Each document's fields are the `DOCUMENT_*` attributes.
pub const RERANKER_OUTPUT_DOCUMENTS: &str = "reranker.output_documents";
/// The query passed to the reranker.
pub const RERANKER_QUERY: &str = "reranker.query";
/// The name of the reranker model.
pub const RERANKER_MODEL_NAME: &str = "reranker.model_name";
/// The top-K parameter of the reranker, as an integer.
pub const RERANKER_TOP_K: &str = "reranker.top_k";

// Message attributes, nested under LLM_INPUT_MESSAGES or LLM_OUTPUT_MESSAGES,
// e.g. `llm.input_messages.0.message.role`.

/// The role of the message's author, e.g. `user`, `system` or `tool`.
pub const MESSAGE_ROLE: &str = "message.role";
/// The content of the message, as a string.
///
/// For multiple content items, or when their order matters, use
/// [`MESSAGE_CONTENTS`] instead.
pub const MESSAGE_CONTENT: &str = "message.content";
/// Prefix for the message's content items, flattened with an index, e.g.
/// `llm.input_messages.0.message.contents.0.message_content.type`.
///
/// Each item's fields are the `MESSAGE_CONTENT_*` attributes. Use this form
/// when a message has multiple content items or when their order matters.
pub const MESSAGE_CONTENTS: &str = "message.contents";
/// The name of the function or tool that produced a `tool` or `function` role
/// message.
pub const MESSAGE_NAME: &str = "message.name";
/// Prefix for the tool calls generated by the model, flattened with an index,
/// e.g. `llm.output_messages.0.message.tool_calls.0.tool_call.id`.
///
/// Each tool call's fields are the `TOOL_CALL_*` attributes.
pub const MESSAGE_TOOL_CALLS: &str = "message.tool_calls";
/// The name of the function called, for the legacy function-calling API.
pub const MESSAGE_FUNCTION_CALL_NAME: &str = "message.function_call_name";
/// The arguments passed to the function, as a JSON string, for the legacy
/// function-calling API.
pub const MESSAGE_FUNCTION_CALL_ARGUMENTS_JSON: &str = "message.function_call_arguments_json";
/// The id of the tool call this message is the result of, matching
/// [`TOOL_CALL_ID`].
pub const MESSAGE_TOOL_CALL_ID: &str = "message.tool_call_id";

// Message-content attributes, nested under MESSAGE_CONTENTS.

/// The type of the content item, e.g. `text`, `image`, `audio`, `video`,
/// `reasoning` or `tool_use`.
pub const MESSAGE_CONTENT_TYPE: &str = "message_content.type";
/// The text of the content item, when [`MESSAGE_CONTENT_TYPE`] is `text` or
/// `reasoning`.
pub const MESSAGE_CONTENT_TEXT: &str = "message_content.text";
/// Prefix for the image of the content item, when [`MESSAGE_CONTENT_TYPE`] is
/// `image`.
///
/// The nested field is [`IMAGE_URL`].
pub const MESSAGE_CONTENT_IMAGE: &str = "message_content.image";
/// Prefix for the audio of the content item, when [`MESSAGE_CONTENT_TYPE`] is
/// `audio`.
///
/// The nested fields are [`AUDIO_URL`] and [`AUDIO_TRANSCRIPT`].
pub const MESSAGE_CONTENT_AUDIO: &str = "message_content.audio";
/// Prefix for the video of the content item, when [`MESSAGE_CONTENT_TYPE`] is
/// `video`.
///
/// The nested field is [`VIDEO_URL`].
pub const MESSAGE_CONTENT_VIDEO: &str = "message_content.video";
/// A provider-assigned identifier for the content item.
///
/// For OpenAI Responses reasoning items, this maps to
/// `ResponseReasoningItem.id` and should be preserved for stateless replay.
pub const MESSAGE_CONTENT_ID: &str = "message_content.id";
/// An opaque vendor-issued signature, captured verbatim.
///
/// Maps to provider `signature` fields, and to Gemini `thoughtSignature` when
/// it is attached to a non-tool content part.
pub const MESSAGE_CONTENT_SIGNATURE: &str = "message_content.signature";
/// Opaque vendor-issued data, captured verbatim.
///
/// Maps to Anthropic `redacted_thinking.data`.
pub const MESSAGE_CONTENT_DATA: &str = "message_content.data";
/// OpenAI `encrypted_content`, captured verbatim.
pub const MESSAGE_CONTENT_ENCRYPTED_CONTENT: &str = "message_content.encrypted_content";

// Image attributes, nested under MESSAGE_CONTENT_IMAGE, INPUT_IMAGES or
// OUTPUT_IMAGES.

/// A link to the image, or its base64 encoding.
pub const IMAGE_URL: &str = "image.url";

// Audio attributes, nested under MESSAGE_CONTENT_AUDIO.

/// The URL of an audio file, e.g. in cloud storage.
pub const AUDIO_URL: &str = "audio.url";
/// The MIME type of the audio file.
///
/// Legacy: new instrumentation should not set this, and consumers infer the
/// type from [`AUDIO_URL`] instead.
pub const AUDIO_MIME_TYPE: &str = "audio.mime_type";
/// The transcript of the audio file, e.g. a Whisper transcription.
pub const AUDIO_TRANSCRIPT: &str = "audio.transcript";

// Video attributes, nested under MESSAGE_CONTENT_VIDEO.

/// The URL, object-store URI or base64 data URI of a video.
///
/// Consumers infer the MIME type from the path extension or the data URI
/// prefix.
pub const VIDEO_URL: &str = "video.url";

// Document attributes, nested under RETRIEVAL_DOCUMENTS,
// RERANKER_INPUT_DOCUMENTS or RERANKER_OUTPUT_DOCUMENTS.

/// The unique identifier of the document.
pub const DOCUMENT_ID: &str = "document.id";
/// The relevance score of the document, as a float.
pub const DOCUMENT_SCORE: &str = "document.score";
/// The content of the document.
pub const DOCUMENT_CONTENT: &str = "document.content";
/// Metadata associated with the document, as a JSON string.
pub const DOCUMENT_METADATA: &str = "document.metadata";

// Tool-call attributes, nested under MESSAGE_TOOL_CALLS, or under
// MESSAGE_CONTENTS when MESSAGE_CONTENT_TYPE is `tool_use`.

/// The id of the tool call, used to match calls with their results when the
/// model makes several at once.
pub const TOOL_CALL_ID: &str = "tool_call.id";
/// The name of the function the tool call invokes.
pub const TOOL_CALL_FUNCTION_NAME: &str = "tool_call.function.name";
/// The arguments of the function the tool call invokes, as a JSON string.
pub const TOOL_CALL_FUNCTION_ARGUMENTS_JSON: &str = "tool_call.function.arguments";
/// An opaque vendor-issued reasoning token attached to a tool call, captured
/// verbatim.
///
/// Used by Gemini, which puts `thoughtSignature` on `functionCall` parts.
pub const TOOL_CALL_REASONING_SIGNATURE: &str = "tool_call.reasoning_signature";

// Completions-API attributes, nested under LLM_PROMPTS and LLM_CHOICES.

/// The text of a prompt sent to a completions API.
pub const PROMPT_TEXT: &str = "prompt.text";
/// The text of a choice returned by a completions API.
pub const COMPLETION_TEXT: &str = "completion.text";

// Prompt-source attributes, identifying a prompt from a prompt registry or
// library.

/// The vendor or origin of the prompt, e.g. `langsmith`, `portkey` or
/// `arize-phoenix`.
pub const PROMPT_VENDOR: &str = "prompt.vendor";
/// A vendor-specific id used to identify the prompt.
pub const PROMPT_ID: &str = "prompt.id";
/// A vendor-specific URL used to locate the prompt on the web.
pub const PROMPT_URL: &str = "prompt.url";
