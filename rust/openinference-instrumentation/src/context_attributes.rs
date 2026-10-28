use openinference_semantic_conventions::attributes::{
    LLM_PROMPT_TEMPLATE, LLM_PROMPT_TEMPLATE_VARIABLES, LLM_PROMPT_TEMPLATE_VERSION, METADATA,
    SESSION_ID, TAG_TAGS, USER_ID,
};
use opentelemetry::{Array, Context, ContextGuard, KeyValue, StringValue, Value};

#[derive(Debug)]
struct SessionId(String);

#[derive(Debug)]
struct UserId(String);

#[derive(Debug)]
struct Metadata(String);

#[derive(Debug)]
struct Tags(Vec<String>);

#[derive(Debug)]
struct PromptTemplate(String);

#[derive(Debug)]
struct PromptTemplateVersion(String);

#[derive(Debug)]
struct PromptTemplateVariables(String);

/// Returns `cx` with `session_id` attached; spans created in the returned
/// context record it as `session.id`.
pub fn with_session(cx: &Context, session_id: impl Into<String>) -> Context {
    with_nonempty(cx, session_id.into(), SessionId)
}

/// Returns `cx` with `user_id` attached; spans created in the returned
/// context record it as `user.id`.
pub fn with_user(cx: &Context, user_id: impl Into<String>) -> Context {
    with_nonempty(cx, user_id.into(), UserId)
}

/// Returns `cx` with `metadata` attached; spans created in the returned
/// context record it as `metadata`. Pass a JSON object as a string,
/// e.g. `r#"{"tenant":"acme"}"#`.
pub fn with_metadata(cx: &Context, metadata: impl Into<String>) -> Context {
    with_nonempty(cx, metadata.into(), Metadata)
}

/// Returns `cx` with `tags` attached; spans created in the returned
/// context record them as `tag.tags`. An empty list leaves `cx` unchanged.
pub fn with_tags(cx: &Context, tags: impl IntoIterator<Item = impl Into<String>>) -> Context {
    let tags: Vec<String> = tags.into_iter().map(Into::into).collect();
    if tags.is_empty() {
        return cx.clone();
    }
    cx.with_value(Tags(tags))
}

/// Returns `cx` with a prompt template attached; spans created in the
/// returned context record it as `llm.prompt_template.*`. Empty values are
/// skipped. Pass `variables` as a JSON object string, e.g.
/// `r#"{"city":"Johannesburg"}"#`.
pub fn with_prompt_template(
    cx: &Context,
    template: impl Into<String>,
    version: impl Into<String>,
    variables: impl Into<String>,
) -> Context {
    let cx = with_nonempty(cx, template.into(), PromptTemplate);
    let cx = with_nonempty(&cx, version.into(), PromptTemplateVersion);
    with_nonempty(&cx, variables.into(), PromptTemplateVariables)
}

/// Returns `cx` with `wrap(value)` attached, or `cx` unchanged if `value` is
/// empty.
fn with_nonempty<T: Send + Sync + 'static>(
    cx: &Context,
    value: String,
    wrap: fn(String) -> T,
) -> Context {
    if value.is_empty() {
        return cx.clone();
    }
    cx.with_value(wrap(value))
}

// Guards: attach to the current thread until dropped, like Python's `using_*`.

/// Attaches `session_id` to the current context until the returned guard is
/// dropped. Like [`suppress_tracing`](crate::suppression::suppress_tracing),
/// the guard applies only to the current thread; async code should pass
/// [`with_session`]'s context along instead.
#[must_use = "the session is detached when the guard is dropped; bind it with `let _guard`"]
pub fn using_session(session_id: impl Into<String>) -> ContextGuard {
    with_session(&Context::current(), session_id).attach()
}

/// Like [`using_session`], for [`with_user`].
#[must_use = "the user is detached when the guard is dropped; bind it with `let _guard`"]
pub fn using_user(user_id: impl Into<String>) -> ContextGuard {
    with_user(&Context::current(), user_id).attach()
}

/// Like [`using_session`], for [`with_metadata`].
#[must_use = "the metadata is detached when the guard is dropped; bind it with `let _guard`"]
pub fn using_metadata(metadata: impl Into<String>) -> ContextGuard {
    with_metadata(&Context::current(), metadata).attach()
}

