use super::super::agent::AgentDecision;
use super::super::types::Action;
use super::prompt_assembly::{PromptSectionKind, PromptSectionPriority};
use serde::{Deserialize, Serialize};

pub(super) fn parse_limit_arg(
    value: Option<&serde_json::Value>,
    default: usize,
    max: usize,
) -> usize {
    value
        .and_then(|value| value.as_u64())
        .map(|value| value.clamp(1, max as u64) as usize)
        .unwrap_or(default)
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ModuleCallExchange {
    pub module: String,
    pub args: serde_json::Value,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecisionPhase {
    Plan,
    ModuleLoop,
    DecisionDraft,
    Finalize,
}

impl DecisionPhase {
    pub(super) fn step_type(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::ModuleLoop => "module_call",
            Self::DecisionDraft => "decision_draft",
            Self::Finalize => "final_decision",
        }
    }

    pub(super) fn prompt_instruction(self) -> &'static str {
        match self {
            Self::Plan => {
                "本轮是 plan 阶段：优先输出 {\"type\":\"plan\",...}；若信息充足可直接输出最终 decision JSON。"
            }
            Self::ModuleLoop => {
                "本轮是 module_call 阶段：优先输出 module_call；若信息充分可输出 decision_draft 或最终 decision JSON。"
            }
            Self::DecisionDraft => {
                "本轮是 decision_draft 阶段：请优先输出 {\"type\":\"decision_draft\",...}，随后进入最终决策。"
            }
            Self::Finalize => {
                "本轮是 final_decision 阶段：请只输出最终 decision JSON，不要输出额外文本。"
            }
        }
    }
}

pub(super) fn prompt_section_kind_name(kind: PromptSectionKind) -> &'static str {
    match kind {
        PromptSectionKind::Policy => "policy",
        PromptSectionKind::Goals => "goals",
        PromptSectionKind::Context => "context",
        PromptSectionKind::Tools => "tools",
        PromptSectionKind::History => "history",
        PromptSectionKind::Memory => "memory",
        PromptSectionKind::OutputSchema => "output_schema",
        PromptSectionKind::StepMeta => "step_meta",
    }
}

pub(super) fn prompt_section_priority_name(priority: PromptSectionPriority) -> &'static str {
    match priority {
        PromptSectionPriority::High => "high",
        PromptSectionPriority::Medium => "medium",
        PromptSectionPriority::Low => "low",
    }
}

pub(super) fn summarize_trace_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max_chars {
            break;
        }
        truncated.push(ch);
    }
    truncated.push_str("...(truncated)");
    truncated
}

pub(super) fn serialize_decision_for_prompt(decision: &AgentDecision) -> String {
    let value = match decision {
        AgentDecision::Wait => serde_json::json!({ "decision": "wait" }),
        AgentDecision::WaitTicks(ticks) => serde_json::json!({
            "decision": "wait_ticks",
            "ticks": ticks,
        }),
        AgentDecision::Act(Action::MoveAgent { to, .. }) => serde_json::json!({
            "decision": "move_agent",
            "to": to,
        }),
        AgentDecision::Act(Action::HarvestRadiation { max_amount, .. }) => serde_json::json!({
            "decision": "harvest_radiation",
            "max_amount": max_amount,
        }),
        AgentDecision::Act(_) => serde_json::json!({ "decision": "wait" }),
    };

    serde_json::to_string(&value).unwrap_or_else(|_| "{\"decision\":\"wait\"}".to_string())
}

