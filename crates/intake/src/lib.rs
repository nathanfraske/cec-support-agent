// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Intake judge: map a person's support request to a structured case.
//!
//! People don't like to explain things. The [`Interview`] takes whatever they
//! typed, infers every case field it can, and asks the standard helpdesk
//! follow-up questions — only for the fields the text left open, at most
//! [`MAX_QUESTIONS`], never repeating one. The result is a [`Case`]: the
//! structured object the rest of the pipeline runs on.
//!
//! This is step 1 of the standard troubleshooting methodology (CompTIA A+),
//! and the engine already implements the rest:
//!
//! 1. **Identify the problem** (question the user, identify symptoms,
//!    determine what changed) — this crate.
//! 2. **Establish a theory of probable cause** — the swarm's
//!    hypothesis-seeded generators, primed by [`Case::recent_change`].
//! 3. **Test the theory** — sandbox validation.
//! 4. **Establish and implement a plan of action** — the judge's winner,
//!    executed under the consent gate.
//! 5. **Verify full system functionality** — `verify_outcome` against the
//!    original signature; [`Case::reproducibility`] decides whether a clean
//!    re-collection confirms a fix or only paroles it.
//! 6. **Document findings, actions, and outcomes** — the labeled corpus
//!    write.
//!
//! The interview *structure* is deterministic and model-free (a cold-start
//! invariant): the question bank decides which field to ask about and when to
//! stop, and the answer parsers fill the case — a model can never add a
//! question category or extend the funnel. What a model *may* do is phrase
//! the ask: the [`Interviewer`] trait separates question selection from
//! question wording, with the [`ScriptedInterviewer`] question bank as the
//! cold-start default and the [`ModelInterviewer`] sharpening each prompt
//! with case context (falling back to the script on any error or
//! non-question reply). Every answer is run through structured extraction, so
//! the moment the user finally types the stop code it lands in the fault
//! signature. Free text stays in the [`Case`] (ticket context); only
//! structured symptoms flow toward the corpus.

use async_trait::async_trait;
use common::{extract_symptoms, FaultSignature, Fluency, Symptom};
use inference::{ChatCompletionRequest, ChatMessage, Completer};
use serde::{Deserialize, Serialize};

/// Hard cap on follow-up questions: the interview funnels, it does not
/// interrogate.
pub const MAX_QUESTIONS: usize = 5;

/// When the problem began, relative to a working state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Onset {
    /// Not established yet.
    #[default]
    Unknown,
    /// It worked, then stopped at a point in time.
    Sudden,
    /// Degrading over time.
    Gradual,
    /// It has never worked.
    NeverWorked,
}

/// What changed shortly before the problem (the single highest-yield helpdesk
/// question). Primes which causal hypothesis the swarm should weigh first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecentChange {
    /// Not established yet.
    #[default]
    Unknown,
    /// The user reports nothing changed.
    NoChange,
    /// An OS update landed.
    OsUpdate,
    /// A driver was updated or installed.
    DriverUpdate,
    /// New software was installed.
    NewSoftware,
    /// New hardware was added or swapped.
    NewHardware,
    /// Settings or configuration were changed.
    ConfigChange,
}

/// How reliably the fault reproduces. Decides the verification class: an
/// intermittent fault can never be *confirmed* fixed by one clean
/// re-collection, only paroled under a monitoring horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reproducibility {
    /// Not established yet — treated as intermittent, the conservative class.
    #[default]
    Unknown,
    /// Reproduces every time.
    Always,
    /// Happens sometimes, unpredictably.
    Intermittent,
    /// Observed once so far.
    Once,
}

/// Whether the fault is contained to one program or system-wide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Not established yet.
    #[default]
    Unknown,
    /// One application or component.
    SingleApp,
    /// The whole machine.
    WholeSystem,
}

/// The structured case an intake interview produces: the "actual case" behind
/// whatever the person typed. The free-text fields are ticket context and
/// never enter the corpus; the corpus path sees only the structured
/// [`signature`](Case::signature).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Case {
    /// The user's original statement, verbatim (ticket context only).
    pub statement: String,
    /// Structured symptoms accumulated from the statement and every answer.
    pub symptoms: Vec<Symptom>,
    /// When the problem began.
    pub onset: Onset,
    /// What changed shortly before it.
    pub recent_change: RecentChange,
    /// How reliably it reproduces.
    pub reproducibility: Reproducibility,
    /// One program or the whole machine.
    pub scope: Scope,
    /// How reasoned and precise the initial explanation was; calibrates the
    /// response register (see [`Fluency`]). Assessed once, from the person's
    /// own words — answers coaxed out by the interview don't change it.
    pub fluency: Fluency,
}

impl Case {
    /// The de-identified fault signature over the accumulated symptoms.
    pub fn signature(&self) -> FaultSignature {
        FaultSignature::from_symptoms(self.symptoms.clone())
    }

