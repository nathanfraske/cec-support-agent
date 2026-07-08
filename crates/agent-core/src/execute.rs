use common::{ExecutionResult, Plan, StepResult};
use provenance::{ProvenanceError, SignedPlan, SigningKey};

use crate::consent::Consent;
use crate::dispatch::Dispatcher;
use crate::eula::EulaAcceptances;

/// Execute a judge-signed plan: verify the signature, then run the plan.
///
/// This is the agent-neutrality enforcement point — a plan enters the
/// on-machine zone only through the judge, and the proof is checked here, in
/// code, before any step runs. A plan that was modified after signing, signed
/// with another key, or never signed at all is refused without touching a
/// single tool.
pub async fn execute_signed_plan(
    dispatcher: &Dispatcher,
    signed: &SignedPlan,
    key: &SigningKey,
    consent: Consent,
    accepted_eulas: &EulaAcceptances,
) -> Result<ExecutionResult, ProvenanceError> {
    key.verify(signed)?;
    Ok(execute_plan(dispatcher, &signed.plan, consent, accepted_eulas).await)
}

/// Execute every step of `plan`, in order, through `dispatcher` under `consent`.
///
/// Each step's [`action`](common::PlanStep::action) is dispatched as a tool
/// name. Execution stops at the first step the consent gate refuses, the EULA
/// gate refuses, or that fails, so a remediation never keeps applying changes
/// after an error; a plan whose tools all succeed runs to completion. Plan
/// steps carry no arguments in this bootstrap, so tools receive `null` and apply
/// their own defaults.
///
/// Two gates run before each step. The consent gate (in `dispatch`) refuses a
/// step whose risk exceeds the grant. The EULA gate (here, BEFORE dispatch)
/// refuses a step whose tool installs license-bearing software unless the user
/// accepted that license on screen (`accepted_eulas`) — the engine never
/// accepts a EULA for the user, so a EULA-bearing install with no recorded
/// acceptance stops the plan and the installer never runs.
pub async fn execute_plan(
    dispatcher: &Dispatcher,
    plan: &Plan,
    consent: Consent,
    accepted_eulas: &EulaAcceptances,
) -> ExecutionResult {
    let mut result = ExecutionResult::new(&plan.id);
    for (index, step) in plan.steps.iter().enumerate() {
        // EULA gate — checked BEFORE dispatch, so a license the user did not
        // accept on screen never reaches the installer. This is a liability
        // boundary, not a risk one: the shop must not accept a license for the
        // user, so absence of acceptance fails closed (refuse + stop).
        if let Some(eula) = dispatcher.eula_of(&step.action) {
            if !accepted_eulas.accepted(eula) {
                result.steps.push(StepResult {
                    step: index + 1,
                    action: step.action.clone(),
                    ok: false,
                    summary: format!(
                        "installation refused: '{eula}' requires the user to accept its \
                         license agreement on screen, which was not done"
                    )
                    .into(),
                });
                return result;
            }
        }
        let (ok, summary) = match dispatcher
            .dispatch(&step.action, serde_json::Value::Null, consent)
            .await
        {
            Ok(outcome) => (outcome.ok, outcome.summary),
            Err(error) => (false, error.to_string()),
        };
        result.steps.push(StepResult {
            step: index + 1,
            action: step.action.clone(),
            ok,
            summary: summary.into(),
        });
        if !ok {
            return result;
        }
    }
    result.completed = true;
    result
}