/// Like [`using_session`], for [`with_tags`].
#[must_use = "the tags are detached when the guard is dropped; bind it with `let _guard`"]
pub fn using_tags(tags: impl IntoIterator<Item = impl Into<String>>) -> ContextGuard {
    with_tags(&Context::current(), tags).attach()
}

/// Like [`using_session`], for [`with_prompt_template`].
#[must_use = "the prompt template is detached when the guard is dropped; bind it with `let _guard`"]
pub fn using_prompt_template(
    template: impl Into<String>,
    version: impl Into<String>,
    variables: impl Into<String>,
) -> ContextGuard {
    with_prompt_template(&Context::current(), template, version, variables).attach()
}

/// Returns the attributes set on `cx` by the `with_*` functions, ready to
/// add to a span.
pub fn context_attributes(cx: &Context) -> Vec<KeyValue> {
    let mut attributes = Vec::new();
    if let Some(SessionId(id)) = cx.get::<SessionId>() {
        attributes.push(KeyValue::new(SESSION_ID, id.clone()));
    }

    if let Some(UserId(id)) = cx.get::<UserId>() {
        attributes.push(KeyValue::new(USER_ID, id.clone()));
    }

    if let Some(Metadata(md)) = cx.get::<Metadata>() {
        attributes.push(KeyValue::new(METADATA, md.clone()));
    }

    if let Some(Tags(tags)) = cx.get::<Tags>() {
        let tags: Vec<StringValue> = tags.iter().cloned().map(StringValue::from).collect();
        attributes.push(KeyValue::new(TAG_TAGS, Value::Array(Array::String(tags))));
    }

    if let Some(PromptTemplate(pt)) = cx.get::<PromptTemplate>() {
        attributes.push(KeyValue::new(LLM_PROMPT_TEMPLATE, pt.clone()));
    }

    if let Some(PromptTemplateVersion(ptv)) = cx.get::<PromptTemplateVersion>() {
        attributes.push(KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, ptv.clone()));
    }

    if let Some(PromptTemplateVariables(ptvar)) = cx.get::<PromptTemplateVariables>() {
        attributes.push(KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, ptvar.clone()));
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_context_has_no_attributes() {
        assert!(context_attributes(&Context::new()).is_empty());
    }

    #[test]
    fn session_is_recorded() {
        let cx = with_session(&Context::new(), "abc");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(SESSION_ID, "abc")]
        );
    }

    #[test]
    fn user_is_recorded() {
        let cx = with_user(&Context::new(), "u1");
        assert_eq!(context_attributes(&cx), vec![KeyValue::new(USER_ID, "u1")]);
    }

    #[test]
    fn empty_ids_add_nothing() {
        let cx = with_user(&with_session(&Context::new(), ""), "");
        assert!(context_attributes(&cx).is_empty());
        let cx = with_metadata(&Context::new(), "");
        assert!(context_attributes(&cx).is_empty());
        let cx = with_tags(&Context::new(), Vec::<String>::new());
        assert!(context_attributes(&cx).is_empty());
    }

    #[test]
    fn empty_id_keeps_the_existing_value() {
        let cx = with_session(&with_session(&Context::new(), "abc"), "");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(SESSION_ID, "abc")]
        );
    }

    #[test]
    fn nested_setters_combine() {
        let cx = with_user(&with_session(&Context::new(), "s"), "u");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(SESSION_ID, "s"), KeyValue::new(USER_ID, "u")]
        );
    }

    #[test]
    fn inner_session_overrides_outer() {
        let cx = with_session(&with_session(&Context::new(), "outer"), "inner");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(SESSION_ID, "inner")]
        );
    }

    #[test]
    fn setters_leave_the_original_context_unchanged() {
        let parent = Context::new();
        let _child = with_session(&parent, "abc");
        assert!(context_attributes(&parent).is_empty());
    }

    #[test]
    fn metadata_is_recorded() {
        let json = r#"{"tenant":"acme"}"#;
        let cx = with_metadata(&Context::new(), json);
        assert_eq!(context_attributes(&cx), vec![KeyValue::new(METADATA, json)]);
    }

    #[test]
    fn tags_are_recorded_as_a_string_array() {
        let cx = with_tags(&Context::new(), ["a", "b"]);
        let expected = Value::Array(Array::String(vec!["a".into(), "b".into()]));
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(TAG_TAGS, expected)]
        );
    }

    #[test]
    fn tags_accept_owned_strings_and_keep_empty_entries() {
        let cx = with_tags(&Context::new(), vec![String::from("a"), String::new()]);
        let expected = Value::Array(Array::String(vec!["a".into(), "".into()]));
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(TAG_TAGS, expected)]
        );
    }

    #[test]
    fn all_attributes_combine() {
        let cx = with_session(&Context::new(), "s");
        let cx = with_user(&cx, "u");
        let cx = with_metadata(&cx, "{}");
        let cx = with_tags(&cx, ["t"]);
        let cx = with_prompt_template(&cx, "tpl", "v1", "{}");
        let tags = Value::Array(Array::String(vec!["t".into()]));
        assert_eq!(
            context_attributes(&cx),
            vec![
                KeyValue::new(SESSION_ID, "s"),
                KeyValue::new(USER_ID, "u"),
                KeyValue::new(METADATA, "{}"),
                KeyValue::new(TAG_TAGS, tags),
                KeyValue::new(LLM_PROMPT_TEMPLATE, "tpl"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, "v1"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, "{}"),
            ]
        );
    }

    #[test]
    fn prompt_template_is_recorded() {
        let variables = r#"{"city":"Johannesburg"}"#;
        let cx = with_prompt_template(&Context::new(), "Weather in {city}", "v1.0", variables);
        assert_eq!(
            context_attributes(&cx),
            vec![
                KeyValue::new(LLM_PROMPT_TEMPLATE, "Weather in {city}"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, "v1.0"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, variables),
            ]
        );
    }

    #[test]
    fn prompt_template_skips_empty_values() {
        let cx = with_prompt_template(&Context::new(), "tpl", "", "");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(LLM_PROMPT_TEMPLATE, "tpl")]
        );

        let cx = with_prompt_template(&Context::new(), "", "v1", "");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, "v1")]
        );

        let cx = with_prompt_template(&Context::new(), "", "", "{}");
        assert_eq!(
            context_attributes(&cx),
            vec![KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, "{}")]
        );
    }

    #[test]
    fn empty_prompt_template_adds_nothing() {
        let cx = with_prompt_template(&Context::new(), "", "", "");
        assert!(context_attributes(&cx).is_empty());
    }

    #[test]
    fn guard_attaches_until_dropped() {
        let guard = using_session("abc");
        assert_eq!(
            context_attributes(&Context::current()),
            vec![KeyValue::new(SESSION_ID, "abc")]
        );
        drop(guard);
        assert!(context_attributes(&Context::current()).is_empty());
    }

    #[test]
    fn nested_guards_restore_the_outer_scope() {
        let outer = using_user("u");
        let inner = using_session("s");
        assert_eq!(
            context_attributes(&Context::current()),
            vec![KeyValue::new(SESSION_ID, "s"), KeyValue::new(USER_ID, "u")]
        );
        drop(inner);
        assert_eq!(
            context_attributes(&Context::current()),
            vec![KeyValue::new(USER_ID, "u")]
        );
        drop(outer);
        assert!(context_attributes(&Context::current()).is_empty());
    }

    #[test]
    fn every_guard_attaches_its_attribute() {
        let _session = using_session("s");
        let _user = using_user("u");
        let _metadata = using_metadata("{}");
        let _tags = using_tags(["t"]);
        let _template = using_prompt_template("tpl", "v1", "{}");
        let tags = Value::Array(Array::String(vec!["t".into()]));
        assert_eq!(
            context_attributes(&Context::current()),
            vec![
                KeyValue::new(SESSION_ID, "s"),
                KeyValue::new(USER_ID, "u"),
                KeyValue::new(METADATA, "{}"),
                KeyValue::new(TAG_TAGS, tags),
                KeyValue::new(LLM_PROMPT_TEMPLATE, "tpl"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, "v1"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, "{}"),
            ]
        );
    }

    #[test]
    fn prompt_template_values_are_replaced_independently() {
        let cx = with_prompt_template(&Context::new(), "old", "v1", "{}");
        let cx = with_prompt_template(&cx, "new", "", "");
        assert_eq!(
            context_attributes(&cx),
            vec![
                KeyValue::new(LLM_PROMPT_TEMPLATE, "new"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VERSION, "v1"),
                KeyValue::new(LLM_PROMPT_TEMPLATE_VARIABLES, "{}"),
            ]
        );
    }
}
