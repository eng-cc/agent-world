use super::*;
use crate::simulator::ModuleInstallTarget;

#[test]
fn decision_tool_schema_includes_module_lifecycle_actions_and_fields() {
    let decision_tool = responses_tools()
        .into_iter()
        .find_map(|tool| match tool {
            Tool::Function(function_tool)
                if function_tool.name == OPENAI_TOOL_AGENT_SUBMIT_DECISION =>
            {
                Some(function_tool)
            }
            _ => None,
        })
        .expect("decision tool exists");

    let parameters = decision_tool.parameters.expect("decision tool parameters");
    let decision_enum = parameters
        .get("properties")
        .and_then(|value| value.get("decision"))
        .and_then(|value| value.get("enum"))
        .and_then(|value| value.as_array())
        .expect("decision enum")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    assert!(decision_enum.contains(&"compile_module_artifact_from_source"));
    assert!(decision_enum.contains(&"deploy_module_artifact"));
    assert!(decision_enum.contains(&"install_module_from_artifact"));
    assert!(decision_enum.contains(&"install_module_to_target_from_artifact"));
    assert!(decision_enum.contains(&"list_module_artifact_for_sale"));
    assert!(decision_enum.contains(&"buy_module_artifact"));
    assert!(decision_enum.contains(&"delist_module_artifact"));
    assert!(decision_enum.contains(&"destroy_module_artifact"));
    assert!(decision_enum.contains(&"place_module_artifact_bid"));
    assert!(decision_enum.contains(&"cancel_module_artifact_bid"));

    let properties = parameters
        .get("properties")
        .and_then(|value| value.as_object())
        .expect("decision properties");
    assert!(properties.contains_key("manifest_path"));
    assert!(properties.contains_key("source_files"));
    assert!(properties.contains_key("wasm_hash"));
    assert!(properties.contains_key("wasm_bytes_hex"));
    assert!(properties.contains_key("module_version"));
    assert!(properties.contains_key("activate"));
    assert!(properties.contains_key("install_target_type"));
    assert!(properties.contains_key("install_target_location_id"));
    assert!(properties.contains_key("price_kind"));
    assert!(properties.contains_key("price_amount"));
    assert!(properties.contains_key("bid_order_id"));
    assert!(properties.contains_key("bidder"));
}

#[test]
fn llm_parse_compile_module_artifact_from_source_action() {
    let turns = completion_turns_from_output(
        r#"{"decision":"compile_module_artifact_from_source","publisher":"self","module_id":"m.llm.compile","manifest_path":"Cargo.toml","source_files":{"Cargo.toml":"cargo-content","src/lib.rs":"lib-content"}}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Decision {
            decision:
                AgentDecision::Act(Action::CompileModuleArtifactFromSource {
                    publisher_agent_id,
                    module_id,
                    manifest_path,
                    source_files,
                }),
            ..
        } => {
            assert_eq!(publisher_agent_id, "agent-1");
            assert_eq!(module_id, "m.llm.compile");
            assert_eq!(manifest_path, "Cargo.toml");
            assert!(source_files.contains_key("Cargo.toml"));
            assert!(source_files.contains_key("src/lib.rs"));
            assert_eq!(
                source_files
                    .get("src/lib.rs")
                    .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                    .as_deref(),
                Some("lib-content")
            );
        }
        other => panic!("unexpected parsed turn: {other:?}"),
    }
}

#[test]
fn llm_parse_deploy_module_artifact_rejects_invalid_hex_bytes() {
    let turns = completion_turns_from_output(
        r#"{"decision":"deploy_module_artifact","publisher":"self","wasm_hash":"abc","wasm_bytes_hex":"not-hex"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Invalid(message) => {
            assert!(
                message.contains("wasm_bytes_hex") && message.contains("valid hex"),
                "unexpected parse error: {message}"
            );
        }
        other => panic!("expected invalid decision, got {other:?}"),
    }
}

#[test]
fn llm_parse_install_module_from_artifact_defaults_version_and_activate() {
    let turns = completion_turns_from_output(
        r#"{"decision":"install_module_from_artifact","installer":"self","module_id":"m.llm.install","wasm_hash":"abcd"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Decision {
            decision:
                AgentDecision::Act(Action::InstallModuleFromArtifact {
                    installer_agent_id,
                    module_id,
                    module_version,
                    wasm_hash,
                    activate,
                }),
            ..
        } => {
            assert_eq!(installer_agent_id, "agent-1");
            assert_eq!(module_id, "m.llm.install");
            assert_eq!(module_version, "0.1.0");
            assert_eq!(wasm_hash, "abcd");
            assert!(*activate);
        }
        other => panic!("unexpected parsed turn: {other:?}"),
    }
}

#[test]
fn llm_parse_install_module_to_target_from_artifact_action() {
    let turns = completion_turns_from_output(
        r#"{"decision":"install_module_to_target_from_artifact","installer":"self","module_id":"m.llm.install.target","module_version":"0.2.0","wasm_hash":"hash-target","activate":false,"install_target_type":"location_infrastructure","install_target_location_id":"loc-hub"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Decision {
            decision:
                AgentDecision::Act(Action::InstallModuleToTargetFromArtifact {
                    installer_agent_id,
                    module_id,
                    module_version,
                    wasm_hash,
                    activate,
                    install_target,
                }),
            ..
        } => {
            assert_eq!(installer_agent_id, "agent-1");
            assert_eq!(module_id, "m.llm.install.target");
            assert_eq!(module_version, "0.2.0");
            assert_eq!(wasm_hash, "hash-target");
            assert!(!*activate);
            assert_eq!(
                install_target,
                &ModuleInstallTarget::LocationInfrastructure {
                    location_id: "loc-hub".to_string(),
                }
            );
        }
        other => panic!("unexpected parsed turn: {other:?}"),
    }
}

