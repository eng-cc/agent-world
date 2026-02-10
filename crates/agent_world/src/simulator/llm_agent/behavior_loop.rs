use super::*;
use std::time::Instant;

impl<C: LlmCompletionClient> AgentBehavior for LlmAgentBehavior<C> {
    fn agent_id(&self) -> &str {
        self.agent_id.as_str()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        self.memory
            .record_observation(observation.time, Self::observe_memory_summary(observation));

        if let Some(active_execute_until) = self.active_execute_until.as_mut() {
            match active_execute_until.evaluate_next_step(observation) {
                Ok(()) => {
                    let decision = AgentDecision::Act(active_execute_until.action().clone());
                    let output_summary = format!(
                        "execute_until continue: events={} remaining_ticks={}",
                        active_execute_until.until_events_summary(),
                        active_execute_until.remaining_ticks(),
                    );
                    self.replan_guard_state.record_decision(&decision);
                    self.memory
                        .record_decision(observation.time, decision.clone());
                    self.pending_trace = Some(AgentDecisionTrace {
                        agent_id: self.agent_id.clone(),
                        time: observation.time,
                        decision: decision.clone(),
                        llm_input: None,
                        llm_output: Some(output_summary.clone()),
                        llm_error: None,
                        parse_error: None,
                        llm_diagnostics: Some(LlmDecisionDiagnostics {
                            model: Some(self.config.model.clone()),
                            latency_ms: Some(0),
                            prompt_tokens: None,
                            completion_tokens: None,
                            total_tokens: None,
                            retry_count: 0,
                        }),
                        llm_effect_intents: vec![],
                        llm_effect_receipts: vec![],
                        llm_step_trace: vec![LlmStepTrace {
                            step_index: 0,
                            step_type: "execute_until_continue".to_string(),
                            input_summary: "skip_llm_with_active_execute_until".to_string(),
                            output_summary,
                            status: "ok".to_string(),
                        }],
                        llm_prompt_section_trace: vec![],
                    });
                    return decision;
                }
                Err(stop_reason) => {
                    self.memory.record_note(
                        observation.time,
                        format!("execute_until stop: {stop_reason}"),
                    );
                    self.active_execute_until = None;
                }
            }
        }

        let mut decision = AgentDecision::Wait;
        let mut parse_error: Option<String> = None;
        let mut llm_error: Option<String> = None;
        let mut module_history = Vec::new();
        let mut llm_effect_intents = Vec::new();
        let mut llm_effect_receipts = Vec::new();
        let mut trace_inputs = Vec::new();
        let mut trace_outputs = Vec::new();
        let mut llm_step_trace = Vec::new();
        let mut llm_prompt_section_trace = Vec::new();

        let mut model = Some(self.config.model.clone());
        let mut latency_total_ms = 0_u64;
        let mut prompt_tokens_total = 0_u64;
        let mut completion_tokens_total = 0_u64;
        let mut total_tokens_total = 0_u64;
        let mut has_prompt_tokens = false;
        let mut has_completion_tokens = false;
        let mut has_total_tokens = false;

        let mut resolved = false;
        let max_turns = self.config.max_decision_steps.max(1);
        let should_force_replan = self
            .replan_guard_state
            .should_force_replan(self.config.force_replan_after_same_action);
        let replan_guard_summary = if should_force_replan {
            self.replan_guard_state
                .guard_summary(self.config.force_replan_after_same_action)
                .unwrap_or_else(|| {
                    format!(
                        "consecutive_same_action>=threshold({})",
                        self.config.force_replan_after_same_action
                    )
                })
        } else {
            String::new()
        };

        let mut phase = if should_force_replan {
            DecisionPhase::ModuleLoop
        } else {
            DecisionPhase::Plan
        };
        let mut pending_draft: Option<AgentDecision> = None;
        let mut repair_rounds_used = 0_u32;
        let mut repair_context: Option<String> = None;
        let mut deferred_parse_error: Option<String> = None;
        let mut resolved_via_execute_until = false;

        for turn in 0..max_turns {
            let active_phase = phase;
            let is_repair_turn = repair_context.is_some();
            let step_type = if is_repair_turn {
                "repair"
            } else {
                active_phase.step_type()
            };

            let prompt_output =
                self.assemble_prompt_output(observation, &module_history, turn, max_turns);
            llm_prompt_section_trace.extend(prompt_output.section_trace.iter().map(|section| {
                LlmPromptSectionTrace {
                    step_index: turn,
                    kind: prompt_section_kind_name(section.kind).to_string(),
                    priority: prompt_section_priority_name(section.priority).to_string(),
                    included: section.included,
                    estimated_tokens: section.estimated_tokens,
                    emitted_tokens: section.emitted_tokens,
                }
            }));

            let mut user_prompt = prompt_output.user_prompt.clone();
            let module_calls_remaining = self
                .config
                .max_module_calls
                .saturating_sub(module_history.len());
            let turns_remaining = max_turns.saturating_sub(turn.saturating_add(1));
            let must_finalize_due_budget = turns_remaining <= 1 || module_calls_remaining <= 1;

            user_prompt.push_str(
                "

[Step Orchestration]
",
            );
            user_prompt.push_str(active_phase.prompt_instruction());
            user_prompt.push_str(
                "

[Output Constraints]
- 本轮只允许输出一个 JSON 对象（非数组）；禁止 `---` 分隔与代码块包裹 JSON。
",
            );
            user_prompt.push_str(
                format!(
                    "- module_calls_remaining={} turns_remaining={}
",
                    module_calls_remaining, turns_remaining
                )
                .as_str(),
            );
            user_prompt.push_str(
                format!(
                    "- harvest_radiation.max_amount 必须在 [1, {}]；超限将被运行时裁剪。
",
                    self.config.harvest_max_amount_cap
                )
                .as_str(),
            );
            user_prompt.push_str(
                format!(
                    "- 若连续两轮输出同一可执行动作，优先使用 execute_until（参考 schema 推荐模板）减少重复决策；auto_reenter_ticks={}。
",
                    self.config.execute_until_auto_reenter_ticks
                )
                .as_str(),
            );
            if must_finalize_due_budget {
                user_prompt.push_str("- 预算接近上限：本轮必须直接输出最终 decision（可 execute_until），禁止 plan/module_call/decision_draft。
");
            } else {
                user_prompt.push_str("- 若输出 module_call，本轮仅允许一个 module_call；不要在同一回复混合 module_call 与 decision*。
");
            }

            if should_force_replan {
                user_prompt.push_str(
                    "

[Anti-Repetition Guard]
",
                );
                user_prompt.push_str(
                    "- 检测到连续重复动作，请优先输出 plan/module_call 进行再规划。
",
                );
                user_prompt.push_str("- 若确实需要重复执行同一动作，请使用 execute_until，并提供 until.event（阈值事件需附 until.value_lte）与 max_ticks。
");
                user_prompt.push_str("- guard_state: ");
                user_prompt.push_str(replan_guard_summary.as_str());
            }
            if let Some(draft) = pending_draft.as_ref() {
                let draft_json = serialize_decision_for_prompt(draft);
                user_prompt.push_str(
                    "
- 已有 decision_draft，可直接输出最终 decision：",
                );
                user_prompt.push_str(draft_json.as_str());
                user_prompt.push_str(
                    "
- 已存在 decision_draft，本轮禁止再次输出 decision_draft，必须输出最终 decision。
",
                );
            }
            if let Some(repair_reason) = repair_context.as_ref() {
                user_prompt.push_str(
                    "

[Repair]
",
                );
                user_prompt.push_str("上一轮输出解析失败，请修复为合法 JSON：");
                user_prompt.push_str(repair_reason.as_str());
                user_prompt.push_str(
                    "
仅返回一个合法 JSON 对象，不要追加其他 JSON 块。
",
                );
            }

            let request = LlmCompletionRequest {
                model: self.config.model.clone(),
                system_prompt: prompt_output.system_prompt.clone(),
                user_prompt,
            };
            let input_summary = format!(
                "phase={step_type}; module_calls={}/{}; repair_rounds={}/{}; force_replan={}; prompt_profile={}",
                module_history.len(),
                self.config.max_module_calls,
                repair_rounds_used,
                self.config.max_repair_rounds,
                should_force_replan,
                self.config.prompt_profile.as_str(),
            );
            trace_inputs.push(Self::trace_input(
                request.system_prompt.as_str(),
                request.user_prompt.as_str(),
            ));

            let request_started_at = Instant::now();
            match self.client.complete(&request) {
                Ok(completion) => {
                    let latency_ms = request_started_at.elapsed().as_millis() as u64;
                    latency_total_ms = latency_total_ms.saturating_add(latency_ms);

                    if let Some(returned_model) = completion.model.clone() {
                        model = Some(returned_model);
                    }
                    if let Some(tokens) = completion.prompt_tokens {
                        has_prompt_tokens = true;
                        prompt_tokens_total = prompt_tokens_total.saturating_add(tokens);
                    }
                    if let Some(tokens) = completion.completion_tokens {
                        has_completion_tokens = true;
                        completion_tokens_total = completion_tokens_total.saturating_add(tokens);
                    }
                    if let Some(tokens) = completion.total_tokens {
                        has_total_tokens = true;
                        total_tokens_total = total_tokens_total.saturating_add(tokens);
                    }

                    trace_outputs.push(completion.output.clone());

                    let mut step_status = "ok".to_string();
                    let mut step_output_summary =
                        summarize_trace_text(completion.output.as_str(), 220);

                    let parsed_turns = parse_llm_turn_responses(
                        completion.output.as_str(),
                        self.agent_id.as_str(),
                    );

                    for parsed_turn in parsed_turns {
                        match parsed_turn {
                            ParsedLlmTurn::Decision(parsed_decision, decision_parse_error) => {
                                if should_force_replan && module_history.is_empty() {
                                    let err = "replan guard requires plan/module_call before terminal decision"
                                        .to_string();
                                    step_status = "error".to_string();
                                    step_output_summary = err.clone();
                                    if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                        repair_rounds_used = repair_rounds_used.saturating_add(1);
                                        repair_context = Some(err);
                                    } else {
                                        parse_error = Some(err);
                                        resolved = true;
                                        step_status = "degraded".to_string();
                                    }
                                } else {
                                    let (guarded_decision, guardrail_note) =
                                        self.apply_decision_guardrails(parsed_decision);
                                    decision = guarded_decision;
                                    parse_error = decision_parse_error;
                                    pending_draft = None;
                                    repair_context = None;
                                    phase = DecisionPhase::Finalize;
                                    resolved = true;
                                    resolved_via_execute_until = false;
                                    if let Some(note) = guardrail_note {
                                        self.memory.record_note(observation.time, note.clone());
                                        step_output_summary = format!(
                                            "{}; {}",
                                            step_output_summary,
                                            summarize_trace_text(note.as_str(), 160)
                                        );
                                    }
                                }
                            }
                            ParsedLlmTurn::ExecuteUntil(directive) => {
                                let mut guarded_directive = directive;
                                let (guarded_action, guardrail_note) =
                                    self.apply_action_guardrails(guarded_directive.action);
                                guarded_directive.action = guarded_action.clone();
                                decision = AgentDecision::Act(guarded_action);
                                self.active_execute_until =
                                    Some(ActiveExecuteUntil::from_directive(
                                        guarded_directive.clone(),
                                        observation,
                                    ));
                                pending_draft = None;
                                repair_context = None;
                                phase = DecisionPhase::Finalize;
                                resolved = true;
                                resolved_via_execute_until = true;
                                step_output_summary = format!(
                                    "execute_until events={} max_ticks={}",
                                    guarded_directive
                                        .until_conditions
                                        .iter()
                                        .map(|condition| condition.summary())
                                        .collect::<Vec<_>>()
                                        .join("|"),
                                    guarded_directive.max_ticks
                                );
                                if let Some(note) = guardrail_note {
                                    self.memory.record_note(observation.time, note.clone());
                                    step_output_summary = format!(
                                        "{}; {}",
                                        step_output_summary,
                                        summarize_trace_text(note.as_str(), 160)
                                    );
                                }
                            }
                            ParsedLlmTurn::Plan(plan) => {
                                repair_context = None;
                                let wants_module_call = plan
                                    .next
                                    .as_deref()
                                    .is_some_and(|next| next.eq_ignore_ascii_case("module_call"));
                                let must_keep_module_loop =
                                    should_force_replan && module_history.is_empty();
                                phase = if must_keep_module_loop
                                    || wants_module_call
                                    || !plan.missing.is_empty()
                                {
                                    DecisionPhase::ModuleLoop
                                } else {
                                    DecisionPhase::DecisionDraft
                                };

                                let missing_summary = if plan.missing.is_empty() {
                                    "-".to_string()
                                } else {
                                    plan.missing.join(",")
                                };
                                let next_summary = plan.next.unwrap_or_else(|| "-".to_string());
                                step_output_summary = format!(
                                    "plan missing={missing_summary}; next={next_summary}; force_module_loop={must_keep_module_loop}"
                                );
                            }
                            ParsedLlmTurn::ModuleCall(module_request) => {
                                repair_context = None;
                                if module_history.len() >= self.config.max_module_calls {
                                    deferred_parse_error = Some(format!(
                                        "module call limit exceeded: max_module_calls={}",
                                        self.config.max_module_calls
                                    ));
                                    step_status = "error".to_string();
                                    step_output_summary =
                                        "module_call skipped: limit exceeded".to_string();
                                    phase = DecisionPhase::DecisionDraft;
                                } else {
                                    let module_name = module_request.module.clone();
                                    let intent_id = self.next_prompt_intent_id();
                                    let intent = LlmEffectIntentTrace {
                                        intent_id: intent_id.clone(),
                                        kind: LLM_PROMPT_MODULE_CALL_KIND.to_string(),
                                        params: serde_json::json!({
                                            "module": module_request.module,
                                            "args": module_request.args,
                                        }),
                                        cap_ref: LLM_PROMPT_MODULE_CALL_CAP_REF.to_string(),
                                        origin: LLM_PROMPT_MODULE_CALL_ORIGIN.to_string(),
                                    };
                                    let module_result =
                                        self.run_prompt_module(&module_request, observation);
                                    let status_label = if module_result
                                        .get("ok")
                                        .and_then(|value| value.as_bool())
                                        .unwrap_or(false)
                                    {
                                        "ok"
                                    } else {
                                        "error"
                                    };
                                    let receipt = LlmEffectReceiptTrace {
                                        intent_id: intent.intent_id.clone(),
                                        status: status_label.to_string(),
                                        payload: module_result.clone(),
                                        cost_cents: None,
                                    };

                                    llm_effect_intents.push(intent);
                                    llm_effect_receipts.push(receipt);
                                    trace_inputs.push(format!(
                                        "[module_result:{}]
{}",
                                        module_name,
                                        serde_json::to_string(&module_result)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    ));
                                    module_history.push(ModuleCallExchange {
                                        module: module_request.module,
                                        args: module_request.args,
                                        result: module_result,
                                    });

                                    phase = if module_history.len() >= self.config.max_module_calls
                                    {
                                        DecisionPhase::DecisionDraft
                                    } else {
                                        DecisionPhase::ModuleLoop
                                    };
                                    step_output_summary =
                                        format!("module_call {module_name} status={status_label}");
                                }
                            }
                            ParsedLlmTurn::DecisionDraft(draft) => {
                                if should_force_replan && module_history.is_empty() {
                                    let err =
                                        "replan guard requires module_call before decision_draft"
                                            .to_string();
                                    step_status = "error".to_string();
                                    step_output_summary = err.clone();
                                    if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                        repair_rounds_used = repair_rounds_used.saturating_add(1);
                                        repair_context = Some(err);
                                    } else {
                                        parse_error = Some(err);
                                        resolved = true;
                                        step_status = "degraded".to_string();
                                    }
                                } else {
                                    let confidence = draft.confidence;
                                    let need_verify = draft.need_verify;
                                    let (guarded_decision, guardrail_note) =
                                        self.apply_decision_guardrails(draft.decision);
                                    pending_draft = Some(guarded_decision);
                                    repair_context = None;
                                    phase = if need_verify
                                        && module_history.len() < self.config.max_module_calls
                                    {
                                        DecisionPhase::ModuleLoop
                                    } else {
                                        DecisionPhase::Finalize
                                    };
                                    step_output_summary = format!(
                                        "decision_draft confidence={:?}; need_verify={}",
                                        confidence, need_verify
                                    );
                                    if let Some(note) = guardrail_note {
                                        self.memory.record_note(observation.time, note.clone());
                                        step_output_summary = format!(
                                            "{}; {}",
                                            step_output_summary,
                                            summarize_trace_text(note.as_str(), 160)
                                        );
                                    }
                                }
                            }
                            ParsedLlmTurn::Invalid(err) => {
                                step_status = "error".to_string();
                                step_output_summary = format!(
                                    "parse_error: {}",
                                    summarize_trace_text(err.as_str(), 180)
                                );
                                if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                    repair_rounds_used = repair_rounds_used.saturating_add(1);
                                    repair_context = Some(err);
                                } else {
                                    parse_error = Some(err);
                                    resolved = true;
                                    step_status = "degraded".to_string();
                                }
                            }
                        }

                        if resolved || repair_context.is_some() {
                            break;
                        }
                    }

                    llm_step_trace.push(LlmStepTrace {
                        step_index: turn,
                        step_type: step_type.to_string(),
                        input_summary: input_summary.clone(),
                        output_summary: step_output_summary,
                        status: step_status,
                    });

                    if resolved {
                        break;
                    }
                }
                Err(err) => {
                    llm_error = Some(err.to_string());
                    latency_total_ms = latency_total_ms
                        .saturating_add(request_started_at.elapsed().as_millis() as u64);
                    llm_step_trace.push(LlmStepTrace {
                        step_index: turn,
                        step_type: step_type.to_string(),
                        input_summary,
                        output_summary: summarize_trace_text(err.to_string().as_str(), 220),
                        status: "degraded".to_string(),
                    });
                    resolved = true;
                    break;
                }
            }
        }

        if !resolved {
            if let Some(draft_decision) = pending_draft.take() {
                decision = draft_decision;
                parse_error = Some(format!(
                    "no terminal decision after {} turn(s); fallback to decision_draft",
                    max_turns
                ));
            } else {
                parse_error = deferred_parse_error
                    .take()
                    .or_else(|| Some(format!("no terminal decision after {} turn(s)", max_turns)));
            }
        }

        if self.config.execute_until_auto_reenter_ticks > 0
            && self.active_execute_until.is_none()
            && !resolved_via_execute_until
        {
            if let AgentDecision::Act(action) = &decision {
                if self.replan_guard_state.is_same_action_as_last(action) {
                    let projected_repeat = self
                        .replan_guard_state
                        .projected_consecutive_same_action(action);
                    let will_force_replan_next = self.config.force_replan_after_same_action > 0
                        && projected_repeat >= self.config.force_replan_after_same_action;
                    if !will_force_replan_next {
                        let max_ticks = self.config.execute_until_auto_reenter_ticks as u64;
                        let active_execute_until = ActiveExecuteUntil::from_auto_reentry(
                            action.clone(),
                            observation,
                            max_ticks,
                        );
                        let until_summary = active_execute_until.until_events_summary();
                        self.active_execute_until = Some(active_execute_until);
                        let note = format!(
                            "execute_until auto reentry armed: max_ticks={} until={}",
                            max_ticks, until_summary
                        );
                        self.memory.record_note(observation.time, note.clone());
                        llm_step_trace.push(LlmStepTrace {
                            step_index: max_turns,
                            step_type: "execute_until_auto_reentry".to_string(),
                            input_summary: "post_decision_same_action_detected".to_string(),
                            output_summary: summarize_trace_text(note.as_str(), 220),
                            status: "ok".to_string(),
                        });
                    }
                }
            }
        }

        self.replan_guard_state.record_decision(&decision);
        self.memory
            .record_decision(observation.time, decision.clone());

        self.pending_trace = Some(AgentDecisionTrace {
            agent_id: self.agent_id.clone(),
            time: observation.time,
            decision: decision.clone(),
            llm_input: Some(trace_inputs.join("\n\n---\n\n")),
            llm_output: (!trace_outputs.is_empty()).then(|| trace_outputs.join("\n\n---\n\n")),
            llm_error,
            parse_error,
            llm_diagnostics: Some(LlmDecisionDiagnostics {
                model,
                latency_ms: Some(latency_total_ms),
                prompt_tokens: has_prompt_tokens.then_some(prompt_tokens_total),
                completion_tokens: has_completion_tokens.then_some(completion_tokens_total),
                total_tokens: has_total_tokens.then_some(total_tokens_total),
                retry_count: repair_rounds_used,
            }),
            llm_effect_intents,
            llm_effect_receipts,
            llm_step_trace,
            llm_prompt_section_trace,
        });

        decision
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        let time = result.event.time;
        self.memory
            .record_action_result(time, result.action.clone(), result.success);
        if !result.success {
            self.memory.long_term.store_with_tags(
                format!(
                    "action_failed: action={:?}; event={:?}",
                    result.action, result.event.kind
                ),
                time,
                vec!["action_result".to_string(), "failed".to_string()],
            );
        }
        if let Some(active_execute_until) = self.active_execute_until.as_mut() {
            active_execute_until.update_from_action_result(result);
        }

        self.memory.consolidate(time, 0.9);
    }

    fn on_event(&mut self, event: &WorldEvent) {
        self.memory
            .record_event(event.time, format!("event: {:?}", event.kind));
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        self.pending_trace.take()
    }
}
