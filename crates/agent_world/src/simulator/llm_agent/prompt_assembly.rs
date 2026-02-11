use serde::Serialize;

const DEFAULT_CONTEXT_WINDOW_TOKENS: usize = 8_192;
const DEFAULT_RESERVED_OUTPUT_TOKENS: usize = 1_024;
const DEFAULT_SAFETY_MARGIN_TOKENS: usize = 512;
const MIN_EFFECTIVE_INPUT_BUDGET_TOKENS: usize = 256;
const HISTORY_SOFT_CAP_TOKENS: usize = 256;
const MEMORY_SOFT_CAP_TOKENS: usize = 192;
const FINALIZE_HISTORY_SOFT_CAP_TOKENS: usize = 192;
const FINALIZE_MEMORY_SOFT_CAP_TOKENS: usize = 128;
const CONTEXT_MIN_TOKENS: usize = 64;
const PEAK_MIN_TARGET_TOKENS: usize = 768;
const PEAK_SOFT_RESERVE_TOKENS: usize = 384;
const PEAK_HARD_RESERVE_TOKENS: usize = 256;
const FINALIZE_PEAK_SOFT_RESERVE_TOKENS: usize = 448;
const FINALIZE_PEAK_HARD_RESERVE_TOKENS: usize = 320;
const PEAK_HISTORY_SOFT_CAP_TOKENS: usize = 192;
const PEAK_MEMORY_SOFT_CAP_TOKENS: usize = 128;
const PEAK_HISTORY_HARD_CAP_TOKENS: usize = 128;
const PEAK_MEMORY_HARD_CAP_TOKENS: usize = 96;

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
    pub harvest_max_amount_cap: i64,
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
                    "[Agent Goals]\n- short_term_goal: {}\n- long_term_goal: {}\n- anti_stagnation: 缺少新证据时避免重复同一动作。\n- exploration_bias: 局部状态不变时优先探索新线索。",
                    input.short_term_goal, input.long_term_goal,
                ),
            },
            true,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::Tools,
                priority: PromptSectionPriority::High,
                content: r#"[Tool Protocol]
