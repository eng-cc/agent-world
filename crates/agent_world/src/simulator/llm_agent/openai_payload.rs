use super::*;

const AGENT_SUBMIT_DECISION_SCHEMA_JSON: &str = r#"{
  "type": "object",
  "properties": {
    "decision": {
      "type": "string",
      "enum": [
        "wait",
        "wait_ticks",
        "move_agent",
        "harvest_radiation",
        "buy_power",
        "sell_power",
        "place_power_order",
        "cancel_power_order",
        "transfer_resource",
        "mine_compound",
        "refine_compound",
        "build_factory",
        "schedule_recipe",
        "compile_module_artifact_from_source",
        "deploy_module_artifact",
        "install_module_from_artifact",
        "list_module_artifact_for_sale",
        "buy_module_artifact",
        "delist_module_artifact",
        "destroy_module_artifact",
        "place_module_artifact_bid",
        "cancel_module_artifact_bid",
        "publish_social_fact",
        "challenge_social_fact",
        "adjudicate_social_fact",
        "revoke_social_fact",
        "declare_social_edge",
        "execute_until"
      ]
    },
    "ticks": { "type": "integer", "minimum": 1 },
    "to": { "type": "string" },
    "from": { "type": "string" },
    "max_amount": { "type": "integer", "minimum": 1 },
    "from_owner": { "type": "string" },
    "to_owner": { "type": "string" },
    "kind": { "type": "string" },
    "amount": { "type": "integer", "minimum": 1 },
    "buyer": { "type": "string" },
    "seller": { "type": "string" },
    "price_per_pu": { "type": "integer", "minimum": 0 },
    "side": { "type": "string", "enum": ["buy", "sell"] },
    "limit_price_per_pu": { "type": "integer", "minimum": 0 },
    "order_id": { "type": "integer", "minimum": 1 },
    "owner": { "type": "string" },
    "compound_mass_g": { "type": "integer", "minimum": 1 },
    "location_id": { "type": "string" },
    "factory_id": { "type": "string" },
    "factory_kind": { "type": "string" },
    "recipe_id": { "type": "string" },
    "batches": { "type": "integer", "minimum": 1 },
    "publisher": { "type": "string" },
    "installer": { "type": "string" },
    "bidder": { "type": "string" },
    "module_id": { "type": "string" },
    "module_version": { "type": "string" },
    "manifest_path": { "type": "string" },
    "source_files": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "wasm_hash": { "type": "string" },
    "wasm_bytes_hex": { "type": "string" },
    "price_kind": { "type": "string" },
    "price_amount": { "type": "integer", "minimum": 1 },
    "bid_order_id": { "type": "integer", "minimum": 1 },
    "activate": { "type": "boolean" },
    "actor": { "type": "string" },
    "declarer": { "type": "string" },
    "subject": { "type": "string" },
    "object": { "type": "string" },
    "schema_id": { "type": "string" },
    "claim": { "type": "string" },
    "confidence_ppm": { "type": "integer", "minimum": 1, "maximum": 1000000 },
    "evidence_event_ids": {
      "type": "array",
      "items": { "type": "integer", "minimum": 1 }
    },
    "ttl_ticks": { "type": "integer", "minimum": 1 },
    "stake": {
      "type": "object",
      "properties": {
        "kind": { "type": "string" },
        "amount": { "type": "integer", "minimum": 1 }
      },
      "additionalProperties": false
    },
    "challenger": { "type": "string" },
    "fact_id": { "type": "integer", "minimum": 1 },
    "reason": { "type": "string" },
    "adjudicator": { "type": "string" },
    "adjudication": { "type": "string", "enum": ["confirm", "retract"] },
    "notes": { "type": "string" },
    "relation_kind": { "type": "string" },
    "weight_bps": { "type": "integer", "minimum": -10000, "maximum": 10000 },
    "backing_fact_ids": {
      "type": "array",
      "items": { "type": "integer", "minimum": 1 }
    },
    "action": {
      "type": "object",
      "additionalProperties": true
    },
    "until": {
      "type": "object",
      "properties": {
        "event": { "type": "string" },
        "event_any_of": {
          "type": "array",
          "items": { "type": "string" }
        },
        "value_lte": { "type": "integer", "minimum": 0 }
      },
      "additionalProperties": false
    },
    "max_ticks": { "type": "integer", "minimum": 1 },
    "message_to_user": { "type": "string" }
  },
  "required": ["decision"],
  "additionalProperties": false
}"#;

fn decision_tool_parameters() -> serde_json::Value {
    serde_json::from_str(AGENT_SUBMIT_DECISION_SCHEMA_JSON)
        .expect("agent_submit_decision schema JSON should be valid")
}

#[cfg(test)]
pub(super) fn responses_tools() -> Vec<Tool> {
    responses_tools_with_debug_mode(false)
}