#[derive(Debug, Deserialize)]
struct LlmDecisionPayload {
    decision: String,
    ticks: Option<u64>,
    to: Option<String>,
    max_amount: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlmModuleCallRequest {
    pub module: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(super) struct LlmPlanPayload {
    #[serde(default)]
    pub missing: Vec<String>,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawLlmDecisionDraftPayload {
    decision: serde_json::Value,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    need_verify: Option<bool>,
}

#[derive(Debug, Clone)]
pub(super) struct LlmDecisionDraft {
    pub decision: AgentDecision,
    pub confidence: Option<f64>,
    pub need_verify: bool,
}

#[derive(Debug)]
pub(super) enum ParsedLlmTurn {
    Plan(LlmPlanPayload),
    DecisionDraft(LlmDecisionDraft),
    Decision(AgentDecision, Option<String>),
    ModuleCall(LlmModuleCallRequest),
    Invalid(String),
}

pub(super) fn parse_llm_turn_response(output: &str, agent_id: &str) -> ParsedLlmTurn {
    let json = extract_json_block(output).unwrap_or(output);
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => {
            return ParsedLlmTurn::Invalid(format!("json parse failed: {err}"));
        }
    };

    if let Some(turn_type) = value
        .get("type")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_ascii_lowercase())
    {
        return match turn_type.as_str() {
            "module_call" => match serde_json::from_value::<LlmModuleCallRequest>(value) {
                Ok(request) => {
                    if request.module.trim().is_empty() {
                        ParsedLlmTurn::Invalid("module_call missing `module`".to_string())
                    } else {
                        ParsedLlmTurn::ModuleCall(request)
                    }
                }
                Err(err) => ParsedLlmTurn::Invalid(format!("module_call parse failed: {err}")),
            },
            "plan" => match serde_json::from_value::<LlmPlanPayload>(value) {
                Ok(plan) => ParsedLlmTurn::Plan(plan),
                Err(err) => ParsedLlmTurn::Invalid(format!("plan parse failed: {err}")),
            },
            "decision_draft" => match parse_llm_decision_draft(value, agent_id) {
                Ok(draft) => ParsedLlmTurn::DecisionDraft(draft),
                Err(err) => ParsedLlmTurn::Invalid(err),
            },
            other => ParsedLlmTurn::Invalid(format!("unsupported turn type: {other}")),
        };
    }

    let (decision, parse_error) = parse_llm_decision_with_error(json, agent_id);
    if let Some(err) = parse_error {
        ParsedLlmTurn::Invalid(err)
    } else {
        ParsedLlmTurn::Decision(decision, None)
    }
}

fn parse_llm_decision_draft(
    value: serde_json::Value,
    agent_id: &str,
) -> Result<LlmDecisionDraft, String> {
    let payload = serde_json::from_value::<RawLlmDecisionDraftPayload>(value)
        .map_err(|err| format!("decision_draft parse failed: {err}"))?;

    let decision_json = serde_json::to_string(&payload.decision)
        .map_err(|err| format!("decision_draft serialize failed: {err}"))?;
    let (decision, parse_error) = parse_llm_decision_with_error(decision_json.as_str(), agent_id);
    if let Some(err) = parse_error {
        return Err(format!("decision_draft invalid decision: {err}"));
    }

    Ok(LlmDecisionDraft {
        decision,
        confidence: payload.confidence,
        need_verify: payload.need_verify.unwrap_or(true),
    })
}

fn parse_llm_decision_with_error(output: &str, agent_id: &str) -> (AgentDecision, Option<String>) {
    let json = extract_json_block(output).unwrap_or(output);
    let parsed = match serde_json::from_str::<LlmDecisionPayload>(json) {
        Ok(value) => value,
        Err(err) => {
            return (
                AgentDecision::Wait,
                Some(format!("json parse failed: {err}")),
            );
        }
    };

    let decision = match parsed.decision.trim().to_ascii_lowercase().as_str() {
        "wait" => AgentDecision::Wait,
        "wait_ticks" => AgentDecision::WaitTicks(parsed.ticks.unwrap_or(1).max(1)),
        "move_agent" => {
            let to = parsed.to.unwrap_or_default();
            if to.trim().is_empty() {
                return (
                    AgentDecision::Wait,
                    Some("move_agent missing `to`".to_string()),
                );
            }
            AgentDecision::Act(Action::MoveAgent {
                agent_id: agent_id.to_string(),
                to,
            })
        }
        "harvest_radiation" => {
            let max_amount = parsed.max_amount.unwrap_or(1);
            if max_amount <= 0 {
                return (
                    AgentDecision::Wait,
                    Some("harvest_radiation requires positive max_amount".to_string()),
                );
            }
            AgentDecision::Act(Action::HarvestRadiation {
                agent_id: agent_id.to_string(),
                max_amount,
            })
        }
        other => {
            return (
                AgentDecision::Wait,
                Some(format!("unsupported decision: {other}")),
            );
        }
    };

    (decision, None)
}

fn extract_json_block(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end < start {
        return None;
    }
    raw.get(start..=end)
}
