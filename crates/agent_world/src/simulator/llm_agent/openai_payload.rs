use super::*;

pub(super) fn responses_tools() -> Vec<Tool> {
    vec![
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
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "decision": {
                        "type": "string",
                        "enum": [
                            "wait",
                            "wait_ticks",
                            "move_agent",
                            "harvest_radiation",
                            "transfer_resource",
                            "refine_compound",
                            "build_factory",
                            "schedule_recipe",
                            "execute_until"
                        ]
                    },
                    "ticks": { "type": "integer", "minimum": 1 },
                    "to": { "type": "string" },
                    "max_amount": { "type": "integer", "minimum": 1 },
                    "from_owner": { "type": "string" },
                    "to_owner": { "type": "string" },
                    "kind": { "type": "string" },
                    "amount": { "type": "integer", "minimum": 1 },
                    "owner": { "type": "string" },
                    "compound_mass_g": { "type": "integer", "minimum": 1 },
                    "location_id": { "type": "string" },
                    "factory_id": { "type": "string" },
                    "factory_kind": { "type": "string" },
                    "recipe_id": { "type": "string" },
                    "batches": { "type": "integer", "minimum": 1 },
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
            })),
            strict: None,
        }),
    ]
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

pub(super) fn function_call_to_module_call_json(name: &str, arguments: &str) -> String {
    if name == OPENAI_TOOL_AGENT_SUBMIT_DECISION {
        return decode_tool_arguments(arguments).to_string();
    }
    let module = module_name_from_tool_name(name);
    let args = decode_tool_arguments(arguments);
    serde_json::json!({
        "type": "module_call",
        "module": module,
        "args": args,
    })
    .to_string()
}

pub(super) fn output_item_to_module_call_json(item: &OutputItem) -> Option<String> {
    match item {
        OutputItem::FunctionCall(function_call) => Some(function_call_to_module_call_json(
            function_call.name.as_str(),
            function_call.arguments.as_str(),
        )),
        _ => None,
    }
}

pub(super) fn extract_module_call_from_raw_response(value: &serde_json::Value) -> Option<String> {
    let output_items = value.get("output")?.as_array()?;
    let mut tool_outputs = Vec::new();
    for item in output_items {
        if item.get("type").and_then(|kind| kind.as_str()) != Some("function_call") {
            continue;
        }
        let name = item.get("name").and_then(|name| name.as_str())?;
        let arguments = item
            .get("arguments")
            .and_then(|arguments| arguments.as_str())
            .unwrap_or("{}");
        tool_outputs.push(function_call_to_module_call_json(name, arguments));
    }
    if tool_outputs.is_empty() {
        None
    } else {
        Some(tool_outputs.join("\n"))
    }
}

pub(super) fn completion_result_from_raw_response_json(
    raw: &str,
) -> Result<LlmCompletionResult, LlmClientError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| LlmClientError::DecodeResponse {
            message: format!("compat parse failed: {err}"),
        })?;

    let output = extract_module_call_from_raw_response(&value).unwrap_or_default();

    if output.trim().is_empty() {
        return Err(LlmClientError::EmptyChoice);
    }

    let usage = value.get("usage");
    Ok(LlmCompletionResult {
        output,
        model: value
            .get("model")
            .and_then(|model| model.as_str())
            .map(|model| model.to_string()),
        prompt_tokens: usage
            .and_then(|usage| usage.get("input_tokens"))
            .and_then(|value| value.as_u64()),
        completion_tokens: usage
            .and_then(|usage| usage.get("output_tokens"))
            .and_then(|value| value.as_u64()),
        total_tokens: usage
            .and_then(|usage| usage.get("total_tokens"))
            .and_then(|value| value.as_u64()),
    })
}

pub(super) fn completion_result_from_sdk_response(
    response: Response,
) -> Result<LlmCompletionResult, LlmClientError> {
    let outputs = response
        .output
        .iter()
        .filter_map(output_item_to_module_call_json)
        .collect::<Vec<_>>();
    let output = outputs.join("\n");

    if output.trim().is_empty() {
        return Err(LlmClientError::EmptyChoice);
    }

    let usage = response.usage.as_ref();
    Ok(LlmCompletionResult {
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
        .tools(responses_tools())
        .tool_choice(ToolChoiceParam::Mode(ToolChoiceOptions::Required))
        .build()
        .map_err(|err| LlmClientError::DecodeResponse {
            message: err.to_string(),
        })
}
