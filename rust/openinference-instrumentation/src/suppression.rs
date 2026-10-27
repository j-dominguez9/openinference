use opentelemetry::{Context, ContextGuard};

/// Stops tracing on the current thread until the returned guard is dropped.
///
/// Bind the guard with `let _guard`; `let _ =` drops it immediately. The guard
/// applies only to the current thread, so async code should pass a suppressed
/// [`Context`] along instead.
#[must_use = "suppression ends when the guard is dropped; bind it with `let _guard`"]
pub fn suppress_tracing() -> ContextGuard {
    Context::enter_telemetry_suppressed_scope()
}

/// Returns whether instrumentors should skip creating spans in `cx`.
pub fn is_tracing_suppressed(cx: &Context) -> bool {
    cx.is_telemetry_suppressed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_suppresses_tracing_until_dropped() {
        assert!(!is_tracing_suppressed(&Context::current()));
        {
            let _guard = suppress_tracing();
            assert!(is_tracing_suppressed(&Context::current()));
        }
        assert!(!is_tracing_suppressed(&Context::current()));
    }

    #[test]
    fn nested_guards_restore_the_outer_scope() {
        let _outer = suppress_tracing();
        {
            let _inner = suppress_tracing();
            assert!(is_tracing_suppressed(&Context::current()));
        }
        // Dropping the inner guard restores the outer, still-suppressed context.
        assert!(is_tracing_suppressed(&Context::current()));
    }

    #[test]
    fn suppression_does_not_leak_to_other_threads() {
        let _guard = suppress_tracing();
        let other = std::thread::spawn(|| is_tracing_suppressed(&Context::current()));
        assert!(!other.join().unwrap());
    }

    #[test]
    fn a_suppressed_context_can_be_passed_explicitly() {
        // What async code does instead of relying on the thread's current context.
        let cx = Context::current().with_telemetry_suppressed();
        assert!(is_tracing_suppressed(&cx));
        assert!(!is_tracing_suppressed(&Context::current()));
    }
}