    /// A one-line structured brief — for run output and for priming the
    /// hypothesis generators ("the user reports it began after a driver
    /// update"). Built only from enum fields, so it is safe in any prompt.
    pub fn brief(&self) -> String {
        format!(
            "onset={:?} change={:?} repro={:?} scope={:?}",
            self.onset, self.recent_change, self.reproducibility, self.scope
        )
    }

    /// Whether every intake field is established (none left `Unknown`). A
    /// fully established case is the "routine ticket" signal: callers use it
    /// to route work to a lighter model tier, while vague or novel cases keep
    /// the heavyweight one.
    pub fn is_established(&self) -> bool {
        self.onset != Onset::Unknown
            && self.recent_change != RecentChange::Unknown
            && self.reproducibility != Reproducibility::Unknown
            && self.scope != Scope::Unknown
    }
}

/// One follow-up question category. Ordered as the helpdesk funnel asks them:
/// exact evidence first, then history, then behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    /// "Type the error exactly as shown" — the highest-yield evidence ask.
    ExactError,
    /// Did it ever work; when did it stop.
    Onset,
    /// What changed shortly before.
    RecentChange,
    /// Every time, or sometimes.
    Reproducibility,
    /// One app, or the whole machine.
    Scope,
}

/// A follow-up question ready to put to the user. Both registers fill the
/// same case field — the guided form explains its terms and gives examples,
/// the concise form respects an explanation that was already precise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Question {
    /// Which case field this question fills.
    pub kind: QuestionKind,
    /// The guided prompt: terms defined, examples given.
    pub prompt: &'static str,
    /// The concise prompt for technically fluent reporters.
    pub concise: &'static str,
}

const QUESTIONS: &[Question] = &[
    Question {
        kind: QuestionKind::ExactError,
        prompt: "When the problem happens, does any message appear on the screen? It \
                 might be a small pop-up window, or a full blue screen with white text. \
                 On a blue screen, look near the bottom for a line that says 'Stop \
                 code:' followed by words joined by underscores (for example \
                 CRITICAL_PROCESS_DIED). Type whatever the message says, exactly as \
                 written — or 'none' if there is no message.",
        concise: "Exact error message, code, or stop code, verbatim? (or 'none')",
    },
    Question {
        kind: QuestionKind::Onset,
        prompt: "When did this start — and did the computer ever work without the \
                 problem? For example: 'it was fine until yesterday', 'it has never \
                 worked right', or 'it has been slowly getting worse for weeks'.",
        concise: "When did it start — did it ever work? (sudden / gradual / never worked)",
    },
    Question {
        kind: QuestionKind::RecentChange,
        prompt: "Did anything change on the computer shortly before the problem began? \
                 For example: Windows installed updates, a driver was updated (a driver \
                 is the small program that runs a piece of hardware, like the graphics \
                 card), you installed a new program or game, you added new hardware, or \
                 settings were changed. If nothing changed that you know of, just say \
                 'nothing'.",
        concise: "What changed shortly before — OS/driver updates, new software or \
                  hardware, settings? (or 'nothing')",
    },
    Question {
        kind: QuestionKind::Reproducibility,
        prompt: "Does the problem happen every single time you do the same thing, or \
                 only sometimes? For example: 'every time I log in', 'randomly, a few \
                 times a week', or 'it has only happened once'.",
        concise: "Reproducible every time, intermittent, or a one-off?",
    },
    Question {
        kind: QuestionKind::Scope,
        prompt: "Is the problem limited to one program (only one app or game \
                 misbehaves), or does it affect the whole computer (everything becomes \
                 slow, frozen, or crashes)?",
        concise: "Limited to one program, or system-wide?",
    },
];

/// The intake interview: infer what the initial description already answers,
/// then ask only what is still open.
///
/// ```
/// use intake::Interview;
///
/// let mut interview = Interview::new("my pc is broken");
/// while let Some(question) = interview.next_question() {
///     let answer = "every time i log in"; // from the user
///     interview.answer(question.kind, answer);
/// }
/// let case = interview.into_case();
/// ```
#[derive(Debug, Clone)]
pub struct Interview {
    case: Case,
    asked: Vec<QuestionKind>,
    transcript: Vec<(QuestionKind, String)>,
}

impl Interview {
    /// Start an interview from the user's initial description, inferring every
    /// case field the text already answers.
    pub fn new(statement: &str) -> Self {
        let mut case = Case {
            statement: statement.to_string(),
            symptoms: extract_symptoms(statement),
            onset: Onset::Unknown,
            recent_change: RecentChange::Unknown,
            reproducibility: Reproducibility::Unknown,
            scope: Scope::Unknown,
            fluency: Fluency::Guided,
        };
        infer_into(&mut case, statement);
        case.fluency = assess_fluency(&case);
        Self {
            case,
            asked: Vec::new(),
            transcript: Vec::new(),
        }
    }

    /// The next follow-up question, or `None` when the case is sufficient,
    /// every open field has been asked once, or [`MAX_QUESTIONS`] is reached.
    pub fn next_question(&self) -> Option<Question> {
        if self.asked.len() >= MAX_QUESTIONS {
            return None;
        }
        QUESTIONS
            .iter()
            .find(|q| !self.asked.contains(&q.kind) && self.is_open(q.kind))
            .copied()
    }

