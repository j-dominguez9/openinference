use std::borrow::Cow;
use std::time::SystemTime;

use opentelemetry::trace::{Span, SpanBuilder, SpanContext, Status, Tracer};
use opentelemetry::{Context, KeyValue};

use crate::context_attributes::context_attributes;
use crate::suppression::is_tracing_suppressed;
use crate::trace_config::TraceConfig;

/// The span context of a span that was never started because tracing was
/// suppressed.
static NO_SPAN_CONTEXT: SpanContext = SpanContext::NONE;

/// Wraps a tracer so that every span it starts records the context
/// attributes and masks its attributes according to a [`TraceConfig`].
///
/// # Examples
///
/// ```
/// use opentelemetry::global;
/// use opentelemetry::trace::Tracer;
/// use openinference_instrumentation::trace_config::TraceConfig;
/// use openinference_instrumentation::tracer::OITracer;
///
/// let tracer = OITracer::new(global::tracer("my-app"), TraceConfig::from_env());
/// let span = tracer.start("chat");
/// ```
#[derive(Debug)]
pub struct OITracer<T> {
    tracer: T,
    config: TraceConfig,
}

/// A span started by [`OITracer`]. Every attribute set on it is masked first.
#[derive(Debug)]
pub struct OISpan<S> {
    // `None` when tracing was suppressed, so the span records nothing.
    span: Option<S>,
    config: TraceConfig,
}

impl<T> OITracer<T> {
    /// Wraps `tracer`, masking span attributes according to `config`.
    pub fn new(tracer: T, config: TraceConfig) -> Self {
        Self { tracer, config }
    }

    fn wrap<S>(&self, span: Option<S>) -> OISpan<S> {
        OISpan {
            span,
            config: self.config.clone(),
        }
    }
}

impl<T: Tracer> Tracer for OITracer<T> {
    type Span = OISpan<T::Span>;

    fn build_with_context(&self, mut builder: SpanBuilder, parent_cx: &Context) -> Self::Span {
        if is_tracing_suppressed(parent_cx) {
            return self.wrap(None);
        }

        // The caller's attributes win over context attributes with the same
        // key. They also go first, so the SDK's attribute limit drops context
        // attributes before them.
        let mut attributes = builder.attributes.take().unwrap_or_default();
        let context: Vec<KeyValue> = context_attributes(parent_cx)
            .into_iter()
            .filter(|c| !attributes.iter().any(|a| a.key == c.key))
            .collect();
        attributes.extend(context);
        // Mask before building, so samplers never see hidden values.
        builder.attributes = Some(
            attributes
                .into_iter()
                .filter_map(|attribute| mask(&self.config, attribute))
                .collect(),
        );

        self.wrap(Some(self.tracer.build_with_context(builder, parent_cx)))
    }
}

impl<S: Span> Span for OISpan<S> {
    fn set_attribute(&mut self, attribute: KeyValue) {
        if let Some(span) = &mut self.span {
            if let Some(attribute) = mask(&self.config, attribute) {
                span.set_attribute(attribute);
            }
        }
    }

    // Everything else is forwarded to the wrapped span unchanged.

    fn add_event_with_timestamp<T>(
        &mut self,
        name: T,
        timestamp: SystemTime,
        attributes: Vec<KeyValue>,
    ) where
        T: Into<Cow<'static, str>>,
    {
        if let Some(span) = &mut self.span {
            span.add_event_with_timestamp(name, timestamp, attributes);
        }
    }

    fn span_context(&self) -> &SpanContext {
        match &self.span {
            Some(span) => span.span_context(),
            None => &NO_SPAN_CONTEXT,
        }
    }

    fn is_recording(&self) -> bool {
        self.span.as_ref().is_some_and(Span::is_recording)
    }

    fn set_status(&mut self, status: Status) {
        if let Some(span) = &mut self.span {
            span.set_status(status);
        }
    }

    fn update_name<T>(&mut self, new_name: T)
    where
        T: Into<Cow<'static, str>>,
    {
        if let Some(span) = &mut self.span {
            span.update_name(new_name);
        }
    }

    fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>) {
        if let Some(span) = &mut self.span {
            span.add_link(span_context, attributes);
        }
    }

    fn end_with_timestamp(&mut self, timestamp: SystemTime) {
        if let Some(span) = &mut self.span {
            span.end_with_timestamp(timestamp);
        }
    }
}

