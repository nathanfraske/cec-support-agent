use async_trait::async_trait;

use crate::error::InferenceError;
use crate::types::{ChatCompletionRequest, ChatCompletionResponse};

/// Connection settings for an OpenAI-compatible endpoint.
///
/// The default targets a local server, keeping the engine cold-startable with
/// no outbound connection.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// Base URL including the API version segment, e.g. `http://host:port/v1`.
    pub base_url: String,
    /// Optional bearer token. Local servers usually need none.
    pub api_key: Option<String>,
}

impl Endpoint {
    /// Create an endpoint from a base URL, trimming any trailing slash.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
        }
    }

    /// Attach a bearer token for hosted endpoints that require one.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self::new("http://localhost:8080/v1")
    }
}

/// Anything that can complete a chat request. Implemented by [`OpenAiClient`];
/// tests and offline flows supply their own.
#[async_trait]
pub trait Completer: Send + Sync {
    /// Complete `request`, returning the parsed response or an error.
    async fn complete(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, InferenceError>;
}

/// HTTP client for an OpenAI-compatible Chat Completions endpoint.
#[derive(Debug, Clone)]
pub struct OpenAiClient {
    endpoint: Endpoint,
    http: reqwest::Client,
}

impl OpenAiClient {
    /// Build a client for `endpoint`.
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            http: reqwest::Client::new(),
        }
    }

    /// The endpoint this client targets.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}

#[async_trait]
impl Completer for OpenAiClient {
    async fn complete(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, InferenceError> {
        let url = format!("{}/chat/completions", self.endpoint.base_url);
        let mut builder = self.http.post(&url).json(&request);
        if let Some(key) = &self.endpoint.api_key {
            builder = builder.bearer_auth(key);
        }

        let response = builder.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(InferenceError::Status {
                status: status.as_u16(),
                body,
            });
        }

        let parsed: ChatCompletionResponse = response.json().await?;
        if parsed.choices.is_empty() {
            return Err(InferenceError::EmptyResponse);
        }
        Ok(parsed)
    }
}