    /// Record the user's answer to a question. The answer text is run through
    /// structured extraction (an error code typed here lands in the
    /// signature) and parsed into the matching case field. An unparseable
    /// answer leaves the field unknown — the question is not repeated.
    pub fn answer(&mut self, kind: QuestionKind, text: &str) {
        if !self.asked.contains(&kind) {
            self.asked.push(kind);
        }
        self.transcript.push((kind, text.to_string()));
        merge_symptoms(&mut self.case.symptoms, extract_symptoms(text));
        match kind {
            // The exact-error ask only feeds extraction, above.
            QuestionKind::ExactError => {}
            QuestionKind::Onset => {
                if let Some(onset) = parse_onset(text) {
                    self.case.onset = onset;
                }
            }
            QuestionKind::RecentChange => {
                if let Some(change) = parse_recent_change(text) {
                    self.case.recent_change = change;
                }
            }
            QuestionKind::Reproducibility => {
                if let Some(repro) = parse_reproducibility(text) {
                    self.case.reproducibility = repro;
                }
            }
            QuestionKind::Scope => {
                if let Some(scope) = parse_scope(text) {
                    self.case.scope = scope;
                }
            }
        }
    }

    /// The case as established so far.
    pub fn case(&self) -> &Case {
        &self.case
    }

    /// Question/answer pairs recorded so far, in order. Ticket context only —
    /// the transcript is free text and never enters the corpus.
    pub fn transcript(&self) -> &[(QuestionKind, String)] {
        &self.transcript
    }

    /// Finish the interview and take the case.
    pub fn into_case(self) -> Case {
        self.case
    }

    /// Whether a question's field is still unestablished.
    fn is_open(&self, kind: QuestionKind) -> bool {
        match kind {
            // Exact evidence is "answered" once any code-grade symptom exists:
            // a hex code, an id-bearing term, or a module name.
            QuestionKind::ExactError => !self
                .case
                .symptoms
                .iter()
                .any(|s| s.0.starts_with("0x") || s.0.contains('_') || s.0.ends_with(".exe")),
            QuestionKind::Onset => self.case.onset == Onset::Unknown,
            QuestionKind::RecentChange => self.case.recent_change == RecentChange::Unknown,
            QuestionKind::Reproducibility => self.case.reproducibility == Reproducibility::Unknown,
            QuestionKind::Scope => self.case.scope == Scope::Unknown,
        }
    }
}

/// The question-bank prompt for a question kind in a register: the
/// deterministic wording and the fallback for every model-phrased ask.
/// Guided gets the explain-your-terms form; Technical gets the concise form.
pub fn scripted_prompt(kind: QuestionKind, fluency: Fluency) -> &'static str {
    let question = QUESTIONS
        .iter()
        .find(|q| q.kind == kind)
        .expect("every QuestionKind has a bank entry");
    match fluency {
        Fluency::Guided => question.prompt,
        Fluency::Technical => question.concise,
    }
}

/// What an interviewer must accomplish with a question of this kind. Handed
/// to the model so a sharper phrasing still targets the same case field.
fn goal_for(kind: QuestionKind) -> &'static str {
    match kind {
        QuestionKind::ExactError => {
            "obtain the exact on-screen error text, code, or stop code, typed verbatim"
        }
        QuestionKind::Onset => "establish when the problem began and whether it ever worked",
        QuestionKind::RecentChange => {
            "establish what changed shortly before the problem (updates, new software \
             or hardware, settings)"
        }
        QuestionKind::Reproducibility => {
            "establish whether the problem happens every time, only sometimes, or \
             happened once"
        }
        QuestionKind::Scope => {
            "establish whether the problem is limited to one program or affects the \
             whole machine"
        }
    }
}

/// Phrases follow-up questions. Which field is asked about, in what order,
/// and when the interview stops are decided by [`Interview`] — an interviewer
/// only words the ask, so the funnel stays auditable whichever
/// implementation is in use.
#[async_trait]
pub trait Interviewer: Send + Sync {
    /// The prompt to put to the user for `kind`, given the interview so far.
    /// Infallible by contract: an implementation that cannot produce a
    /// phrasing must fall back to [`scripted_prompt`], never fail the
    /// interview.
    async fn ask(&self, interview: &Interview, kind: QuestionKind) -> String;
}

/// The deterministic question bank: the cold-start interviewer, usable with
/// no inference endpoint.
pub struct ScriptedInterviewer;

#[async_trait]
impl Interviewer for ScriptedInterviewer {
    async fn ask(&self, interview: &Interview, kind: QuestionKind) -> String {
        scripted_prompt(kind, interview.case().fluency).to_string()
    }
}

/// A model-backed interviewer: same funnel, sharper questions. The model sees
/// the original statement, the case so far, and the transcript, and is asked
/// to phrase one plain-language question targeting the current field's goal
/// ("you mentioned it dies during games — does the screen go black instantly,
/// or freeze first?"). Any error, empty reply, over-long reply, or reply that
/// is not a question falls back to the script, so the interview degrades to
/// cold-start behavior rather than failing.
pub struct ModelInterviewer<'a> {
    completer: &'a dyn Completer,
    model: String,
}

