/// Values for the `openinference.span.kind` attribute. Pick the value that best
/// describes the span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SpanKind {
    /// Raw provider API Calls
    Llm,
    /// Orchestration boundaries
    Chain,
    /// Decision-model call that scores or selects among candidate options
    Decision,
    /// Function/tool execution
    Tool,
    /// Vector-store lookups
    Retriever,
    /// Embedding API calls
    Embedding,
    /// Autonomous sub-agent run nested inside a larger chain
    Agent,
    /// Rerank API calls
    Reranker,
    /// Guardrail/policy checks
    Guardrail,
    /// Online eval calls
    Evaluator,
    /// Prompt-registry lookup
    Prompt,
    /// Fallback when no other kind applies
    Unknown,
}

impl SpanKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            SpanKind::Llm => "LLM",
            SpanKind::Chain => "CHAIN",
            SpanKind::Decision => "DECISION",
            SpanKind::Tool => "TOOL",
            SpanKind::Retriever => "RETRIEVER",
            SpanKind::Embedding => "EMBEDDING",
            SpanKind::Agent => "AGENT",
            SpanKind::Reranker => "RERANKER",
            SpanKind::Guardrail => "GUARDRAIL",
            SpanKind::Evaluator => "EVALUATOR",
            SpanKind::Prompt => "PROMPT",
            SpanKind::Unknown => "UNKNOWN",
        }
    }
}

/// Values for `annotation.annotator_kind` and `evaluation.annotator_kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AnnotatorKind {
    Human,
    Llm,
    Code,
}

impl AnnotatorKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            AnnotatorKind::Human => "HUMAN",
            AnnotatorKind::Llm => "LLM",
            AnnotatorKind::Code => "CODE",
        }
    }
}

/// Values for the `input.mime_type` / `output.mime_type` attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MimeType {
    Text,
    Json,
}

impl MimeType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MimeType::Text => "text/plain",
            MimeType::Json => "application/json",
        }
    }
}

/// Values for the `llm.system` attribute (the AI product as identified by the
/// client or server).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LlmSystem {
    Openai,
    Anthropic,
    Cohere,
    Mistralai,
    Vertexai,
    TypeSafe,
}

impl LlmSystem {
    pub const fn as_str(&self) -> &'static str {
        match self {
            LlmSystem::Openai => "openai",
            LlmSystem::Anthropic => "anthropic",
            LlmSystem::Cohere => "cohere",
            LlmSystem::Mistralai => "mistralai",
            LlmSystem::Vertexai => "vertexai",
            LlmSystem::TypeSafe => "typesafe",
        }
    }
}

/// Values for the `llm.provider` attribute (the company providing the model —
/// often the same as the system, but distinct for providers that resell
/// models from elsewhere, e.g. Azure → OpenAI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LlmProvider {
    Openai,
    Anthropic,
    Cohere,
    Mistralai,
    Google,
    Azure,
    Aws,
    Xai,
    DeepSeek,
    Groq,
    Fireworks,
    Moonshot,
    Cerebras,
    Perplexity,
    Together,
    Ollama,
    Meta,
    Zai,
    MiniMax,
    Oracle,
    TypeSafe,
}

impl LlmProvider {
    pub const fn as_str(&self) -> &'static str {
        match self {
            LlmProvider::Openai => "openai",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Cohere => "cohere",
            LlmProvider::Mistralai => "mistralai",
            LlmProvider::Google => "google",
            LlmProvider::Azure => "azure",
            LlmProvider::Aws => "aws",
            LlmProvider::Xai => "xai",
            LlmProvider::DeepSeek => "deepseek",
            LlmProvider::Groq => "groq",
            LlmProvider::Fireworks => "fireworks",
            LlmProvider::Moonshot => "moonshot",
            LlmProvider::Cerebras => "cerebras",
            LlmProvider::Perplexity => "perplexity",
            LlmProvider::Together => "together",
            LlmProvider::Ollama => "ollama",
            LlmProvider::Meta => "meta",
            LlmProvider::Zai => "zai",
            LlmProvider::MiniMax => "minimax",
            LlmProvider::Oracle => "oracle",
            LlmProvider::TypeSafe => "typesafe",
        }
    }
}

/// A subset of vendors currently known to offer
/// a decision API. Values for `decision.system` attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DecisionSystem {
    TypeSafe,
    Openai,
}

