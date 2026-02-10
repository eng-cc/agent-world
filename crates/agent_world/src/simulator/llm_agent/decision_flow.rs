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

#[derive(Debug, Deserialize)]
struct RawLlmExecuteUntilPayload {
    decision: String,
    action: serde_json::Value,
    until: RawLlmExecuteUntilUntil,
    #[serde(default)]
    max_ticks: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawLlmExecuteUntilUntil {
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    event_any_of: Vec<String>,
    #[serde(default)]
    value_lte: Option<i64>,
}

#[derive(Debug, Clone)]
pub(super) struct LlmDecisionDraft {
    pub decision: AgentDecision,
    pub confidence: Option<f64>,
    pub need_verify: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExecuteUntilEventKind {
    ActionRejected,
    NewVisibleAgent,
    NewVisibleLocation,
    ArriveTarget,
    InsufficientElectricity,
    ThermalOverload,
    HarvestYieldBelow,
    HarvestAvailableBelow,
}

impl ExecuteUntilEventKind {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "action_rejected" => Some(Self::ActionRejected),
            "new_visible_agent" => Some(Self::NewVisibleAgent),
            "new_visible_location" => Some(Self::NewVisibleLocation),
            "arrive_target" => Some(Self::ArriveTarget),
            "insufficient_electricity" => Some(Self::InsufficientElectricity),
            "thermal_overload" => Some(Self::ThermalOverload),
            "harvest_yield_below" => Some(Self::HarvestYieldBelow),
            "harvest_available_below" => Some(Self::HarvestAvailableBelow),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ActionRejected => "action_rejected",
            Self::NewVisibleAgent => "new_visible_agent",
            Self::NewVisibleLocation => "new_visible_location",
            Self::ArriveTarget => "arrive_target",
            Self::InsufficientElectricity => "insufficient_electricity",
            Self::ThermalOverload => "thermal_overload",
            Self::HarvestYieldBelow => "harvest_yield_below",
            Self::HarvestAvailableBelow => "harvest_available_below",
        }
    }