/// Longest model phrasing accepted before falling back to the script. Roomy
/// enough for a question that explains its own terms (the scripted questions
/// do too); a follow-up the user must scroll is still worse than the stock
/// question.
const MAX_PROMPT_LEN: usize = 480;

impl<'a> ModelInterviewer<'a> {
    /// Build an interviewer over any [`Completer`] and a model name.
    pub fn new(completer: &'a dyn Completer, model: impl Into<String>) -> Self {
        Self {
            completer,
            model: model.into(),
        }
    }
}

#[async_trait]
impl Interviewer for ModelInterviewer<'_> {
    async fn ask(&self, interview: &Interview, kind: QuestionKind) -> String {
        // Match the register to how reasoned the person's own explanation
        // was: teach a novice, be measured with a fluent reporter.
        let register = match interview.case().fluency {
            Fluency::Guided => {
                "Assume the user does not know technical terms: if one is \
                 unavoidable (stop code, driver, BIOS), say in a few words what it \
                 is and where on the screen to find it, and give a short example of \
                 what an answer might look like."
            }
            Fluency::Technical => {
                "The user's own explanation was precise and technical, so be \
                 measured and direct: one concise question, no definitions, no \
                 examples."
            }
        };
        let system = format!(
            "You are the intake interviewer on a PC support helpdesk. Your goal for \
             this turn: {}. Ask ONE plain-language follow-up question, specific to \
             their situation. {register} Reply with the question only — no preamble, \
             no list.",
            goal_for(kind)
        );
        let mut context = format!(
            "Original request: {}\nEstablished so far: {}",
            interview.case().statement,
            interview.case().brief()
        );
        for (asked_kind, answer) in interview.transcript() {
            context.push_str(&format!("\nAnswer to {asked_kind:?}: {answer}"));
        }

        // No max_tokens cap: a reasoning model spends its budget thinking and
        // returns empty content under a tight cap. The caller's endpoint
        // timeout bounds wall-time instead, and the sanitizer bounds length.
        let request = ChatCompletionRequest::new(
            self.model.clone(),
            vec![ChatMessage::system(system), ChatMessage::user(context)],
        );

        match self.completer.complete(request).await {
            Ok(response) => response
                .choices
                .into_iter()
                .next()
                .map(|choice| choice.message.content)
                .and_then(|content| sanitize_question(&content))
                .unwrap_or_else(|| scripted_prompt(kind, interview.case().fluency).to_string()),
            Err(_) => scripted_prompt(kind, interview.case().fluency).to_string(),
        }
    }
}

/// Accept a model reply only if it reads as one short question: any inline
/// `<think>…</think>` block stripped, first non-empty line, unquoted,
/// question mark present, within length. Anything else returns `None` and the
/// caller falls back to the script.
fn sanitize_question(content: &str) -> Option<String> {
    // Some OpenAI-compatible servers inline the reasoning trace in content.
    let content = match (content.find("<think>"), content.find("</think>")) {
        (Some(start), Some(end)) if start < end => {
            let mut stripped = String::new();
            stripped.push_str(&content[..start]);
            stripped.push_str(&content[end + "</think>".len()..]);
            stripped
        }
        _ => content.to_string(),
    };
    let line = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        .trim_matches('"')
        .trim();
    if line.is_empty() || line.len() > MAX_PROMPT_LEN || !line.contains('?') {
        return None;
    }
    Some(line.to_string())
}

/// Vocabulary that marks *knowledge*: actions taken and tools used that no
/// error screen ever prints. "I reseated the card", "I ran sfc", "checked
/// Event Viewer" cannot be recited off a dialog — they describe what the
/// person did or knows.
const KNOWLEDGE_TERMS: &[&str] = &[
    "driver",
    "bios",
    "uefi",
    "registry",
    "event viewer",
    "event log",
    "device manager",
    "minidump",
    "dump",
    "dmesg",
    "safe mode",
    "clean boot",
    "sfc",
    "dism",
    "firmware",
    "reinstall",
    "rolled back",
    "roll back",
    "reseat",
    "swapped",
    "memtest",
    "ddu",
    "undervolt",
    "underclock",
    "overclock",
    "stress test",
    "clean install",
    "fresh install",
    "reformat",
    "flashed",
];

/// Vocabulary that error screens print verbatim. A person can quote any of
/// these — a WHEA bluescreen, an Xid popup, "display driver nvlddmkm stopped
/// responding" — with no idea what it means, so these are evidence about the
/// *machine*, never about the *reporter*.
const RECITED_TERMS: &[&str] = &[
    "wer",
    "whea",
    "xid",
    "bugcheck",
    "stop code",
    "error code",
    "bsod",
    "kernel",
    "nvlddmkm",
    "dxgi",
];

