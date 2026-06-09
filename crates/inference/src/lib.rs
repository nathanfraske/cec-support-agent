// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! OpenAI-compatible inference over HTTP.
//!
//! This crate hardwires no provider, model, or GPU runtime (invariant 4). It
//! speaks the OpenAI Chat Completions wire format to any endpoint that
//! implements it — a hosted API or a local server such as llama.cpp, vLLM,
//! LM Studio, or Ollama. The endpoint is fully pluggable via [`Endpoint`], so
//! the engine cold-starts against `http://localhost` with no outbound call
//! (invariants 3 and 5).

mod client;
mod error;
mod types;

pub use client::{Completer, Endpoint, OpenAiClient};
pub use error::InferenceError;
pub use types::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, Choice, Role, Usage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_and_omits_unset_optionals() {
        let req = ChatCompletionRequest::new("local-model", vec![ChatMessage::user("hello")]);
        let value = serde_json::to_value(&req).expect("serialize");
        assert_eq!(value["model"], "local-model");
        assert_eq!(value["messages"][0]["role"], "user");
        assert_eq!(value["messages"][0]["content"], "hello");
        // Unset optional fields must not appear on the wire.
        assert!(value.get("temperature").is_none());
        assert!(value.get("max_tokens").is_none());
    }

    #[test]
    fn endpoint_trims_trailing_slash() {
        let ep = Endpoint::new("http://localhost:8080/v1/");
        assert_eq!(ep.base_url, "http://localhost:8080/v1");
    }
}
