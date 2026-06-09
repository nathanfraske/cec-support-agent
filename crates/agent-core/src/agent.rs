use inference::{ChatCompletionRequest, ChatMessage, Completer};
use serde::Serialize;

use crate::consent::Consent;
use crate::dispatch::Dispatcher;
use crate::tool::{ToolError, ToolOutcome};

/// A minimal agent execution loop.
///
/// It reaches a model through any [`Completer`] and dispatches tools through the
/// consent-gated [`Dispatcher`]. [`Agent::respond`] is the single-turn
/// primitive; [`Agent::run`] is the bounded tool-using loop (capped by
/// `max_steps`) built on it.
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
    /// The tool-using loop ([`Agent::run`]) builds on this primitive.
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

    /// Run the bounded tool-using loop on a task.
    ///
    /// The model is told the available tools and a strict reply protocol: on
    /// each turn it returns exactly one JSON object — either
    /// `{"tool": "<name>", "args": {…}}` to call a tool, or
    /// `{"final": "<answer>"}` to stop. Tool calls are dispatched through the
    /// consent gate; the outcome is fed back as the next user message. The loop
    /// runs until the model finishes or `max_steps` is reached.
    ///
    /// The protocol is JSON-in-content, so it works with any OpenAI-compatible
    /// endpoint and does not depend on server-side function-calling support.
    pub async fn run(&self, system: &str, task: &str) -> anyhow::Result<AgentRun> {
        let protocol = format!(
            "{system}\n\n\
             You have these tools:\n{tools}\n\n\
             Reply with EXACTLY ONE JSON object per turn and nothing else:\n\
             - to use a tool:  {{\"tool\": \"<name>\", \"args\": {{ ... }}}}\n\
             - to finish:      {{\"final\": \"<answer>\"}}\n\
             After a tool runs you receive its result; then reply again.",
            tools = self.dispatcher.catalog(),
        );
        let mut messages = vec![ChatMessage::system(protocol), ChatMessage::user(task)];
        let mut steps: Vec<AgentStep> = Vec::new();

        for _ in 0..self.max_steps {
            let request = ChatCompletionRequest::new(self.model.clone(), messages.clone());
            let response = self.completer.complete(request).await?;
            let content = response
                .choices
                .into_iter()
                .next()
                .map(|choice| choice.message.content)
                .unwrap_or_default();

            match parse_action(&content) {
                Action::Final(answer) => {
                    return Ok(AgentRun {
                        answer,
                        steps,
                        finished: true,
                    });
                }
                Action::Call { tool, args } => {
                    messages.push(ChatMessage::assistant(content));
                    let outcome = self
                        .dispatcher
                        .dispatch(&tool, args.clone(), self.consent)
                        .await;
                    messages.push(ChatMessage::user(feedback_for(&tool, &outcome)));
                    steps.push(AgentStep {
                        tool,
                        args,
                        outcome: outcome.map_err(|error| error.to_string()),
                    });
                }
            }
        }

        // The step cap was reached before the model produced a final answer.
        Ok(AgentRun {
            answer: String::new(),
            steps,
            finished: false,
        })
    }
}

/// The result of an [`Agent::run`] loop.
#[derive(Debug, Clone, Serialize)]
pub struct AgentRun {
    /// The model's final answer (empty if the step cap was hit first).
    pub answer: String,
    /// The tool calls made and their outcomes, in order.
    pub steps: Vec<AgentStep>,
    /// Whether the loop ended with a final answer rather than hitting the cap.
    pub finished: bool,
}

/// One tool call made during an [`Agent::run`] loop.
#[derive(Debug, Clone, Serialize)]
pub struct AgentStep {
    /// The tool that was dispatched.
    pub tool: String,
    /// The JSON arguments passed to it.
    pub args: serde_json::Value,
    /// The outcome, or a stringified dispatch error (e.g. consent denied).
    pub outcome: Result<ToolOutcome, String>,
}

/// One parsed instruction from a model turn.
enum Action {
    /// Invoke a tool with JSON arguments.
    Call {
        tool: String,
        args: serde_json::Value,
    },
    /// Stop with a final answer.
    Final(String),
}

fn feedback_for(tool: &str, outcome: &Result<ToolOutcome, ToolError>) -> String {
    match outcome {
        Ok(o) => format!("tool '{tool}' result: ok={}, {}", o.ok, o.summary),
        Err(error) => format!("tool '{tool}' error: {error}"),
    }
}

/// Parse a model turn into an [`Action`]. Recognizes a `{"tool":…,"args":…}` or
/// `{"final":…}` object; anything else is treated as a final prose answer.
fn parse_action(content: &str) -> Action {
    if let Some(object) = extract_json_object(content) {
        if let Some(tool) = object.get("tool").and_then(|v| v.as_str()) {
            let args = object
                .get("args")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            return Action::Call {
                tool: tool.to_string(),
                args,
            };
        }
        if let Some(answer) = object.get("final").and_then(|v| v.as_str()) {
            return Action::Final(answer.to_string());
        }
    }
    Action::Final(content.trim().to_string())
}

/// Best-effort extraction of a single JSON object from model output that may
/// wrap it in prose or a `<think>` block: try the whole trimmed string, then the
/// span from the first `{` to the last `}`.
fn extract_json_object(content: &str) -> Option<serde_json::Value> {
    let trimmed = content.trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if value.is_object() {
            return Some(value);
        }
    }
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&content[start..=end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_call() {
        match parse_action(r#"{"tool":"cim_query","args":{"class":"X"}}"#) {
            Action::Call { tool, args } => {
                assert_eq!(tool, "cim_query");
                assert_eq!(args["class"], "X");
            }
            Action::Final(_) => panic!("expected a tool call"),
        }
    }

    #[test]
    fn parses_final() {
        match parse_action(r#"{"final":"done"}"#) {
            Action::Final(answer) => assert_eq!(answer, "done"),
            Action::Call { .. } => panic!("expected a final answer"),
        }
    }

    #[test]
    fn extracts_json_after_a_think_block() {
        let content = "<think>I should read the state.</think> {\"tool\":\"read\",\"args\":{}}";
        match parse_action(content) {
            Action::Call { tool, .. } => assert_eq!(tool, "read"),
            Action::Final(_) => panic!("expected a tool call from wrapped JSON"),
        }
    }

    #[test]
    fn prose_is_treated_as_a_final_answer() {
        match parse_action("just some text") {
            Action::Final(answer) => assert_eq!(answer, "just some text"),
            Action::Call { .. } => panic!("expected a final answer"),
        }
    }
}
