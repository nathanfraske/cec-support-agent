use serde::{Deserialize, Serialize};

/// How reasoned and precise the person's own explanation was — and therefore
/// what register the response should take.
///
/// This is calibration, not gatekeeping: it changes how much each reply
/// *teaches* (definitions, examples, walkthroughs), never what is checked,
/// which questions are asked, or which safety boundaries are stated. Consent
/// renderings and safety warnings are identical in both registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fluency {
    /// The default: assume no technical background. Every term is explained,
    /// every question carries an example, every verdict is taught.
    #[default]
    Guided,
    /// The explanation was precise and reasoned — exact codes or module
    /// names, technical vocabulary, the standard intake facts volunteered
    /// unprompted. Respond in kind: concise steps, no definitions.
    Technical,
}