pub(super) fn responses_tools_with_debug_mode(debug_mode: bool) -> Vec<Tool> {
    let mut tools = vec![
        Tool::Function(FunctionTool {
            name: OPENAI_TOOL_AGENT_MODULES_LIST.to_string(),
            description: Some("列出 Agent 可调用的模块能力与参数。".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            })),
            strict: None,
        }),
        Tool::Function(FunctionTool {
            name: OPENAI_TOOL_ENVIRONMENT_CURRENT_OBSERVATION.to_string(),
            description: Some("读取当前 tick 的环境观测。".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            })),
            strict: None,
        }),
        Tool::Function(FunctionTool {
            name: OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT.to_string(),
            description: Some("读取最近短期记忆。".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 32
                    }
                },
                "additionalProperties": false,
            })),
            strict: None,
        }),
        Tool::Function(FunctionTool {
            name: OPENAI_TOOL_MEMORY_LONG_TERM_SEARCH.to_string(),
            description: Some("按关键词检索长期记忆（query 为空时按重要度返回）。".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 32
                    }
                },
                "additionalProperties": false,
            })),
            strict: None,
        }),
        Tool::Function(FunctionTool {
            name: OPENAI_TOOL_AGENT_SUBMIT_DECISION.to_string(),
            description: Some(
                "提交最终决策；所有世界动作必须通过该 tool call，而不是输出文本 JSON。".to_string(),
            ),
            parameters: Some(decision_tool_parameters()),
            strict: None,
        }),
    ];

    if debug_mode {
        tools.push(Tool::Function(FunctionTool {
            name: OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE.to_string(),
            description: Some(
                "仅 debug 模式可用：向 owner 追加任意资源数量用于调试闭环。".to_string(),
            ),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string" },
                    "kind": { "type": "string" },
                    "amount": { "type": "integer", "minimum": 1 }
                },
                "required": ["kind", "amount"],
                "additionalProperties": false
            })),
            strict: None,
        }));
    }

    tools
}

pub(super) fn module_name_from_tool_name(name: &str) -> &str {
    match name {
        OPENAI_TOOL_AGENT_MODULES_LIST => "agent.modules.list",
        OPENAI_TOOL_ENVIRONMENT_CURRENT_OBSERVATION => "environment.current_observation",
        OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT => "memory.short_term.recent",
        OPENAI_TOOL_MEMORY_LONG_TERM_SEARCH => "memory.long_term.search",
        other => other,
    }
}

pub(super) fn decode_tool_arguments(arguments: &str) -> serde_json::Value {
    let trimmed = arguments.trim();
    if trimmed.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| {
            serde_json::json!({
                "_raw": trimmed,
            })
        })
    }
}

pub(super) fn function_call_to_completion_turn(name: &str, arguments: &str) -> LlmCompletionTurn {
    if name == OPENAI_TOOL_AGENT_SUBMIT_DECISION {
        return LlmCompletionTurn::Decision {
            payload: decode_tool_arguments(arguments),
        };
    }
    if name == OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE {
        let mut payload = decode_tool_arguments(arguments);
        if let Some(map) = payload.as_object_mut() {
            map.insert(
                "decision".to_string(),
                serde_json::Value::String("debug_grant_resource".to_string()),
            );
        } else {
            payload = serde_json::json!({
                "decision": "debug_grant_resource",
                "raw_args": payload,
            });
        }
        return LlmCompletionTurn::Decision { payload };
    }
    LlmCompletionTurn::ModuleCall {
        module: module_name_from_tool_name(name).to_string(),
        args: decode_tool_arguments(arguments),
    }
}

pub(super) fn output_item_to_completion_turn(item: &OutputItem) -> Option<LlmCompletionTurn> {
    match item {
        OutputItem::FunctionCall(function_call) => Some(function_call_to_completion_turn(
            function_call.name.as_str(),
            function_call.arguments.as_str(),
        )),
        _ => None,
    }
}

pub(super) fn completion_turn_to_trace_json(turn: &LlmCompletionTurn) -> String {
    match turn {
        LlmCompletionTurn::Decision { payload } => payload.to_string(),
        LlmCompletionTurn::ModuleCall { module, args } => serde_json::json!({
            "type": "module_call",
            "module": module,
            "args": args,
        })
        .to_string(),
    }
}

pub(super) fn completion_result_from_sdk_response(
    response: Response,
) -> Result<LlmCompletionResult, LlmClientError> {
    let turns = response
        .output
        .iter()
        .filter_map(output_item_to_completion_turn)
        .collect::<Vec<_>>();
    let output = turns
        .iter()
        .map(completion_turn_to_trace_json)
        .collect::<Vec<_>>()
        .join("\n");

    if output.trim().is_empty() {
        return Err(LlmClientError::EmptyChoice);
    }

    let usage = response.usage.as_ref();
    Ok(LlmCompletionResult {
        turns,
        output,
        model: Some(response.model),
        prompt_tokens: usage.map(|usage| usage.input_tokens as u64),
        completion_tokens: usage.map(|usage| usage.output_tokens as u64),
        total_tokens: usage.map(|usage| usage.total_tokens as u64),
    })
}

pub(super) fn normalize_openai_api_base_url(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/');
    if let Some(stripped) = normalized.strip_suffix("/chat/completions") {
        stripped.to_string()
    } else if let Some(stripped) = normalized.strip_suffix("/responses") {
        stripped.to_string()
    } else {
        normalized.to_string()
    }
}

pub(super) fn build_responses_request_payload(
    request: &LlmCompletionRequest,
) -> Result<CreateResponse, LlmClientError> {
    CreateResponseArgs::default()
        .model(request.model.clone())
        .instructions(request.system_prompt.clone())
        .input(InputParam::Text(request.user_prompt.clone()))
        .tools(responses_tools_with_debug_mode(request.debug_mode))
        .tool_choice(ToolChoiceParam::Mode(ToolChoiceOptions::Required))
        .parallel_tool_calls(false)
        .build()
        .map_err(|err| LlmClientError::DecodeResponse {
            message: err.to_string(),
        })
}
