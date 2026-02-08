use crate::models::BodyKernelView;
use crate::simulator::ResourceKind;

use super::super::events::{Action, ActionEnvelope};
use super::super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput,
};
use super::{
    decode_input, failure, finalize_output, BuiltinModule, M1_BODY_ACTION_COST_ELECTRICITY,
};

#[derive(Debug, Clone)]
pub struct M1BodyModule {
    cost_electricity: i64,
}

impl Default for M1BodyModule {
    fn default() -> Self {
        Self {
            cost_electricity: M1_BODY_ACTION_COST_ELECTRICITY,
        }
    }
}

impl M1BodyModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let (agent_id, kind, payload) = match action {
            Action::BodyAction {
                agent_id,
                kind,
                payload,
            } => (agent_id, kind, payload),
            _ => {
                return finalize_output(
                    ModuleOutput {
                        new_state: None,
                        effects: Vec::new(),
                        emits: Vec::new(),
                        output_bytes: 0,
                    },
                    request,
                );
            }
        };

        let mut decision = RuleDecision {
            action_id: id,
            verdict: RuleVerdict::Allow,
            override_action: None,
            cost: ResourceDelta::default(),
            notes: Vec::new(),
        };

        let view: BodyKernelView = match serde_json::from_value(payload) {
            Ok(view) => view,
            Err(err) => {
                decision.verdict = RuleVerdict::Deny;
                decision
                    .notes
                    .push(format!("body action payload decode failed: {err}"));
                return finalize_output(
                    ModuleOutput {
                        new_state: None,
                        effects: Vec::new(),
                        emits: vec![ModuleEmit {
                            kind: "rule.decision".to_string(),
                            payload: serde_json::to_value(decision).map_err(|err| {
                                failure(
                                    request,
                                    ModuleCallErrorCode::InvalidOutput,
                                    format!("rule decision encode failed: {err}"),
                                )
                            })?,
                        }],
                        output_bytes: 0,
                    },
                    request,
                );
            }
        };

        decision.verdict = RuleVerdict::Modify;
        decision.override_action = Some(Action::EmitBodyAttributes {
            agent_id,
            view,
            reason: format!("body.{kind}"),
        });

        if self.cost_electricity > 0 {
            decision
                .cost
                .entries
                .insert(ResourceKind::Electricity, -self.cost_electricity);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "rule.decision".to_string(),
                    payload: serde_json::to_value(decision).map_err(|err| {
                        failure(
                            request,
                            ModuleCallErrorCode::InvalidOutput,
                            format!("rule decision encode failed: {err}"),
                        )
                    })?,
                }],
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1BodyModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}
