use super::*;
use std::time::Instant;

impl<C: LlmCompletionClient> AgentBehavior for LlmAgentBehavior<C> {
    fn agent_id(&self) -> &str {
        self.agent_id.as_str()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        self.memory
            .record_observation(observation.time, Self::observe_memory_summary(observation));
        let trace_chat_start = self
            .conversation_trace_cursor
            .min(self.conversation_history.len());

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
                    let trace_chat_messages =
                        self.conversation_history[trace_chat_start..].to_vec();
                    self.conversation_trace_cursor = self.conversation_history.len();
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
                        llm_chat_messages: trace_chat_messages,
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
        let force_replan_threshold = if self.config.execute_until_auto_reenter_ticks > 0
            && self.config.force_replan_after_same_action >= 4
        {
            self.config.force_replan_after_same_action.saturating_add(2)
        } else {
            self.config.force_replan_after_same_action
        };
        let should_force_replan = self
            .replan_guard_state
            .should_force_replan(force_replan_threshold);
        let replan_guard_summary = if should_force_replan {
            self.replan_guard_state
                .guard_summary(force_replan_threshold)
                .unwrap_or_else(|| {
                    format!(
                        "consecutive_same_action>=threshold({})",
                        force_replan_threshold
                    )
                })
        } else {
            String::new()
        };

        let mut repair_rounds_used = 0_u32;
        let mut repair_context: Option<String> = None;
        let mut deferred_parse_error: Option<String> = None;
        let mut resolved_via_execute_until = false;

        for turn in 0..max_turns {
            let is_repair_turn = repair_context.is_some();
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

            let module_calls_remaining = self
                .config
                .max_module_calls
                .saturating_sub(module_history.len());
            let turns_remaining = max_turns.saturating_sub(turn.saturating_add(1));

            let mut user_prompt = prompt_output.user_prompt.clone();
            user_prompt.push_str(
                "\n\n[Dialogue Constraints]\n- 本轮只允许输出一个 JSON 对象（非数组）；禁止 `---` 分隔与代码块包裹 JSON。\n",
            );
            user_prompt.push_str(
                format!(
                    "- dialogue_turn={}; turns_remaining={}; module_calls_remaining={}.\n",
                    turn, turns_remaining, module_calls_remaining,
                )
                .as_str(),
            );
            user_prompt.push_str(
                format!(
                    "- harvest_radiation.max_amount 必须在 [1, {}]；超限将被运行时裁剪。\n",
                    self.config.harvest_max_amount_cap
                )
                .as_str(),
            );
            user_prompt.push_str(
                "- move_agent.to 不能是当前所在位置（observation 中 distance_cm=0 的 location）。\n",
            );
            user_prompt.push_str(
                "- 若需要信息查询，只能输出一个 module_call JSON；拿到工具结果后下一轮再输出最终 decision。\n",
            );
            user_prompt.push_str(
                "- 若要向玩家解释当前决策，请在 JSON 内使用 message_to_user 字段；不要在 JSON 外输出文本。\n",
            );
            user_prompt.push_str(
                "- 若上下文已足够，请直接输出最终 decision；message_to_user 仅保留关键信息，避免冗长。\n",
            );
            user_prompt.push_str(
                format!(
                    "- 若连续两轮输出同一可执行动作，优先使用 execute_until（auto_reenter_ticks={}）。\n",
                    self.config.execute_until_auto_reenter_ticks
                )
                .as_str(),
            );

            if should_force_replan {
                user_prompt.push_str("\n[Anti-Repetition Guard]\n");
                user_prompt.push_str(
                    "- 检测到连续重复动作，请避免原样重复动作；应先查询新证据或切换动作。\n",
                );
                user_prompt.push_str(
                    "- 若确需连续执行同一动作，请使用 execute_until，并设置 until.event（阈值事件需附 until.value_lte）与 max_ticks。\n",
                );
                user_prompt.push_str("- guard_state: ");
                user_prompt.push_str(replan_guard_summary.as_str());
                user_prompt.push('\n');
            }

            if module_calls_remaining <= 1 || turns_remaining <= 1 {
                user_prompt.push_str(
                    "- 预算接近上限：本轮必须输出最终 decision（可 execute_until），不要继续 module_call。\n",
                );
            }

            if let Some(repair_reason) = repair_context.as_ref() {
                user_prompt.push_str("\n[Repair]\n上一轮输出解析失败，请修复为合法 JSON：");
                user_prompt.push_str(repair_reason.as_str());
                user_prompt.push_str("\n仅返回一个合法 JSON 对象，不要追加其他 JSON 块。\n");
            }

            let request = LlmCompletionRequest {
                model: self.config.model.clone(),
                system_prompt: prompt_output.system_prompt.clone(),
                user_prompt,
            };
            let input_summary = format!(
                "turn={turn}; module_calls={}/{}; repair_rounds={}/{}; force_replan={}; prompt_profile={}",
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

                    let mut turn_status = "ok".to_string();
                    let mut turn_output_summary =
                        summarize_trace_text(completion.output.as_str(), 220);
                    let mut retry_next_turn = false;
                    let mut tool_called_this_turn = false;

                    let parsed_turns = parse_llm_turn_responses(
                        completion.output.as_str(),
                        self.agent_id.as_str(),
                    );

                    for parsed_turn in parsed_turns {
                        match parsed_turn {
                            ParsedLlmTurn::Decision {
                                decision: parsed_decision,
                                parse_error: decision_parse_error,
                                message_to_user,
                            } => {
                                if let Some(message_to_user) = message_to_user.as_deref() {
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Agent,
                                        message_to_user,
                                    );
                                }
                                let repeats_last_action = matches!(
                                    &parsed_decision,
                                    AgentDecision::Act(action)
                                        if self.replan_guard_state.is_same_action_as_last(action)
                                );
                                if should_force_replan
                                    && module_history.is_empty()
                                    && repeats_last_action
                                {
                                    let err = "replan guard requires module_call or execute_until before repeating same terminal action"
                                        .to_string();
                                    turn_status = "error".to_string();
                                    turn_output_summary = err.clone();
                                    if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                        repair_rounds_used = repair_rounds_used.saturating_add(1);
                                        repair_context = Some(err);
                                        retry_next_turn = true;
                                    } else {
                                        parse_error = Some(err);
                                        turn_status = "degraded".to_string();
                                        resolved = true;
                                    }
                                } else {
                                    let (guarded_decision, guardrail_note) =
                                        self.apply_decision_guardrails(parsed_decision);
                                    decision = guarded_decision;
                                    parse_error = decision_parse_error;
                                    repair_context = None;
                                    resolved = true;
                                    resolved_via_execute_until = false;

                                    if let Some(note) = guardrail_note {
                                        self.memory.record_note(observation.time, note.clone());
                                        let _ = self.append_conversation_message(
                                            observation.time,
                                            LlmChatRole::System,
                                            note.as_str(),
                                        );
                                        turn_output_summary = format!(
                                            "{}; {}",
                                            turn_output_summary,
                                            summarize_trace_text(note.as_str(), 160)
                                        );
                                    }
                                }
                            }
                            ParsedLlmTurn::ExecuteUntil {
                                directive,
                                message_to_user,
                            } => {
                                if let Some(message_to_user) = message_to_user.as_deref() {
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Agent,
                                        message_to_user,
                                    );
                                }
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
                                repair_context = None;
                                resolved = true;
                                resolved_via_execute_until = true;

                                turn_output_summary = format!(
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
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::System,
                                        note.as_str(),
                                    );
                                    turn_output_summary = format!(
                                        "{}; {}",
                                        turn_output_summary,
                                        summarize_trace_text(note.as_str(), 160)
                                    );
                                }
                            }
                            ParsedLlmTurn::ModuleCall {
                                request: module_request,
                                message_to_user,
                            } => {
                                if let Some(message_to_user) = message_to_user.as_deref() {
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Agent,
                                        message_to_user,
                                    );
                                }
                                if module_history.len() >= self.config.max_module_calls {
                                    deferred_parse_error = Some(format!(
                                        "module call limit exceeded: max_module_calls={}",
                                        self.config.max_module_calls
                                    ));
                                    turn_status = "error".to_string();
                                    turn_output_summary =
                                        "module_call skipped: limit exceeded".to_string();
                                } else {
                                    let module_name = module_request.module.clone();
                                    let module_args = module_request.args.clone();
                                    let intent_id = self.next_prompt_intent_id();
                                    let intent = LlmEffectIntentTrace {
                                        intent_id: intent_id.clone(),
                                        kind: LLM_PROMPT_MODULE_CALL_KIND.to_string(),
                                        params: serde_json::json!({
                                            "module": module_name.clone(),
                                            "args": module_args,
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
                                        "[module_result:{}]\n{}",
                                        module_name,
                                        serde_json::to_string(&module_result)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    ));
                                    module_history.push(ModuleCallExchange {
                                        module: module_name.clone(),
                                        args: module_request.args.clone(),
                                        result: module_result.clone(),
                                    });

                                    let tool_feedback = serde_json::json!({
                                        "type": "module_call_result",
                                        "module": module_name.clone(),
                                        "status": status_label,
                                        "args": module_request.args,
                                        "result": module_result,
                                    })
                                    .to_string();
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Tool,
                                        tool_feedback.as_str(),
                                    );

                                    turn_output_summary =
                                        format!("module_call {module_name} status={status_label}");
                                    tool_called_this_turn = true;
                                }
                            }
                            ParsedLlmTurn::DecisionDraft {
                                draft,
                                message_to_user,
                            } => {
                                if let Some(message_to_user) = message_to_user.as_deref() {
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Agent,
                                        message_to_user,
                                    );
                                }
                                let repeats_last_action = matches!(
                                    &draft.decision,
                                    AgentDecision::Act(action)
                                        if self.replan_guard_state.is_same_action_as_last(action)
                                );
                                if should_force_replan
                                    && module_history.is_empty()
                                    && repeats_last_action
                                {
                                    let err =
                                        "replan guard requires module_call or execute_until before repeating same decision_draft"
                                            .to_string();
                                    turn_status = "error".to_string();
                                    turn_output_summary = err.clone();
                                    if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                        repair_rounds_used = repair_rounds_used.saturating_add(1);
                                        repair_context = Some(err);
                                        retry_next_turn = true;
                                    } else {
                                        parse_error = Some(err);
                                        turn_status = "degraded".to_string();
                                        resolved = true;
                                    }
                                } else {
                                    let (guarded_decision, guardrail_note) =
                                        self.apply_decision_guardrails(draft.decision);
                                    decision = guarded_decision;
                                    repair_context = None;
                                    resolved = true;
                                    resolved_via_execute_until = false;
                                    turn_output_summary = format!(
                                        "decision_draft accepted: confidence={:?}; need_verify={}",
                                        draft.confidence, draft.need_verify
                                    );

                                    if let Some(note) = guardrail_note {
                                        self.memory.record_note(observation.time, note.clone());
                                        let _ = self.append_conversation_message(
                                            observation.time,
                                            LlmChatRole::System,
                                            note.as_str(),
                                        );
                                        turn_output_summary = format!(
                                            "{}; {}",
                                            turn_output_summary,
                                            summarize_trace_text(note.as_str(), 160)
                                        );
                                    }
                                }
                            }
                            ParsedLlmTurn::Plan {
                                payload: plan,
                                message_to_user,
                            } => {
                                if let Some(message_to_user) = message_to_user.as_deref() {
                                    let _ = self.append_conversation_message(
                                        observation.time,
                                        LlmChatRole::Agent,
                                        message_to_user,
                                    );
                                }
                                let missing = if plan.missing.is_empty() {
                                    "-".to_string()
                                } else {
                                    plan.missing.join(",")
                                };
                                let next = plan.next.unwrap_or_else(|| "-".to_string());
                                let err = format!(
                                    "plan output is deprecated in dialogue mode (missing={missing}, next={next}); return module_call or final decision"
                                );
                                turn_status = "error".to_string();
                                turn_output_summary = summarize_trace_text(err.as_str(), 180);
                                if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                    repair_rounds_used = repair_rounds_used.saturating_add(1);
                                    repair_context = Some(err);
                                    retry_next_turn = true;
                                } else {
                                    parse_error = Some(err);
                                    turn_status = "degraded".to_string();
                                    resolved = true;
                                }
                            }
                            ParsedLlmTurn::Invalid(err) => {
                                turn_status = "error".to_string();
                                turn_output_summary = format!(
                                    "parse_error: {}",
                                    summarize_trace_text(err.as_str(), 180)
                                );
                                if repair_rounds_used < self.config.max_repair_rounds as u32 {
                                    repair_rounds_used = repair_rounds_used.saturating_add(1);
                                    repair_context = Some(err);
                                    retry_next_turn = true;
                                } else {
                                    parse_error = Some(err);
                                    turn_status = "degraded".to_string();
                                    resolved = true;
                                }
                            }
                        }

                        if resolved || retry_next_turn {
                            break;
                        }
                    }

                    llm_step_trace.push(LlmStepTrace {
                        step_index: turn,
                        step_type: if is_repair_turn {
                            "repair".to_string()
                        } else {
                            "dialogue_turn".to_string()
                        },
                        input_summary: input_summary.clone(),
                        output_summary: turn_output_summary,
                        status: turn_status,
                    });

                    if resolved {
                        break;
                    }
                    if retry_next_turn || tool_called_this_turn {
                        continue;
                    }

                    let err = "no actionable JSON object found in assistant output".to_string();
                    if repair_rounds_used < self.config.max_repair_rounds as u32 {
                        repair_rounds_used = repair_rounds_used.saturating_add(1);
                        repair_context = Some(err);
                        continue;
                    }
                    parse_error = Some(err);
                    resolved = true;
                }
                Err(err) => {
                    llm_error = Some(err.to_string());
                    latency_total_ms = latency_total_ms
                        .saturating_add(request_started_at.elapsed().as_millis() as u64);
                    llm_step_trace.push(LlmStepTrace {
                        step_index: turn,
                        step_type: if is_repair_turn {
                            "repair".to_string()
                        } else {
                            "dialogue_turn".to_string()
                        },
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
            parse_error = deferred_parse_error.take().or_else(|| {
                Some(format!(
                    "no terminal decision after {} dialogue turns",
                    max_turns
                ))
            });
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
                    let will_force_replan_next =
                        force_replan_threshold > 0 && projected_repeat >= force_replan_threshold;
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
                        let _ = self.append_conversation_message(
                            observation.time,
                            LlmChatRole::System,
                            note.as_str(),
                        );
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
        let trace_chat_messages = self.conversation_history[trace_chat_start..].to_vec();
        self.conversation_trace_cursor = self.conversation_history.len();

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
            llm_chat_messages: trace_chat_messages,
        });

        decision
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        let time = result.event.time;
        self.last_action_summary = Some(Self::summarize_action_result_for_prompt(result));
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

        let feedback = if result.success {
            format!(
                "action_feedback: success=true action={:?} event={:?}",
                result.action, result.event.kind
            )
        } else {
            format!(
                "action_feedback: success=false action={:?} reject_reason={:?}",
                result.action,
                result.reject_reason(),
            )
        };
        let _ = self.append_conversation_message(time, LlmChatRole::System, feedback.as_str());

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