impl DecisionSystem {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DecisionSystem::TypeSafe => "typesafe",
            DecisionSystem::Openai => "openai",
        }
    }
}

/// Well-known values for the `decision.provider` attribute: who hosts the decision model
/// that answered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DecisionProvider {
    TypeSafe,
    Openai,
}

impl DecisionProvider {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DecisionProvider::TypeSafe => "typesafe",
            DecisionProvider::Openai => "openai",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_expected_values() {
        let cases = [
            (SpanKind::Llm, "LLM"),
            (SpanKind::Chain, "CHAIN"),
            (SpanKind::Decision, "DECISION"),
            (SpanKind::Tool, "TOOL"),
            (SpanKind::Retriever, "RETRIEVER"),
            (SpanKind::Embedding, "EMBEDDING"),
            (SpanKind::Agent, "AGENT"),
            (SpanKind::Reranker, "RERANKER"),
            (SpanKind::Guardrail, "GUARDRAIL"),
            (SpanKind::Evaluator, "EVALUATOR"),
            (SpanKind::Prompt, "PROMPT"),
            (SpanKind::Unknown, "UNKNOWN"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_annotator_kind() {
        let cases = [
            (AnnotatorKind::Llm, "LLM"),
            (AnnotatorKind::Human, "HUMAN"),
            (AnnotatorKind::Code, "CODE"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_mime_type() {
        let cases = [
            (MimeType::Text, "text/plain"),
            (MimeType::Json, "application/json"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_llm_system() {
        let cases = [
            (LlmSystem::Openai, "openai"),
            (LlmSystem::Anthropic, "anthropic"),
            (LlmSystem::Cohere, "cohere"),
            (LlmSystem::Mistralai, "mistralai"),
            (LlmSystem::Vertexai, "vertexai"),
            (LlmSystem::TypeSafe, "typesafe"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_llm_provider() {
        let cases = [
            (LlmProvider::Openai, "openai"),
            (LlmProvider::Anthropic, "anthropic"),
            (LlmProvider::Cohere, "cohere"),
            (LlmProvider::Mistralai, "mistralai"),
            (LlmProvider::Google, "google"),
            (LlmProvider::Azure, "azure"),
            (LlmProvider::Aws, "aws"),
            (LlmProvider::Xai, "xai"),
            (LlmProvider::DeepSeek, "deepseek"),
            (LlmProvider::Groq, "groq"),
            (LlmProvider::Fireworks, "fireworks"),
            (LlmProvider::Moonshot, "moonshot"),
            (LlmProvider::Cerebras, "cerebras"),
            (LlmProvider::Perplexity, "perplexity"),
            (LlmProvider::Together, "together"),
            (LlmProvider::Ollama, "ollama"),
            (LlmProvider::Meta, "meta"),
            (LlmProvider::Zai, "zai"),
            (LlmProvider::MiniMax, "minimax"),
            (LlmProvider::Oracle, "oracle"),
            (LlmProvider::TypeSafe, "typesafe"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_decision_system() {
        let cases = [
            (DecisionSystem::TypeSafe, "typesafe"),
            (DecisionSystem::Openai, "openai"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    #[test]
    fn as_str_returns_expected_values_decision_provider() {
        let cases = [
            (DecisionProvider::TypeSafe, "typesafe"),
            (DecisionProvider::Openai, "openai"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected, "wrong string for {kind:?}");
        }
    }

    // The spec requires decision values to alias the matching llm values, so
    // the same string names the same vendor on LLM and DECISION spans.
    #[test]
    fn decision_system_matches_llm_system() {
        let cases = [
            (DecisionSystem::TypeSafe, LlmSystem::TypeSafe),
            (DecisionSystem::Openai, LlmSystem::Openai),
        ];

        for (decision, llm) in cases {
            assert_eq!(decision.as_str(), llm.as_str(), "{decision:?} != {llm:?}");
        }
    }

    #[test]
    fn decision_provider_matches_llm_provider() {
        let cases = [
            (DecisionProvider::TypeSafe, LlmProvider::TypeSafe),
            (DecisionProvider::Openai, LlmProvider::Openai),
        ];

        for (decision, llm) in cases {
            assert_eq!(decision.as_str(), llm.as_str(), "{decision:?} != {llm:?}");
        }
    }
}