#[test]
fn llm_parse_install_module_to_target_from_artifact_rejects_missing_location_id() {
    let turns = completion_turns_from_output(
        r#"{"decision":"install_module_to_target_from_artifact","installer":"self","module_id":"m.llm.install.target","wasm_hash":"hash-target","install_target_type":"location_infrastructure"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Invalid(message) => {
            assert!(
                message.contains("install_target_location_id"),
                "unexpected parse error: {message}"
            );
        }
        other => panic!("expected invalid decision, got {other:?}"),
    }
}

#[test]
fn llm_parse_list_module_artifact_for_sale_action() {
    let turns = completion_turns_from_output(
        r#"{"decision":"list_module_artifact_for_sale","seller":"self","wasm_hash":"hash-1","price_kind":"data","price_amount":3}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Decision {
            decision:
                AgentDecision::Act(Action::ListModuleArtifactForSale {
                    seller_agent_id,
                    wasm_hash,
                    price_kind,
                    price_amount,
                }),
            ..
        } => {
            assert_eq!(seller_agent_id, "agent-1");
            assert_eq!(wasm_hash, "hash-1");
            assert_eq!(*price_kind, ResourceKind::Data);
            assert_eq!(*price_amount, 3);
        }
        other => panic!("unexpected parsed turn: {other:?}"),
    }
}

#[test]
fn llm_parse_cancel_module_artifact_bid_rejects_non_agent_bidder() {
    let turns = completion_turns_from_output(
        r#"{"decision":"cancel_module_artifact_bid","bidder":"location:loc-a","wasm_hash":"hash-1","bid_order_id":7}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    match parsed.first().expect("parsed turn") {
        ParsedLlmTurn::Invalid(message) => {
            assert!(
                message.contains("self or agent:<id>"),
                "unexpected parse error: {message}"
            );
        }
        other => panic!("expected invalid decision, got {other:?}"),
    }
}

#[test]
fn llm_agent_module_lifecycle_status_module_reports_known_records() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let action = Action::DeployModuleArtifact {
        publisher_agent_id: "agent-1".to_string(),
        wasm_hash: "hash-1".to_string(),
        wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d],
        module_id_hint: Some("m.llm.lifecycle".to_string()),
    };
    behavior.on_action_result(&ActionResult {
        action: action.clone(),
        action_id: 1,
        success: true,
        event: WorldEvent {
            id: 1,
            time: 10,
            kind: WorldEventKind::ModuleArtifactDeployed {
                publisher_agent_id: "agent-1".to_string(),
                wasm_hash: "hash-1".to_string(),
                wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d],
                bytes_len: 4,
                module_id_hint: Some("m.llm.lifecycle".to_string()),
            },
        },
    });
    behavior.on_action_result(&ActionResult {
        action: Action::InstallModuleFromArtifact {
            installer_agent_id: "agent-1".to_string(),
            module_id: "m.llm.lifecycle".to_string(),
            module_version: "0.1.0".to_string(),
            wasm_hash: "hash-1".to_string(),
            activate: true,
        },
        action_id: 2,
        success: true,
        event: WorldEvent {
            id: 2,
            time: 11,
            kind: WorldEventKind::ModuleInstalled {
                installer_agent_id: "agent-1".to_string(),
                module_id: "m.llm.lifecycle".to_string(),
                module_version: "0.1.0".to_string(),
                wasm_hash: "hash-1".to_string(),
                active: true,
                install_target: ModuleInstallTarget::SelfAgent,
            },
        },
    });

    let result = behavior.run_prompt_module(
        &LlmModuleCallRequest {
            module: "module.lifecycle.status".to_string(),
            args: serde_json::json!({}),
        },
        &make_observation(),
    );

    assert_eq!(
        result.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    let status = result
        .get("result")
        .expect("module lifecycle status result");
    let artifacts = status
        .get("artifacts")
        .and_then(|value| value.as_array())
        .expect("artifacts array");
    assert_eq!(artifacts.len(), 1);
    assert_eq!(
        artifacts[0]
            .get("wasm_hash")
            .and_then(|value| value.as_str()),
        Some("hash-1")
    );

    let installed = status
        .get("installed_modules")
        .and_then(|value| value.as_array())
        .expect("installed modules array");
    assert_eq!(installed.len(), 1);
    assert_eq!(
        installed[0]
            .get("module_id")
            .and_then(|value| value.as_str()),
        Some("m.llm.lifecycle")
    );
    assert_eq!(
        installed[0]
            .get("install_target")
            .and_then(|value| value.get("type"))
            .and_then(|value| value.as_str()),
        Some("self_agent")
    );
}

#[test]
fn llm_agent_prompt_mentions_module_lifecycle_decisions() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);

    assert!(prompt.contains("compile_module_artifact_from_source"));
    assert!(prompt.contains("deploy_module_artifact"));
    assert!(prompt.contains("install_module_from_artifact"));
    assert!(prompt.contains("install_module_to_target_from_artifact"));
    assert!(prompt.contains("list_module_artifact_for_sale"));
    assert!(prompt.contains("buy_module_artifact"));
    assert!(prompt.contains("place_module_artifact_bid"));
    assert!(prompt.contains("cancel_module_artifact_bid"));
    assert!(prompt.contains("module.lifecycle.status"));
}
