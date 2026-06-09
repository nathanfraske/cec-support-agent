use common::{ExecutionResult, Plan, StepResult};
use provenance::{ProvenanceError, SignedPlan, SigningKey};

use crate::consent::Consent;
use crate::dispatch::Dispatcher;

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
) -> Result<ExecutionResult, ProvenanceError> {
    key.verify(signed)?;
    Ok(execute_plan(dispatcher, &signed.plan, consent).await)
}

/// Execute every step of `plan`, in order, through `dispatcher` under `consent`.
///
/// Each step's [`action`](common::PlanStep::action) is dispatched as a tool
/// name. Execution stops at the first step the consent gate refuses or that
/// fails, so a remediation never keeps applying changes after an error; a plan
/// whose tools all succeed runs to completion. Plan steps carry no arguments in
/// this bootstrap, so tools receive `null` and apply their own defaults.
pub async fn execute_plan(
    dispatcher: &Dispatcher,
    plan: &Plan,
    consent: Consent,
) -> ExecutionResult {
    let mut result = ExecutionResult::new(&plan.id);
    for (index, step) in plan.steps.iter().enumerate() {
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
            summary,
        });
        if !ok {
            return result;
        }
    }
    result.completed = true;
    result
}
