use serde::Serialize;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptAssemblyOutput {
    pub system_prompt: String,
    pub user_prompt: String,
    pub sections: Vec<PromptSection>,
    pub section_trace: Vec<PromptSectionTrace>,
}

pub struct PromptAssembler;

impl PromptAssembler {
    pub fn assemble(input: PromptAssemblyInput<'_>) -> PromptAssemblyOutput {
        let sections = vec![
            PromptSection {
                kind: PromptSectionKind::Policy,
                priority: PromptSectionPriority::High,
                content: format!(
                    "{}\n\n你是一个硅基文明 Agent。请严格输出 JSON，不要输出额外文字。",
                    input.base_system_prompt,
                ),
            },
            PromptSection {
                kind: PromptSectionKind::Goals,
                priority: PromptSectionPriority::High,
                content: format!(
                    "[Agent Goals]\n- short_term_goal: {}\n- long_term_goal: {}",
                    input.short_term_goal, input.long_term_goal,
                ),
            },
            PromptSection {
                kind: PromptSectionKind::Tools,
                priority: PromptSectionPriority::High,
                content: "[Tool Protocol]\n- 如果需要更多信息，可输出模块调用 JSON：{\"type\":\"module_call\",\"module\":\"<module_name>\",\"args\":{...}}\n- 可用模块由 `agent.modules.list` 返回；禁止虚构模块\n- 在获得足够信息后，必须输出最终决策 JSON，不要输出多余文本。".to_string(),
            },
            PromptSection {
                kind: PromptSectionKind::Context,
                priority: PromptSectionPriority::High,
                content: format!(
                    "[Context]\n- agent_id: {}\n- observation(json): {}",
                    input.agent_id, input.observation_json,
                ),
            },
            PromptSection {
                kind: PromptSectionKind::History,
                priority: PromptSectionPriority::Medium,
                content: format!(
                    "[Module History]\n{}",
                    input.module_history_json,
                ),
            },
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
            PromptSection {
                kind: PromptSectionKind::OutputSchema,
                priority: PromptSectionPriority::High,
                content: "[Decision JSON Schema]\n{\"decision\":\"wait\"}\n{\"decision\":\"wait_ticks\",\"ticks\":<u64>}\n{\"decision\":\"move_agent\",\"to\":\"<location_id>\"}\n{\"decision\":\"harvest_radiation\",\"max_amount\":<i64>}\n\n若你需要查询信息，请输出模块调用 JSON：\n{\"type\":\"module_call\",\"module\":\"<module_name>\",\"args\":{...}}".to_string(),
            },
        ];

        let section_trace = sections
            .iter()
            .map(|section| PromptSectionTrace {
                kind: section.kind,
                priority: section.priority,
                included: true,
            })
            .collect::<Vec<_>>();

        let system_prompt = sections
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

        let mut user_parts = vec![
            sections
                .iter()
                .find(|section| section.kind == PromptSectionKind::Context)
                .expect("context section")
                .content
                .clone(),
            sections
                .iter()
                .find(|section| section.kind == PromptSectionKind::History)
                .expect("history section")
                .content
                .clone(),
        ];

        if let Some(memory_digest) = input.memory_digest {
            if !memory_digest.trim().is_empty() {
                user_parts.push(format!("[Memory Digest]\n{}", memory_digest));
            }
        }

        user_parts.push(
            sections
                .iter()
                .find(|section| section.kind == PromptSectionKind::StepMeta)
                .expect("step meta section")
                .content
                .clone(),
        );
        user_parts.push(
            sections
                .iter()
                .find(|section| section.kind == PromptSectionKind::OutputSchema)
                .expect("schema section")
                .content
                .clone(),
        );

        PromptAssemblyOutput {
            system_prompt,
            user_prompt: user_parts.join("\n\n"),
            sections,
            section_trace,
        }
    }
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
        }
    }

    #[test]
    fn prompt_assembly_splits_system_and_user_sections() {
        let output = PromptAssembler::assemble(sample_input());
        assert!(output.system_prompt.contains("base prompt"));
        assert!(output.system_prompt.contains("short goal"));
        assert!(output.system_prompt.contains("module_call"));

        assert!(output.user_prompt.contains("observation(json)"));
        assert!(output.user_prompt.contains("Memory Digest"));
        assert!(output.user_prompt.contains("Decision JSON Schema"));
    }

    #[test]
    fn prompt_assembly_records_section_trace() {
        let output = PromptAssembler::assemble(sample_input());
        assert!(!output.sections.is_empty());
        assert_eq!(output.sections.len(), output.section_trace.len());
        assert!(output.section_trace.iter().all(|trace| trace.included));
    }

    #[test]
    fn prompt_assembly_omits_empty_memory_digest_block() {
        let mut input = sample_input();
        input.memory_digest = Some("   ");

        let output = PromptAssembler::assemble(input);
        assert!(!output.user_prompt.contains("[Memory Digest]"));
    }
}
