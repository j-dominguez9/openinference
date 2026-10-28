//! OpenInference instrumentation for [`async-openai`](https://docs.rs/async-openai).
//!
//! Add [`OpenInferenceLayer`] to the client's tower stack to trace chat
//! completions as OpenInference LLM spans.

mod layer;
mod message;
mod request;
mod response;
mod stream;

pub use layer::{OpenInferenceLayer, OpenInferenceService};
