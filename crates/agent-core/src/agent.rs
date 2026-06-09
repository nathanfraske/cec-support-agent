use inference::{ChatCompletionRequest, ChatMessage, Completer};

use crate::consent::Consent;
use crate::dispatch::Dispatcher;

/// A minimal agent execution loop.
///
/// It reaches a model through any [`Completer`] and dispatches tools through the
/// consent-gated [`Dispatcher`]. This bootstrap exposes the single-turn
/// [`Agent::respond`] primitive; the full tool-using loop (bounded by
/// `max_steps`) builds on it.
pub struct Agent<'a> {
    completer: &'a dyn Completer,
    dispatcher: &'a Dispatcher,
    model: String,
    consent: Consent,
    max_steps: usize,
}

impl<'a> Agent<'a> {
    /// Build an agent bound to a completer, a dispatcher, and a model name.
    pub fn new(
        completer: &'a dyn Completer,
        dispatcher: &'a Dispatcher,
        model: impl Into<String>,
    ) -> Self {
        Self {
            completer,
            dispatcher,
            model: model.into(),
            consent: Consent::default(),
            max_steps: 8,
        }
    }

    /// Set the consent level the agent runs under.
    pub fn with_consent(mut self, consent: Consent) -> Self {
        self.consent = consent;
        self
    }

    /// Cap the number of tool-using steps the loop may take.
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// The consent level the agent runs under.
    pub fn consent(&self) -> Consent {
        self.consent
    }

    /// The configured step cap.
    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    /// The dispatcher the agent uses to run tools.
    pub fn dispatcher(&self) -> &Dispatcher {
        self.dispatcher
    }

    /// Ask the model for a single completion given a system and a user message.
    /// The full tool-using loop builds on this primitive.
    pub async fn respond(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let request = ChatCompletionRequest::new(
            self.model.clone(),
            vec![ChatMessage::system(system), ChatMessage::user(user)],
        );
        let response = self.completer.complete(request).await?;
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("model returned no choices"))?;
        Ok(choice.message.content)
    }
}