/// Applies [`TraceConfig::mask`] to a whole attribute; `None` means drop it.
fn mask(config: &TraceConfig, attribute: KeyValue) -> Option<KeyValue> {
    let KeyValue { key, value, .. } = attribute;
    let value = config.mask(key.as_str(), value)?;
    Some(KeyValue::new(key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_attributes::with_session;
    use crate::suppression::suppress_tracing;
    use crate::trace_config::REDACTED_VALUE;
    use openinference_semantic_conventions::attributes::{
        INPUT_MIME_TYPE, INPUT_VALUE, LLM_MODEL_NAME, SESSION_ID,
    };
    use opentelemetry::Value;
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider, SpanData};

    /// Runs `f` with an `OITracer` over an in-memory exporter, and returns
    /// the spans it finished.
    fn record(
        config: TraceConfig,
        f: impl FnOnce(&OITracer<opentelemetry_sdk::trace::SdkTracer>),
    ) -> Vec<SpanData> {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        f(&OITracer::new(provider.tracer("test"), config));
        exporter.get_finished_spans().unwrap()
    }

    fn attribute(span: &SpanData, key: &str) -> Option<Value> {
        span.attributes
            .iter()
            .find(|kv| kv.key.as_str() == key)
            .map(|kv| kv.value.clone())
    }

    fn hide_inputs() -> TraceConfig {
        TraceConfig {
            hide_inputs: true,
            ..TraceConfig::default()
        }
    }

    #[test]
    fn spans_are_exported() {
        let spans = record(TraceConfig::default(), |tracer| {
            tracer.start("chat").end();
        });
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "chat");
    }

    #[test]
    fn set_attribute_is_masked() {
        let spans = record(hide_inputs(), |tracer| {
            let mut span = tracer.start("chat");
            span.set_attribute(KeyValue::new(INPUT_VALUE, "secret"));
            span.set_attribute(KeyValue::new(INPUT_MIME_TYPE, "text/plain"));
            span.set_attribute(KeyValue::new(LLM_MODEL_NAME, "gpt-4"));
            span.end();
        });
        assert_eq!(
            attribute(&spans[0], INPUT_VALUE),
            Some(REDACTED_VALUE.into())
        );
        assert_eq!(attribute(&spans[0], INPUT_MIME_TYPE), None);
        assert_eq!(attribute(&spans[0], LLM_MODEL_NAME), Some("gpt-4".into()));
    }

    #[test]
    fn set_attributes_is_masked() {
        let spans = record(hide_inputs(), |tracer| {
            let mut span = tracer.start("chat");
            span.set_attributes([KeyValue::new(INPUT_VALUE, "secret")]);
            span.end();
        });
        assert_eq!(
            attribute(&spans[0], INPUT_VALUE),
            Some(REDACTED_VALUE.into())
        );
    }

    #[test]
    fn starting_attributes_are_masked() {
        let spans = record(hide_inputs(), |tracer| {
            let mut span = tracer
                .span_builder("chat")
                .with_attributes([KeyValue::new(INPUT_VALUE, "secret")])
                .start(tracer);
            span.end();
        });
        assert_eq!(
            attribute(&spans[0], INPUT_VALUE),
            Some(REDACTED_VALUE.into())
        );
    }

    #[test]
    fn context_attributes_are_recorded() {
        let spans = record(TraceConfig::default(), |tracer| {
            let cx = with_session(&Context::new(), "abc");
            tracer.start_with_context("chat", &cx).end();
        });
        assert_eq!(attribute(&spans[0], SESSION_ID), Some("abc".into()));
    }

    #[test]
    fn caller_attributes_override_context_attributes() {
        let spans = record(TraceConfig::default(), |tracer| {
            let cx = with_session(&Context::new(), "from-context");
            tracer
                .span_builder("chat")
                .with_attributes([KeyValue::new(SESSION_ID, "from-caller")])
                .start_with_context(tracer, &cx)
                .end();
        });
        let sessions: Vec<_> = spans[0]
            .attributes
            .iter()
            .filter(|kv| kv.key.as_str() == SESSION_ID)
            .collect();
        assert_eq!(sessions, [&KeyValue::new(SESSION_ID, "from-caller")]);
    }

    #[test]
    fn suppressed_spans_are_not_exported() {
        let spans = record(TraceConfig::default(), |tracer| {
            let _guard = suppress_tracing();
            let mut span = tracer.start("chat");
            assert!(!span.is_recording());
            assert_eq!(span.span_context(), &SpanContext::NONE);
            span.set_attribute(KeyValue::new(INPUT_VALUE, "x"));
            span.end();
        });
        assert!(spans.is_empty());
    }
}