/// Score how *reasoned* the initial explanation was — not how much it quotes.
///
/// People quote codes without knowing what they mean: a non-technical user
/// will read an NVIDIA error code or a bluescreen's WHEA line off the screen
/// verbatim. That is excellent evidence for diagnosis, but it says nothing
/// about the reporter — so all quoted evidence combined (hex codes,
/// id-bearing terms, module names, screen-printed vocabulary) is capped at
/// one point: it can supplement a reasoned explanation, never substitute for
/// one. What actually scores is what a screen cannot print: the standard
/// intake facts volunteered unprompted (onset, what changed, reproducibility,
/// scope) and knowledge vocabulary — actions taken, tools used.
///
/// The default is [`Fluency::Guided`] and the bar for `Technical` stays
/// high: over-explaining to a fluent reporter is an annoyance, but
/// under-explaining to a novice is a failure.
fn assess_fluency(case: &Case) -> Fluency {
    let lowered = case.statement.to_lowercase();

    // Reasoning signals: cannot be recited off an error screen.
    let knowledge = KNOWLEDGE_TERMS
        .iter()
        .filter(|term| lowered.contains(*term))
        .count();
    let unprompted = [
        case.onset != Onset::Unknown,
        case.recent_change != RecentChange::Unknown,
        case.reproducibility != Reproducibility::Unknown,
        case.scope != Scope::Unknown,
    ]
    .iter()
    .filter(|given| **given)
    .count();

    // Evidence signals: quotable verbatim, so capped at one point total no
    // matter how many codes or screen-terms appear.
    let quoted = case
        .symptoms
        .iter()
        .filter(|s| s.0.starts_with("0x") || s.0.contains('_') || s.0.ends_with(".exe"))
        .count()
        + RECITED_TERMS
            .iter()
            .filter(|term| lowered.contains(*term))
            .count();
    let evidence = quoted.min(1);

    if knowledge + unprompted + evidence >= 4 {
        Fluency::Technical
    } else {
        Fluency::Guided
    }
}

/// Infer every parseable field from a free-text statement.
fn infer_into(case: &mut Case, text: &str) {
    if let Some(onset) = parse_onset(text) {
        case.onset = onset;
    }
    if let Some(change) = parse_recent_change(text) {
        case.recent_change = change;
    }
    if let Some(repro) = parse_reproducibility(text) {
        case.reproducibility = repro;
    }
    if let Some(scope) = parse_scope(text) {
        case.scope = scope;
    }
}