    pub(super) fn requires_value_lte(self) -> bool {
        matches!(self, Self::HarvestYieldBelow | Self::HarvestAvailableBelow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecuteUntilCondition {
    pub kind: ExecuteUntilEventKind,
    pub value_lte: Option<i64>,
}

impl ExecuteUntilCondition {
    pub(super) fn summary(&self) -> String {
        if let Some(value_lte) = self.value_lte {
            format!("{}<= {}", self.kind.as_str(), value_lte)
        } else {
            self.kind.as_str().to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ExecuteUntilDirective {
    pub action: Action,
    pub until_conditions: Vec<ExecuteUntilCondition>,
    pub max_ticks: u64,
}

#[derive(Debug)]
pub(super) enum ParsedLlmTurn {
    Plan(LlmPlanPayload),
    DecisionDraft(LlmDecisionDraft),
    Decision(AgentDecision, Option<String>),
    ExecuteUntil(ExecuteUntilDirective),
    ModuleCall(LlmModuleCallRequest),
    Invalid(String),
}

pub(super) fn parse_llm_turn_responses(output: &str, agent_id: &str) -> Vec<ParsedLlmTurn> {
    let blocks = extract_json_blocks(output);
    if blocks.is_empty() {
        return vec![parse_llm_turn_response(output, agent_id)];
    }

    blocks
        .into_iter()
        .map(
            |json| match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => parse_llm_turn_value(value, agent_id),
                Err(err) => ParsedLlmTurn::Invalid(format!("json parse failed: {err}")),
            },
        )
        .collect()
}

pub(super) fn parse_llm_turn_response(output: &str, agent_id: &str) -> ParsedLlmTurn {
    let json = extract_json_block(output).unwrap_or(output);
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => {
            return ParsedLlmTurn::Invalid(format!("json parse failed: {err}"));
        }
    };

    parse_llm_turn_value(value, agent_id)
}

fn parse_llm_turn_value(value: serde_json::Value, agent_id: &str) -> ParsedLlmTurn {
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

    if value
        .get("decision")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("execute_until"))
    {
        return match parse_execute_until_decision(value, agent_id) {
            Ok(directive) => ParsedLlmTurn::ExecuteUntil(directive),
            Err(err) => ParsedLlmTurn::Invalid(err),
        };
    }

    let (decision, parse_error) = parse_llm_decision_value_with_error(value, agent_id);
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
    let raw_value = value.clone();
    let payload = serde_json::from_value::<RawLlmDecisionDraftPayload>(value)
        .map_err(|err| format!("decision_draft parse failed: {err}"))?;

    let decision_value = if payload.decision.is_object() {
        payload.decision
    } else {
        decision_draft_shorthand_value(&raw_value).unwrap_or(payload.decision)
    };

    let decision_json = serde_json::to_string(&decision_value)
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

fn decision_draft_shorthand_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    let mut decision_object = value.as_object()?.clone();
    if !decision_object
        .get("decision")
        .is_some_and(|value| value.is_string())
    {
        return None;
    }

    decision_object.remove("type");
    decision_object.remove("confidence");
    decision_object.remove("need_verify");
    Some(serde_json::Value::Object(decision_object))
}

fn parse_execute_until_decision(
    value: serde_json::Value,
    agent_id: &str,
) -> Result<ExecuteUntilDirective, String> {
    const DEFAULT_MAX_TICKS: u64 = 6;
    const MAX_TICKS_CAP: u64 = 256;

    let payload = serde_json::from_value::<RawLlmExecuteUntilPayload>(value)
        .map_err(|err| format!("execute_until parse failed: {err}"))?;

    if !payload
        .decision
        .trim()
        .eq_ignore_ascii_case("execute_until")
    {
        return Err("execute_until missing decision=execute_until".to_string());
    }

    let (action_decision, action_parse_error) =
        parse_llm_decision_value_with_error(payload.action, agent_id);
    if let Some(err) = action_parse_error {
        return Err(format!("execute_until invalid action: {err}"));
    }
    let action = match action_decision {
        AgentDecision::Act(action) => action,
        _ => {
            return Err("execute_until action must be actionable decision".to_string());
        }
    };

    let until_conditions = parse_execute_until_conditions(&payload.until)?;

    let max_ticks = payload
        .max_ticks
        .unwrap_or(DEFAULT_MAX_TICKS)
        .clamp(1, MAX_TICKS_CAP);

    Ok(ExecuteUntilDirective {
        action,
        until_conditions,
        max_ticks,
    })
}

fn parse_execute_until_conditions(
    until: &RawLlmExecuteUntilUntil,
) -> Result<Vec<ExecuteUntilCondition>, String> {
    let mut values = Vec::new();
    if let Some(event) = until.event.as_ref() {
        values.push(event.as_str());
    }
    for event in until.event_any_of.iter() {
        values.push(event.as_str());
    }

    let mut conditions = Vec::new();
    for value in values {
        for token in value.split(['|', ',']) {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                continue;
            }
            let kind = ExecuteUntilEventKind::parse(trimmed)
                .ok_or_else(|| format!("execute_until unsupported until.event: {trimmed}"))?;
            let value_lte = if kind.requires_value_lte() {
                let Some(value_lte) = until.value_lte else {
                    return Err(format!(
                        "execute_until event {} requires until.value_lte",
                        kind.as_str()
                    ));
                };
                if value_lte < 0 {
                    return Err(format!(
                        "execute_until until.value_lte must be non-negative for {}",
                        kind.as_str()
                    ));
                }
                Some(value_lte)
            } else {
                None
            };
            let condition = ExecuteUntilCondition { kind, value_lte };
            if !conditions.contains(&condition) {
                conditions.push(condition);
            }
        }
    }

    if conditions.is_empty() {
        return Err("execute_until missing until.event/event_any_of".to_string());
    }

    Ok(conditions)
}

fn parse_llm_decision_with_error(output: &str, agent_id: &str) -> (AgentDecision, Option<String>) {
    let json = extract_json_block(output).unwrap_or(output);
    let value = match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => value,
        Err(err) => {
            return (
                AgentDecision::Wait,
                Some(format!("json parse failed: {err}")),
            );
        }
    };

    parse_llm_decision_value_with_error(value, agent_id)
}

fn parse_llm_decision_value_with_error(
    value: serde_json::Value,
    agent_id: &str,
) -> (AgentDecision, Option<String>) {
    let parsed = match serde_json::from_value::<LlmDecisionPayload>(value) {
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
    let start = next_json_start(raw, 0)?;
    let (_, end) = extract_json_block_from(raw, start)?;
    raw.get(start..=end)
}

fn extract_json_blocks(raw: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut cursor = 0_usize;

    while let Some(start) = next_json_start(raw, cursor) {
        let Some((_, end)) = extract_json_block_from(raw, start) else {
            break;
        };
        if let Some(block) = raw.get(start..=end) {
            blocks.push(block);
        }
        cursor = end.saturating_add(1);
    }

    blocks
}

fn next_json_start(raw: &str, from: usize) -> Option<usize> {
    raw.get(from..)?
        .char_indices()
        .find_map(|(offset, ch)| match ch {
            '{' | '[' => Some(from + offset),
            _ => None,
        })
}

fn extract_json_block_from(raw: &str, start: usize) -> Option<(usize, usize)> {
    let open_char = raw.get(start..)?.chars().next()?;
    if open_char != '{' && open_char != '[' {
        return None;
    }
    let close_char = if open_char == '{' { '}' } else { ']' };

    let mut depth: u32 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in raw[start..].char_indices() {
        let index = start + offset;
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            c if c == open_char => depth = depth.saturating_add(1),
            c if c == close_char => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((start, index));
                }
            }
            _ => {}
        }
    }

    None
}
