use serde::Serialize;

const DEFAULT_CONTEXT_WINDOW_TOKENS: usize = 8_192;
const DEFAULT_RESERVED_OUTPUT_TOKENS: usize = 1_024;
const DEFAULT_SAFETY_MARGIN_TOKENS: usize = 512;
const MIN_EFFECTIVE_INPUT_BUDGET_TOKENS: usize = 256;
const HISTORY_SOFT_CAP_TOKENS: usize = 256;
const MEMORY_SOFT_CAP_TOKENS: usize = 192;
const CONTEXT_MIN_TOKENS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptBudget {
    pub context_window_tokens: usize,
    pub reserved_output_tokens: usize,
    pub safety_margin_tokens: usize,
}

impl Default for PromptBudget {
    fn default() -> Self {
        Self {
            context_window_tokens: DEFAULT_CONTEXT_WINDOW_TOKENS,
            reserved_output_tokens: DEFAULT_RESERVED_OUTPUT_TOKENS,
            safety_margin_tokens: DEFAULT_SAFETY_MARGIN_TOKENS,
        }
    }
}

impl PromptBudget {
    pub fn effective_input_budget_tokens(&self) -> usize {
        self.context_window_tokens
            .saturating_sub(self.reserved_output_tokens)
            .saturating_sub(self.safety_margin_tokens)
            .max(MIN_EFFECTIVE_INPUT_BUDGET_TOKENS)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptAssemblyInput<'a> {
    pub agent_id: &'a str,
    pub base_system_prompt: &'a str,
    pub short_term_goal: &'a str,
    pub long_term_goal: &'a str,
    pub observation_json: &'a str,
    pub module_history_json: &'a str,
    pub memory_digest: Option<&'a str>,
    pub step_context: PromptStepContext,
    pub prompt_budget: PromptBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PromptStepContext {
    pub step_index: usize,
    pub max_steps: usize,
    pub module_calls_used: usize,
    pub module_calls_max: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSectionKind {
    Policy,
    Goals,
    Context,
    Tools,
    History,
    Memory,
    OutputSchema,
    StepMeta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSectionPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptSection {
    pub kind: PromptSectionKind,
    pub priority: PromptSectionPriority,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptSectionTrace {
    pub kind: PromptSectionKind,
    pub priority: PromptSectionPriority,
    pub included: bool,
    pub estimated_tokens: usize,
    pub emitted_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptAssemblyOutput {
    pub system_prompt: String,
    pub user_prompt: String,
    pub sections: Vec<PromptSection>,
    pub section_trace: Vec<PromptSectionTrace>,
    pub effective_input_budget_tokens: usize,
    pub estimated_input_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SectionState {
    section: PromptSection,
    required: bool,
    included: bool,
    estimated_tokens: usize,
}

impl SectionState {
    fn new(section: PromptSection, required: bool) -> Self {
        let estimated_tokens = estimate_tokens(section.content.as_str());
        Self {
            section,
            required,
            included: true,
            estimated_tokens,
        }
    }

    fn emitted_tokens(&self) -> usize {
        if self.included {
            estimate_tokens(self.section.content.as_str())
        } else {
            0
        }
    }
}

pub struct PromptAssembler;

impl PromptAssembler {
    pub fn assemble(input: PromptAssemblyInput<'_>) -> PromptAssemblyOutput {
        let mut sections = Vec::new();
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::Policy,
                priority: PromptSectionPriority::High,
                content: format!(
                    "{}\n\n你是一个硅基文明 Agent。请严格输出 JSON，不要输出额外文字。",
                    input.base_system_prompt,
                ),
            },
            true,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::Goals,
                priority: PromptSectionPriority::High,
                content: format!(
                    "[Agent Goals]\n- short_term_goal: {}\n- long_term_goal: {}\n- anti_stagnation: 避免在缺乏新证据时重复同一动作；连续重复动作前应先补充信息或解释触发条件。\n- exploration_bias: 当局部状态长期不变时，优先探索新地点、新对象或新线索。",
                    input.short_term_goal, input.long_term_goal,
                ),
            },
            true,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::Tools,
                priority: PromptSectionPriority::High,
                content: "[Tool Protocol]
- 如果需要更多信息，可输出模块调用 JSON：{\"type\":\"module_call\",\"module\":\"<module_name>\",\"args\":{...}}
- 在支持 OpenAI tool_calls 的模型上，优先调用已注册工具（function/tool call），不要编造工具名
- 可用模块由 `agent.modules.list` 返回；禁止虚构模块
- 当连续动作触发反重复门控时，优先输出 plan/module_call，不要直接复读同一决策。
- 若确定需要连续执行某动作，可输出 execute_until（支持 `until.event` 单事件或 `until.event_any_of` 多事件；阈值事件需附 `until.value_lte`）。
- 在获得足够信息后，必须输出最终决策 JSON，不要输出多余文本。".to_string(),
            },
            true,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::Context,
                priority: PromptSectionPriority::High,
                content: format!(
                    "[Context]\n- agent_id: {}\n- observation(json): {}",
                    input.agent_id, input.observation_json,
                ),
            },
            true,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::History,
                priority: PromptSectionPriority::Medium,
                content: format!("[Module History]\n{}", input.module_history_json),
            },
            false,
        ));

        if let Some(memory_digest) = input.memory_digest {
            if !memory_digest.trim().is_empty() {
                sections.push(SectionState::new(
                    PromptSection {
                        kind: PromptSectionKind::Memory,
                        priority: PromptSectionPriority::Low,
                        content: format!("[Memory Digest]\n{}", memory_digest),
                    },
                    false,
                ));
            }
        }

        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::StepMeta,
                priority: PromptSectionPriority::Low,
                content: format!(
                    "[Step]\n- step_index: {}\n- max_steps: {}\n- module_calls_used: {}\n- module_calls_max: {}",
                    input.step_context.step_index,
                    input.step_context.max_steps,
                    input.step_context.module_calls_used,
                    input.step_context.module_calls_max,
                ),
            },
            false,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::OutputSchema,
                priority: PromptSectionPriority::High,
                content: "[Decision JSON Schema]
{\"decision\":\"wait\"}
{\"decision\":\"wait_ticks\",\"ticks\":<u64>}
{\"decision\":\"move_agent\",\"to\":\"<location_id>\"}
{\"decision\":\"harvest_radiation\",\"max_amount\":<i64>}
{\"decision\":\"execute_until\",\"action\":{<decision_json>},\"until\":{\"event\":\"<event_name>\"},\"max_ticks\":<u64>}
{\"decision\":\"execute_until\",\"action\":{<decision_json>},\"until\":{\"event_any_of\":[\"action_rejected\",\"new_visible_agent\"]},\"max_ticks\":<u64>}
{\"decision\":\"execute_until\",\"action\":{<decision_json>},\"until\":{\"event\":\"harvest_available_below\",\"value_lte\":<i64>},\"max_ticks\":<u64>}
- event_name 可选: action_rejected, new_visible_agent, new_visible_location, arrive_target, insufficient_electricity, thermal_overload, harvest_yield_below, harvest_available_below
- 当 event_name 为 harvest_yield_below / harvest_available_below 时，必须提供 until.value_lte（>=0）

若你需要查询信息，请输出模块调用 JSON：
{\"type\":\"module_call\",\"module\":\"<module_name>\",\"args\":{...}}".to_string(),
            },
            true,
        ));

        let budget_tokens = input.prompt_budget.effective_input_budget_tokens();
        Self::apply_budget(&mut sections, budget_tokens);

        let section_trace = sections
            .iter()
            .map(|state| PromptSectionTrace {
                kind: state.section.kind,
                priority: state.section.priority,
                included: state.included,
                estimated_tokens: state.estimated_tokens,
                emitted_tokens: state.emitted_tokens(),
            })
            .collect::<Vec<_>>();

        let included_sections = sections
            .iter()
            .filter(|state| state.included)
            .map(|state| state.section.clone())
            .collect::<Vec<_>>();

        let system_prompt = included_sections
            .iter()
            .filter(|section| {
                matches!(
                    section.kind,
                    PromptSectionKind::Policy | PromptSectionKind::Goals | PromptSectionKind::Tools
                )
            })
            .map(|section| section.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let user_prompt = included_sections
            .iter()
            .filter(|section| {
                matches!(
                    section.kind,
                    PromptSectionKind::Context
                        | PromptSectionKind::History
                        | PromptSectionKind::Memory
                        | PromptSectionKind::StepMeta
                        | PromptSectionKind::OutputSchema
                )
            })
            .map(|section| section.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let estimated_input_tokens = estimate_tokens(system_prompt.as_str())
            .saturating_add(estimate_tokens(user_prompt.as_str()));

        PromptAssemblyOutput {
            system_prompt,
            user_prompt,
            sections: included_sections,
            section_trace,
            effective_input_budget_tokens: budget_tokens,
            estimated_input_tokens,
        }
    }

    fn apply_budget(sections: &mut [SectionState], budget_tokens: usize) {
        Self::truncate_soft_section(
            sections,
            PromptSectionKind::History,
            HISTORY_SOFT_CAP_TOKENS,
        );
        Self::truncate_soft_section(sections, PromptSectionKind::Memory, MEMORY_SOFT_CAP_TOKENS);

        let removable_order = [
            PromptSectionKind::StepMeta,
            PromptSectionKind::Memory,
            PromptSectionKind::History,
        ];

        for kind in removable_order {
            if Self::included_tokens(sections) <= budget_tokens {
                break;
            }
            if let Some(state) = sections
                .iter_mut()
                .find(|state| state.section.kind == kind && state.included && !state.required)
            {
                state.included = false;
            }
        }

        if Self::included_tokens(sections) > budget_tokens {
            Self::truncate_required_context(sections, budget_tokens);
        }
    }

    fn truncate_soft_section(
        sections: &mut [SectionState],
        kind: PromptSectionKind,
        token_cap: usize,
    ) {
        if let Some(state) = sections
            .iter_mut()
            .find(|state| state.section.kind == kind && state.included)
        {
            state.section.content =
                truncate_to_token_cap(state.section.content.as_str(), token_cap);
        }
    }

    fn truncate_required_context(sections: &mut [SectionState], budget_tokens: usize) {
        let current_total = Self::included_tokens(sections);
        if current_total <= budget_tokens {
            return;
        }

        let non_context_tokens = sections
            .iter()
            .filter(|state| state.included && state.section.kind != PromptSectionKind::Context)
            .map(SectionState::emitted_tokens)
            .sum::<usize>();
        let context_cap = budget_tokens
            .saturating_sub(non_context_tokens)
            .max(CONTEXT_MIN_TOKENS);

        if let Some(context) = sections
            .iter_mut()
            .find(|state| state.section.kind == PromptSectionKind::Context && state.included)
        {
            context.section.content =
                truncate_to_token_cap(context.section.content.as_str(), context_cap);
        }
    }

    fn included_tokens(sections: &[SectionState]) -> usize {
        sections.iter().map(SectionState::emitted_tokens).sum()
    }
}

fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    (chars.saturating_add(3) / 4).max(1)
}

fn truncate_to_token_cap(text: &str, token_cap: usize) -> String {
    if token_cap == 0 {
        return "...(truncated)".to_string();
    }
    if estimate_tokens(text) <= token_cap {
        return text.to_string();
    }

    let max_chars = token_cap.saturating_mul(4).saturating_sub(16);
    let mut result = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max_chars {
            break;
        }
        result.push(ch);
    }

    if !result.ends_with("\n") {
        result.push('\n');
    }
    result.push_str("...(truncated)");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input<'a>() -> PromptAssemblyInput<'a> {
        PromptAssemblyInput {
            agent_id: "agent-1",
            base_system_prompt: "base prompt",
            short_term_goal: "short goal",
            long_term_goal: "long goal",
            observation_json: "{\"time\":1}",
            module_history_json: "[]",
            memory_digest: Some("obs@T1 ..."),
            step_context: PromptStepContext {
                step_index: 0,
                max_steps: 4,
                module_calls_used: 0,
                module_calls_max: 3,
            },
            prompt_budget: PromptBudget::default(),
        }
    }

    #[test]
    fn prompt_budget_effective_input_budget_respects_reservations() {
        let budget = PromptBudget {
            context_window_tokens: 4_096,
            reserved_output_tokens: 512,
            safety_margin_tokens: 256,
        };
        assert_eq!(budget.effective_input_budget_tokens(), 3_328);
    }

    #[test]
    fn prompt_assembly_splits_system_and_user_sections() {
        let output = PromptAssembler::assemble(sample_input());
        assert!(output.system_prompt.contains("base prompt"));
        assert!(output.system_prompt.contains("short goal"));
        assert!(output.system_prompt.contains("module_call"));

        assert!(output.user_prompt.contains("observation(json)"));
        assert!(output.user_prompt.contains("[Memory Digest]"));
        assert!(output.user_prompt.contains("Decision JSON Schema"));
    }

    #[test]
    fn prompt_assembly_records_section_trace() {
        let output = PromptAssembler::assemble(sample_input());
        assert!(!output.sections.is_empty());
        assert!(!output.section_trace.is_empty());
        assert!(output
            .section_trace
            .iter()
            .any(|trace| trace.kind == PromptSectionKind::Policy && trace.included));
        assert!(output.section_trace.iter().all(|trace| {
            if trace.included {
                trace.emitted_tokens > 0
            } else {
                trace.emitted_tokens == 0
            }
        }));
    }

    #[test]
    fn prompt_assembly_omits_empty_memory_digest_block() {
        let mut input = sample_input();
        input.memory_digest = Some("   ");

        let output = PromptAssembler::assemble(input);
        assert!(!output.user_prompt.contains("[Memory Digest]"));
    }

    #[test]
    fn prompt_budget_removes_low_priority_sections_first() {
        let history = "h".repeat(4_000);
        let memory = "m".repeat(2_000);
        let input = PromptAssemblyInput {
            agent_id: "agent-1",
            base_system_prompt: "base prompt",
            short_term_goal: "short goal",
            long_term_goal: "long goal",
            observation_json: "{\"time\":1}",
            module_history_json: history.as_str(),
            memory_digest: Some(memory.as_str()),
            step_context: PromptStepContext {
                step_index: 0,
                max_steps: 4,
                module_calls_used: 0,
                module_calls_max: 3,
            },
            prompt_budget: PromptBudget {
                context_window_tokens: 512,
                reserved_output_tokens: 320,
                safety_margin_tokens: 120,
            },
        };

        let output = PromptAssembler::assemble(input);

        let step_meta = output
            .section_trace
            .iter()
            .find(|trace| trace.kind == PromptSectionKind::StepMeta)
            .expect("step meta trace");
        assert!(!step_meta.included);

        let schema = output
            .section_trace
            .iter()
            .find(|trace| trace.kind == PromptSectionKind::OutputSchema)
            .expect("schema trace");
        assert!(schema.included);
        assert!(output.system_prompt.contains("[Tool Protocol]"));
    }

    #[test]
    fn prompt_budget_truncates_history_before_drop() {
        let history = "x".repeat(10_000);
        let input = PromptAssemblyInput {
            agent_id: "agent-1",
            base_system_prompt: "base prompt",
            short_term_goal: "short goal",
            long_term_goal: "long goal",
            observation_json: "{\"time\":1}",
            module_history_json: history.as_str(),
            memory_digest: Some("obs@T1 ..."),
            step_context: PromptStepContext {
                step_index: 0,
                max_steps: 4,
                module_calls_used: 0,
                module_calls_max: 3,
            },
            prompt_budget: PromptBudget {
                context_window_tokens: 2_048,
                reserved_output_tokens: 256,
                safety_margin_tokens: 128,
            },
        };

        let output = PromptAssembler::assemble(input);
        let history = output
            .section_trace
            .iter()
            .find(|trace| trace.kind == PromptSectionKind::History)
            .expect("history trace");
        assert!(history.included);
        assert!(history.emitted_tokens < history.estimated_tokens);
    }
}