/// Merge new symptoms in, keeping the set sorted and deduplicated so the
/// signature stays deterministic as answers accumulate.
fn merge_symptoms(into: &mut Vec<Symptom>, extra: Vec<Symptom>) {
    into.extend(extra);
    into.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    into.dedup();
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn parse_onset(text: &str) -> Option<Onset> {
    let text = text.to_lowercase();
    if contains_any(&text, &["never worked", "never did", "out of the box"]) {
        Some(Onset::NeverWorked)
    } else if contains_any(
        &text,
        &["worse", "gradual", "slowly", "over time", "for weeks"],
    ) {
        Some(Onset::Gradual)
    } else if contains_any(
        &text,
        &[
            "yesterday",
            "today",
            "last week",
            "this morning",
            "since",
            "until",
            "suddenly",
            // Anchoring the start to an event ("started after the update") is
            // a sudden onset; bare "after" is not, to avoid matching timing
            // descriptions like "crashes after a few minutes".
            "started after",
            "began after",
            "right after",
        ],
    ) {
        Some(Onset::Sudden)
    } else {
        None
    }
}

fn parse_recent_change(text: &str) -> Option<RecentChange> {
    let text = text.to_lowercase();
    // Driver before OS: "driver update" must not match as an OS update.
    if text.contains("driver") {
        Some(RecentChange::DriverUpdate)
    } else if contains_any(
        &text,
        &["windows update", "os update", "update", "upgraded"],
    ) {
        Some(RecentChange::OsUpdate)
    } else if contains_any(
        &text,
        &[
            "new gpu",
            "new card",
            "new ram",
            "new ssd",
            "new drive",
            "new psu",
            "new hardware",
            "plugged in",
        ],
    ) {
        Some(RecentChange::NewHardware)
    } else if contains_any(
        &text,
        &["installed", "new program", "new software", "new app"],
    ) {
        Some(RecentChange::NewSoftware)
    } else if contains_any(&text, &["nothing", "no change", "nope"]) {
        // Negation outranks the generic "changed": "nothing changed" is an
        // answer, not a config change.
        Some(RecentChange::NoChange)
    } else if contains_any(&text, &["setting", "settings", "config", "changed"]) {
        Some(RecentChange::ConfigChange)
    } else {
        None
    }
}

fn parse_reproducibility(text: &str) -> Option<Reproducibility> {
    let text = text.to_lowercase();
    if contains_any(
        &text,
        &["every time", "each time", "always", "consistently", "100%"],
    ) {
        Some(Reproducibility::Always)
    } else if contains_any(
        &text,
        &[
            "sometimes",
            "random",
            "occasional",
            "intermittent",
            "now and then",
        ],
    ) {
        Some(Reproducibility::Intermittent)
    } else if contains_any(&text, &["once", "one time", "first time"]) {
        Some(Reproducibility::Once)
    } else {
        None
    }
}

fn parse_scope(text: &str) -> Option<Scope> {
    let text = text.to_lowercase();
    if contains_any(
        &text,
        &[
            "whole",
            "everything",
            "entire",
            "all programs",
            "whole machine",
            "whole system",
            "whole pc",
        ],
    ) {
        Some(Scope::WholeSystem)
    } else if contains_any(
        &text,
        &[
            ".exe",
            "one program",
            "one app",
            "only when",
            "just one",
            "this game",
            "this program",
            "this app",
        ],
    ) {
        Some(Scope::SingleApp)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vague_statement_triggers_the_full_funnel_in_order() {
        let mut interview = Interview::new("my computer is broken");
        let mut kinds = Vec::new();
        while let Some(question) = interview.next_question() {
            kinds.push(question.kind);
            interview.answer(question.kind, "i don't know");
        }
        assert_eq!(
            kinds,
            vec![
                QuestionKind::ExactError,
                QuestionKind::Onset,
                QuestionKind::RecentChange,
                QuestionKind::Reproducibility,
                QuestionKind::Scope,
            ]
        );
        // Unparseable answers leave fields unknown but are never re-asked.
        assert!(interview.next_question().is_none());
    }

    #[test]
    fn a_well_described_fault_asks_nothing() {
        let interview = Interview::new(
            "explorer.exe crashes every time I log in, started yesterday \
             right after a driver update; the rest of the system is fine, \
             just one program",
        );
        let case = interview.case();
        assert_eq!(case.onset, Onset::Sudden);
        assert_eq!(case.recent_change, RecentChange::DriverUpdate);
        assert_eq!(case.reproducibility, Reproducibility::Always);
        assert_eq!(case.scope, Scope::SingleApp);
        assert!(interview.next_question().is_none(), "nothing left to ask");
    }

    #[test]
    fn answers_fill_the_case_and_close_their_question() {
        let mut interview = Interview::new("pc freezes");
        // The first open question is the exact-error ask.
        let q = interview.next_question().expect("question");
        assert_eq!(q.kind, QuestionKind::ExactError);
        interview.answer(q.kind, "blue screen says 0x00000124");

        let q = interview.next_question().expect("question");
        assert_eq!(q.kind, QuestionKind::Onset);
        interview.answer(q.kind, "it worked fine until yesterday");
        assert_eq!(interview.case().onset, Onset::Sudden);

        let q = interview.next_question().expect("question");
        assert_eq!(q.kind, QuestionKind::RecentChange);
        interview.answer(q.kind, "nothing changed");
        assert_eq!(interview.case().recent_change, RecentChange::NoChange);
    }

    #[test]
    fn an_error_code_typed_in_an_answer_lands_in_the_signature() {
        let mut interview = Interview::new("games keep crashing");
        interview.answer(QuestionKind::ExactError, "WHEA error 0x00000124");
        let case = interview.case();
        assert!(case.symptoms.contains(&Symptom("0x00000124".into())));
        assert!(case.symptoms.contains(&Symptom("whea".into())));
        // ...and the exact-error question is now both asked and answered.
        assert!(interview
            .next_question()
            .is_none_or(|q| q.kind != QuestionKind::ExactError));
    }

    #[test]
    fn the_signature_is_deterministic_across_answer_order() {
        let mut a = Interview::new("crash");
        a.answer(QuestionKind::ExactError, "0x1234");
        a.answer(QuestionKind::Onset, "since the boot loop yesterday");

        let mut b = Interview::new("crash");
        b.answer(QuestionKind::Onset, "since the boot loop yesterday");
        b.answer(QuestionKind::ExactError, "0x1234");

        assert_eq!(
            a.case().signature().fingerprint,
            b.case().signature().fingerprint
        );
    }

    #[test]
    fn the_funnel_is_capped() {
        let mut interview = Interview::new("help");
        let mut count = 0;
        while let Some(question) = interview.next_question() {
            count += 1;
            interview.answer(question.kind, "?");
        }
        assert!(count <= MAX_QUESTIONS);
    }

    #[test]
    fn parsers_map_the_common_phrasings() {
        assert_eq!(parse_onset("it never worked"), Some(Onset::NeverWorked));
        assert_eq!(parse_onset("getting worse over time"), Some(Onset::Gradual));
        assert_eq!(
            parse_recent_change("I updated the GPU driver"),
            Some(RecentChange::DriverUpdate)
        );
        assert_eq!(
            parse_recent_change("windows update last night"),
            Some(RecentChange::OsUpdate)
        );
        assert_eq!(
            parse_recent_change("installed a new program"),
            Some(RecentChange::NewSoftware)
        );
        assert_eq!(
            parse_reproducibility("happens randomly"),
            Some(Reproducibility::Intermittent)
        );
        assert_eq!(
            parse_scope("the whole machine locks up"),
            Some(Scope::WholeSystem)
        );
        assert_eq!(parse_onset("blah"), None);
    }

    use inference::{ChatCompletionResponse, Choice, InferenceError};

    /// A [`Completer`] that replays one fixed reply, or errors.
    struct OneReply(Result<&'static str, ()>);

    #[async_trait]
    impl Completer for OneReply {
        async fn complete(
            &self,
            _request: ChatCompletionRequest,
        ) -> Result<ChatCompletionResponse, InferenceError> {
            match self.0 {
                Ok(content) => Ok(ChatCompletionResponse {
                    model: String::new(),
                    choices: vec![Choice {
                        message: ChatMessage::assistant(content),
                        finish_reason: None,
                    }],
                    usage: None,
                }),
                Err(()) => Err(InferenceError::EmptyResponse),
            }
        }
    }

    #[tokio::test]
    async fn the_scripted_interviewer_words_questions_from_the_bank() {
        let interview = Interview::new("help");
        let prompt = ScriptedInterviewer
            .ask(&interview, QuestionKind::Onset)
            .await;
        assert_eq!(
            prompt,
            scripted_prompt(QuestionKind::Onset, Fluency::Guided)
        );
    }

    #[tokio::test]
    async fn the_model_interviewer_uses_a_well_formed_model_question() {
        let model = OneReply(Ok(
            "You mentioned it dies during games — does the screen go black instantly, \
             or freeze first?",
        ));
        let interviewer = ModelInterviewer::new(&model, "m");
        let interview = Interview::new("pc dies during games");
        let prompt = interviewer.ask(&interview, QuestionKind::ExactError).await;
        assert!(prompt.contains("during games"), "model phrasing used");
    }

    #[tokio::test]
    async fn the_model_interviewer_falls_back_on_error_and_on_non_questions() {
        let interview = Interview::new("pc dies during games");
        let fallback = scripted_prompt(QuestionKind::ExactError, Fluency::Guided);

        // Endpoint error → script.
        let dead = OneReply(Err(()));
        let prompt = ModelInterviewer::new(&dead, "m")
            .ask(&interview, QuestionKind::ExactError)
            .await;
        assert_eq!(prompt, fallback);

        // A statement instead of a question → script.
        let lecturing = OneReply(Ok("You should reboot the machine first."));
        let prompt = ModelInterviewer::new(&lecturing, "m")
            .ask(&interview, QuestionKind::ExactError)
            .await;
        assert_eq!(prompt, fallback);

        // An empty reply → script.
        let mute = OneReply(Ok("   \n"));
        let prompt = ModelInterviewer::new(&mute, "m")
            .ask(&interview, QuestionKind::ExactError)
            .await;
        assert_eq!(prompt, fallback);
    }

    #[test]
    fn sanitize_takes_one_clean_question_line() {
        assert_eq!(
            sanitize_question("\n  \"Does it crash every time?\"  \nExtra prose."),
            Some("Does it crash every time?".to_string())
        );
        assert_eq!(sanitize_question("Reboot it."), None);
        let long = format!("{}?", "x".repeat(MAX_PROMPT_LEN + 1));
        assert_eq!(sanitize_question(&long), None);
    }

    #[test]
    fn sanitize_strips_an_inline_reasoning_block() {
        assert_eq!(
            sanitize_question(
                "<think>The user mentioned games? I should ask about the screen.</think>\n\
                 Does the screen go black instantly, or freeze first?"
            ),
            Some("Does the screen go black instantly, or freeze first?".to_string())
        );
        // Reasoning with no question after it falls back.
        assert_eq!(
            sanitize_question("<think>Is this right? Hmm.</think>\nReboot the machine."),
            None
        );
    }

    #[test]
    fn questions_explain_their_own_terms_for_non_technical_users() {
        // The exact-error ask must explain what a stop code is and where it
        // appears — the user may have never heard the term.
        let exact = scripted_prompt(QuestionKind::ExactError, Fluency::Guided);
        assert!(exact.contains("blue screen"));
        assert!(exact.contains("Stop code"));
        assert!(exact.contains("CRITICAL_PROCESS_DIED"), "needs an example");
        // The recent-change ask must explain what a driver is.
        let change = scripted_prompt(QuestionKind::RecentChange, Fluency::Guided);
        assert!(change.contains("a driver is"), "jargon must be defined");
        // Every question carries at least one concrete example.
        for question in QUESTIONS {
            assert!(
                question.prompt.contains("for example")
                    || question.prompt.contains("For example")
                    || question.prompt.contains("(only one app"),
                "question without an example: {}",
                question.prompt
            );
        }
    }

    #[test]
    fn a_vague_explanation_gets_the_guided_register() {
        for statement in [
            "my computer is broken",
            "my game is stuttering",
            "my pc crashes randomly during games and browsing",
            "pc keeps dying, blue screen sometimes, whole machine restarts",
            "everything is slow lately",
        ] {
            assert_eq!(
                Interview::new(statement).case().fluency,
                Fluency::Guided,
                "{statement:?} should be guided"
            );
        }
    }

    #[test]
    fn quoting_codes_off_the_screen_is_caught_and_stays_guided() {
        // People recite exactly what the screen showed — an NVIDIA error
        // code, a WHEA bluescreen line, a crash dialog — without knowing what
        // any of it means. That is evidence about the machine, not fluency in
        // the reporter: it still drives the signature and the routing, but it
        // must not flip the register.
        for statement in [
            "my screen goes black and a popup said display driver nvlddmkm \
             stopped responding",
            "blue screen said WHEA_UNCORRECTABLE_ERROR",
            "it shows nvidia error code 43, no idea what that means",
            "explorer.exe crashes on login with WER bucket 0x1234",
            "game crashed and the box said Xid 79 something",
        ] {
            let case = Interview::new(statement).into_case();
            assert_eq!(
                case.fluency,
                Fluency::Guided,
                "recitation must not read as fluency: {statement:?}"
            );
        }
        // ...while the same evidence still reaches the diagnosis: the codes
        // land in the signature even though the register stays guided.
        let case = Interview::new("blue screen said WHEA_UNCORRECTABLE_ERROR").into_case();
        assert!(
            !case.symptoms.is_empty(),
            "the quoted evidence must still be extracted"
        );
    }

    #[test]
    fn a_reasoned_explanation_gets_the_technical_register() {
        for statement in [
            // The helpdesk facts volunteered unprompted, with causal anchoring.
            "explorer.exe crashes every time I log in, started yesterday right \
             after a driver update; just one program",
            "BSOD 0x00000124, WHEA, started after a BIOS update, reproducible \
             every time under load",
            // Actions taken — vocabulary no error screen ever prints.
            "ran sfc and dism already; explorer.exe still crashes every time I \
             log in since yesterday's update",
            // The fluent counterpart of the code-reciter: same Xid evidence,
            // but with diagnostic work behind it.
            "Xid 79 in dmesg, GPU falls off the bus under load; reseated the \
             card and swapped PSU cables, still happens",
        ] {
            assert_eq!(
                Interview::new(statement).case().fluency,
                Fluency::Technical,
                "{statement:?} should be technical"
            );
        }
    }

    #[test]
    fn the_register_changes_the_wording_but_not_the_funnel() {
        // Same field, two registers: the concise form skips the teaching...
        let guided = scripted_prompt(QuestionKind::ExactError, Fluency::Guided);
        let concise = scripted_prompt(QuestionKind::ExactError, Fluency::Technical);
        assert!(guided.contains("blue screen with white text"));
        assert!(!concise.contains("blue screen with white text"));
        assert!(concise.len() < guided.len() / 2);
        // ...but the funnel itself is identical: a technical-register
        // interview still asks the same kinds in the same order.
        let mut interview = Interview::new(
            "BSOD 0x00000124, WHEA, kernel dump points at firmware; \
             driver and BIOS current",
        );
        assert_eq!(interview.case().fluency, Fluency::Technical);
        let mut kinds = Vec::new();
        while let Some(question) = interview.next_question() {
            kinds.push(question.kind);
            interview.answer(question.kind, "?");
        }
        // ExactError already answered by the codes; the rest still asked.
        assert!(kinds.contains(&QuestionKind::Reproducibility));
        assert!(kinds.contains(&QuestionKind::Scope));
    }

    #[tokio::test]
    async fn the_scripted_interviewer_matches_the_register_to_the_case() {
        let novice = Interview::new("my computer is broken");
        let prompt = ScriptedInterviewer.ask(&novice, QuestionKind::Onset).await;
        assert!(prompt.contains("For example"), "guided form expected");

        let fluent = Interview::new(
            "explorer.exe crashes on login with WER bucket 0x1234 since a \
             driver update",
        );
        let prompt = ScriptedInterviewer.ask(&fluent, QuestionKind::Onset).await;
        assert_eq!(
            prompt,
            scripted_prompt(QuestionKind::Onset, Fluency::Technical)
        );
    }

    #[test]
    fn a_case_is_established_only_when_every_field_is_known() {
        let vague = Interview::new("my computer is broken");
        assert!(!vague.case().is_established());

        let full = Interview::new(
            "explorer.exe crashes every time I log in, started yesterday \
             right after a driver update; just one program",
        );
        assert!(full.case().is_established());
    }

    #[test]
    fn the_transcript_records_answers_in_order() {
        let mut interview = Interview::new("pc broken");
        interview.answer(QuestionKind::ExactError, "0x1234");
        interview.answer(QuestionKind::Onset, "since yesterday");
        let transcript = interview.transcript();
        assert_eq!(transcript.len(), 2);
        assert_eq!(transcript[0], (QuestionKind::ExactError, "0x1234".into()));
        assert_eq!(transcript[1].0, QuestionKind::Onset);
    }

    #[test]
    fn the_brief_is_built_only_from_enums() {
        let interview = Interview::new("DESKTOP-NATHAN01 owned by nathan is slow");
        let brief = interview.case().brief();
        assert!(
            !brief.to_lowercase().contains("nathan"),
            "brief leaked: {brief}"
        );
        assert!(brief.contains("onset="));
    }
}