- 如果需要更多信息，可输出模块调用 JSON：{"type":"module_call","module":"<module_name>","args":{...}}
- 仅允许模块名：agent.modules.list / environment.current_observation / memory.short_term.recent / memory.long_term.search
- 每轮只允许输出一个 JSON 对象（非数组）；禁止 `---` 分隔多段 JSON，禁止代码块包裹 JSON
- 若本轮输出 module_call，则只能输出 1 个 module_call；不要在同一回复混合 module_call 与 decision*
- 当连续动作触发反重复门控时，优先输出 plan/module_call，不要直接复读同一决策
- 若确定需要连续执行某动作，可输出 execute_until（支持 `until.event` 单事件或 `until.event_any_of` 多事件；阈值事件需附 `until.value_lte`）
- 在获得足够信息后，必须输出最终决策 JSON，不要输出多余文本。"#.to_string(),
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

        let turns_remaining = input
            .step_context
            .max_steps
            .saturating_sub(input.step_context.step_index.saturating_add(1));
        let module_calls_remaining = input
            .step_context
            .module_calls_max
            .saturating_sub(input.step_context.module_calls_used);
        let must_finalize_hint = if turns_remaining <= 1 || module_calls_remaining <= 1 {
            "yes"
        } else {
            "no"
        };

        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::StepMeta,
                priority: PromptSectionPriority::Low,
                content: format!(
                    "[Step]\n- step_index: {}\n- max_steps: {}\n- module_calls_used: {}\n- module_calls_max: {}\n- module_calls_remaining: {}\n- turns_remaining: {}\n- must_finalize_hint: {}",
                    input.step_context.step_index,
                    input.step_context.max_steps,
                    input.step_context.module_calls_used,
                    input.step_context.module_calls_max,
                    module_calls_remaining,
                    turns_remaining,
                    must_finalize_hint,
                ),
            },
            false,
        ));
        sections.push(SectionState::new(
            PromptSection {
                kind: PromptSectionKind::OutputSchema,
                priority: PromptSectionPriority::High,
                content: format!(
                    r#"[Decision JSON Schema]
{{"decision":"wait"}}
{{"decision":"wait_ticks","ticks":<u64>}}
{{"decision":"move_agent","to":"<location_id>"}}
{{"decision":"harvest_radiation","max_amount":<i64 1..={}>}}
{{"decision":"execute_until","action":{{<decision_json>}},"until":{{"event":"<event_name>"}},"max_ticks":<u64>}}
- 推荐 move 模板: {{"decision":"execute_until","action":{{"decision":"move_agent","to":"<location_id>"}},"until":{{"event_any_of":["arrive_target","action_rejected","new_visible_agent","new_visible_location"]}},"max_ticks":<u64 1..=8>}}
- 推荐 harvest 模板: {{"decision":"execute_until","action":{{"decision":"harvest_radiation","max_amount":<i64 1..={}>}},"until":{{"event_any_of":["action_rejected","insufficient_electricity","thermal_overload","new_visible_agent","new_visible_location"]}},"max_ticks":<u64 1..=8>}}
- event_name 可选: action_rejected / new_visible_agent / new_visible_location / arrive_target / insufficient_electricity / thermal_overload / harvest_yield_below / harvest_available_below
- 当 event_name 为 harvest_yield_below / harvest_available_below 时，必须提供 until.value_lte（>=0）
- harvest_radiation.max_amount 必须是正整数，且不超过 {}
- 若输出 decision_draft，则 decision_draft.decision 必须是完整 decision 对象（不能是字符串）
- execute_until 仅允许作为最终 decision 输出，不要放在 decision_draft 中

[Output Hard Rules]
- 每轮只输出一个 JSON 对象（非数组），不要输出多个 JSON 块，不要使用 `---` 分隔
- 当 Step 中 `module_calls_remaining <= 1` 或 `turns_remaining <= 1` 时，必须直接输出最终 decision（可 execute_until）

若你需要查询信息，请输出模块调用 JSON：
{{"type":"module_call","module":"<module_name>","args":{{...}}}}"#,
                    input.harvest_max_amount_cap,
                    input.harvest_max_amount_cap,
                    input.harvest_max_amount_cap,
                ),
            },
            true,
        ));

        let budget_tokens = input.prompt_budget.effective_input_budget_tokens();
        let (history_soft_cap, memory_soft_cap) = Self::soft_section_caps(input.step_context);
        Self::apply_budget(
            &mut sections,
            budget_tokens,
            history_soft_cap,
            memory_soft_cap,
        );
        let (peak_soft_tokens, peak_hard_tokens) =
            Self::peak_targets_tokens(input.step_context, budget_tokens);
        Self::apply_peak_budget(&mut sections, peak_soft_tokens, peak_hard_tokens);

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

    fn soft_section_caps(step_context: PromptStepContext) -> (usize, usize) {
        let turns_remaining = step_context
            .max_steps
            .saturating_sub(step_context.step_index.saturating_add(1));
        let module_calls_remaining = step_context
            .module_calls_max
            .saturating_sub(step_context.module_calls_used);
        if turns_remaining <= 1 || module_calls_remaining <= 1 {
            (
                FINALIZE_HISTORY_SOFT_CAP_TOKENS,
                FINALIZE_MEMORY_SOFT_CAP_TOKENS,
            )
        } else {
            (HISTORY_SOFT_CAP_TOKENS, MEMORY_SOFT_CAP_TOKENS)
        }
    }

    fn apply_budget(
        sections: &mut [SectionState],
        budget_tokens: usize,
        history_soft_cap_tokens: usize,
        memory_soft_cap_tokens: usize,
    ) {
        if Self::included_tokens(sections) > budget_tokens {
            Self::truncate_soft_section(
                sections,
                PromptSectionKind::History,
                history_soft_cap_tokens,
            );
        }
        if Self::included_tokens(sections) > budget_tokens {
            Self::truncate_soft_section(
                sections,
                PromptSectionKind::Memory,
                memory_soft_cap_tokens,
            );
        }

        let removable_order = [
            PromptSectionKind::StepMeta,
            PromptSectionKind::Memory,
            PromptSectionKind::History,
        ];

        for kind in removable_order {
            if Self::included_tokens(sections) <= budget_tokens {
                break;
            }
            Self::drop_optional_section(sections, kind);
        }

        if Self::included_tokens(sections) > budget_tokens {
            Self::truncate_required_context(sections, budget_tokens);
        }
    }

    fn peak_targets_tokens(
        step_context: PromptStepContext,
        budget_tokens: usize,
    ) -> (usize, usize) {
        let turns_remaining = step_context
            .max_steps
            .saturating_sub(step_context.step_index.saturating_add(1));
        let module_calls_remaining = step_context
            .module_calls_max
            .saturating_sub(step_context.module_calls_used);

        let (soft_reserve, hard_reserve) = if turns_remaining <= 1 || module_calls_remaining <= 1 {
            (
                FINALIZE_PEAK_SOFT_RESERVE_TOKENS,
                FINALIZE_PEAK_HARD_RESERVE_TOKENS,
            )
        } else {
            (PEAK_SOFT_RESERVE_TOKENS, PEAK_HARD_RESERVE_TOKENS)
        };

        let min_target = PEAK_MIN_TARGET_TOKENS.min(budget_tokens.max(1));
        let hard_target = budget_tokens.saturating_sub(hard_reserve).max(min_target);
        let soft_target = hard_target
            .saturating_sub(soft_reserve.saturating_sub(hard_reserve))
            .max(min_target.saturating_sub(96));

        (soft_target.min(hard_target), hard_target)
    }

    fn apply_peak_budget(
        sections: &mut [SectionState],
        peak_soft_tokens: usize,
        peak_hard_tokens: usize,
    ) {
        if Self::included_tokens(sections) <= peak_soft_tokens {
            return;
        }

        Self::truncate_soft_section(
            sections,
            PromptSectionKind::History,
            PEAK_HISTORY_SOFT_CAP_TOKENS,
        );
        if Self::included_tokens(sections) > peak_soft_tokens {
            Self::truncate_soft_section(
                sections,
                PromptSectionKind::Memory,
                PEAK_MEMORY_SOFT_CAP_TOKENS,
            );
        }
        if Self::included_tokens(sections) > peak_soft_tokens {
            Self::drop_optional_section(sections, PromptSectionKind::StepMeta);
        }

        if Self::included_tokens(sections) > peak_hard_tokens {
            Self::truncate_soft_section(
                sections,
                PromptSectionKind::History,
                PEAK_HISTORY_HARD_CAP_TOKENS,
            );
        }
        if Self::included_tokens(sections) > peak_hard_tokens {
            Self::truncate_soft_section(
                sections,
                PromptSectionKind::Memory,
                PEAK_MEMORY_HARD_CAP_TOKENS,
            );
        }
        if Self::included_tokens(sections) > peak_hard_tokens {
            Self::drop_optional_section(sections, PromptSectionKind::Memory);
        }
        if Self::included_tokens(sections) > peak_hard_tokens {
            Self::drop_optional_section(sections, PromptSectionKind::History);
        }
        if Self::included_tokens(sections) > peak_hard_tokens {
            Self::truncate_required_context(sections, peak_hard_tokens);
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

    fn drop_optional_section(sections: &mut [SectionState], kind: PromptSectionKind) {
        if let Some(state) = sections
            .iter_mut()
            .find(|state| state.section.kind == kind && state.included && !state.required)
        {
            state.included = false;
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
            harvest_max_amount_cap: 100,
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
            harvest_max_amount_cap: 100,
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
            harvest_max_amount_cap: 100,
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

    #[test]
    fn prompt_budget_keeps_soft_sections_when_budget_is_sufficient() {
        let history = "x".repeat(3_000);
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
            harvest_max_amount_cap: 100,
            prompt_budget: PromptBudget::default(),
        };

        let output = PromptAssembler::assemble(input);
        let history = output
            .section_trace
            .iter()
            .find(|trace| trace.kind == PromptSectionKind::History)
            .expect("history trace");
        assert!(history.included);
        assert_eq!(history.emitted_tokens, history.estimated_tokens);
    }

    #[test]
    fn prompt_budget_peak_targets_are_below_effective_budget() {
        let budget = PromptBudget {
            context_window_tokens: 4_608,
            reserved_output_tokens: 896,
            safety_margin_tokens: 512,
        };
        let step = PromptStepContext {
            step_index: 0,
            max_steps: 4,
            module_calls_used: 0,
            module_calls_max: 3,
        };

        let effective = budget.effective_input_budget_tokens();
        let (soft, hard) = PromptAssembler::peak_targets_tokens(step, effective);

        assert!(soft <= hard);
        assert!(hard < effective);
    }

    #[test]
    fn prompt_budget_peak_targets_are_stricter_near_finalize_phase() {
        let budget = PromptBudget {
            context_window_tokens: 4_608,
            reserved_output_tokens: 896,
            safety_margin_tokens: 512,
        };

        let early = PromptStepContext {
            step_index: 0,
            max_steps: 4,
            module_calls_used: 0,
            module_calls_max: 3,
        };
        let near_finalize = PromptStepContext {
            step_index: 3,
            max_steps: 4,
            module_calls_used: 2,
            module_calls_max: 3,
        };

        let effective = budget.effective_input_budget_tokens();
        let (_, early_hard) = PromptAssembler::peak_targets_tokens(early, effective);
        let (_, finalize_hard) = PromptAssembler::peak_targets_tokens(near_finalize, effective);

        assert!(finalize_hard < early_hard);
    }

    #[test]
    fn prompt_budget_peak_budget_enforces_hard_target_on_large_inputs() {
        let history = "h".repeat(14_000);
        let memory = "m".repeat(6_000);
        let step_context = PromptStepContext {
            step_index: 0,
            max_steps: 4,
            module_calls_used: 0,
            module_calls_max: 3,
        };
        let budget = PromptBudget {
            context_window_tokens: 4_608,
            reserved_output_tokens: 896,
            safety_margin_tokens: 512,
        };
        let input = PromptAssemblyInput {
            agent_id: "agent-1",
            base_system_prompt: "base prompt",
            short_term_goal: "short goal",
            long_term_goal: "long goal",
            observation_json: "{\"time\":1}",
            module_history_json: history.as_str(),
            memory_digest: Some(memory.as_str()),
            step_context,
            harvest_max_amount_cap: 100,
            prompt_budget: budget,
        };

        let output = PromptAssembler::assemble(input);
        let (_, hard_target) = PromptAssembler::peak_targets_tokens(
            step_context,
            budget.effective_input_budget_tokens(),
        );

        assert!(output.estimated_input_tokens <= hard_target);
    }

    #[test]
    fn prompt_assembly_includes_single_json_constraints() {
        let output = PromptAssembler::assemble(sample_input());
        assert!(output
            .system_prompt
            .contains("每轮只允许输出一个 JSON 对象"));
        assert!(output.user_prompt.contains("[Output Hard Rules]"));
        assert!(output
            .user_prompt
            .contains("decision_draft.decision 必须是完整 decision 对象"));
    }

    #[test]
    fn prompt_assembly_step_meta_contains_remaining_budget_hints() {
        let mut input = sample_input();
        input.step_context.step_index = 2;
        input.step_context.max_steps = 4;
        input.step_context.module_calls_used = 2;
        input.step_context.module_calls_max = 3;

        let output = PromptAssembler::assemble(input);
        assert!(output.user_prompt.contains("module_calls_remaining: 1"));
        assert!(output.user_prompt.contains("turns_remaining: 1"));
        assert!(output.user_prompt.contains("must_finalize_hint: yes"));
    }

    #[test]
    fn prompt_assembly_includes_harvest_max_amount_cap() {
        let mut input = sample_input();
        input.harvest_max_amount_cap = 42;

        let output = PromptAssembler::assemble(input);
        assert!(output.user_prompt.contains("<i64 1..=42>"));
        assert!(output.user_prompt.contains("不超过 42"));
        assert!(output.user_prompt.contains("推荐 harvest 模板"));
        assert!(output.user_prompt.contains("推荐 move 模板"));
    }
}
