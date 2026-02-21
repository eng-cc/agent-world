use super::*;
use agent_world::simulator::{Action, WorldEvent, WorldEventKind};

#[test]
fn parse_options_defaults() {
    let options = parse_options([].into_iter()).expect("defaults");
    assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
    assert_eq!(options.ticks, 20);
    assert!(options.runtime_gameplay_bridge);
    assert_eq!(options.load_state_dir, None);
    assert_eq!(options.save_state_dir, None);
    assert!(!options.print_llm_io);
    assert_eq!(options.llm_io_max_chars, None);
    assert_eq!(options.llm_system_prompt, None);
    assert_eq!(options.llm_short_term_goal, None);
    assert_eq!(options.llm_long_term_goal, None);
    assert_eq!(options.prompt_switch_tick, None);
    assert_eq!(options.switch_llm_system_prompt, None);
    assert_eq!(options.switch_llm_short_term_goal, None);
    assert_eq!(options.switch_llm_long_term_goal, None);
    assert_eq!(options.prompt_switches_json, None);
    assert!(options.prompt_switches.is_empty());
}

#[test]
fn parse_options_accepts_alias_scenario() {
    let options = parse_options(["llm"].into_iter()).expect("scenario alias");
    assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
}

#[test]
fn parse_options_accepts_ticks() {
    let options = parse_options(["--ticks", "12"].into_iter()).expect("ticks");
    assert_eq!(options.ticks, 12);
}

#[test]
fn parse_options_accepts_report_json_path() {
    let options =
        parse_options(["--report-json", ".tmp/report.json"].into_iter()).expect("report json path");
    assert_eq!(options.report_json.as_deref(), Some(".tmp/report.json"));
}

#[test]
fn parse_options_accepts_state_dir_paths() {
    let options = parse_options(
        [
            "--load-state-dir",
            ".tmp/state/in",
            "--save-state-dir",
            ".tmp/state/out",
        ]
        .into_iter(),
    )
    .expect("state dirs");
    assert_eq!(options.load_state_dir.as_deref(), Some(".tmp/state/in"));
    assert_eq!(options.save_state_dir.as_deref(), Some(".tmp/state/out"));
}

#[test]
fn parse_options_rejects_missing_load_state_dir_path() {
    let err =
        parse_options(["--load-state-dir"].into_iter()).expect_err("missing load state dir path");
    assert!(err.contains("directory path"));
}

#[test]
fn parse_options_disables_runtime_gameplay_bridge() {
    let options =
        parse_options(["--no-runtime-gameplay-bridge"].into_iter()).expect("disable bridge");
    assert!(!options.runtime_gameplay_bridge);
}

#[test]
fn parse_options_enables_print_llm_io() {
    let options = parse_options(["--print-llm-io"].into_iter()).expect("llm io option");
    assert!(options.print_llm_io);
}

#[test]
fn parse_options_accepts_llm_io_max_chars() {
    let options =
        parse_options(["--llm-io-max-chars", "256"].into_iter()).expect("llm io max chars option");
    assert_eq!(options.llm_io_max_chars, Some(256));
}

#[test]
fn parse_options_accepts_initial_prompt_overrides() {
    let options = parse_options(
        [
            "--llm-system-prompt",
            "sys",
            "--llm-short-term-goal",
            "short",
            "--llm-long-term-goal",
            "long",
        ]
        .into_iter(),
    )
    .expect("prompt overrides");
    assert_eq!(options.llm_system_prompt.as_deref(), Some("sys"));
    assert_eq!(options.llm_short_term_goal.as_deref(), Some("short"));
    assert_eq!(options.llm_long_term_goal.as_deref(), Some("long"));
}

#[test]
fn parse_options_accepts_switch_prompt_overrides() {
    let options = parse_options(
        [
            "--prompt-switch-tick",
            "9",
            "--switch-llm-system-prompt",
            "sys2",
            "--switch-llm-short-term-goal",
            "short2",
            "--switch-llm-long-term-goal",
            "long2",
        ]
        .into_iter(),
    )
    .expect("switch prompt overrides");
    assert_eq!(options.prompt_switch_tick, Some(9));
    assert_eq!(options.switch_llm_system_prompt.as_deref(), Some("sys2"));
    assert_eq!(
        options.switch_llm_short_term_goal.as_deref(),
        Some("short2")
    );
    assert_eq!(options.switch_llm_long_term_goal.as_deref(), Some("long2"));
    assert_eq!(options.prompt_switches.len(), 1);
    assert_eq!(options.prompt_switches[0].tick, 9);
}

#[test]
fn parse_options_accepts_prompt_switches_json() {
    let options = parse_options(
        [
            "--prompt-switches-json",
            r#"[{"tick":12,"llm_short_term_goal":"mid"},{"tick":24,"llm_long_term_goal":"late"}]"#,
        ]
        .into_iter(),
    )
    .expect("prompt switches json");
    assert_eq!(options.prompt_switches.len(), 2);
    assert_eq!(options.prompt_switches[0].tick, 12);
    assert_eq!(
        options.prompt_switches[0].llm_short_term_goal.as_deref(),
        Some("mid")
    );
    assert_eq!(options.prompt_switches[1].tick, 24);
    assert_eq!(
        options.prompt_switches[1].llm_long_term_goal.as_deref(),
        Some("late")
    );
}

#[test]
fn parse_options_rejects_invalid_prompt_switches_json() {
    let err = parse_options(["--prompt-switches-json", "not-json"].into_iter())
        .expect_err("invalid prompt switches json");
    assert!(err.contains("invalid --prompt-switches-json"));
}

#[test]
fn parse_options_rejects_prompt_switches_json_without_override_fields() {
    let err = parse_options(["--prompt-switches-json", r#"[{"tick":12}]"#].into_iter())
        .expect_err("missing switch override fields");
    assert!(err.contains("requires at least one llm_* override"));
}

#[test]
fn parse_options_rejects_mixed_legacy_and_prompt_switches_json() {
    let err = parse_options(
        [
            "--prompt-switch-tick",
            "9",
            "--switch-llm-short-term-goal",
            "short2",
            "--prompt-switches-json",
            r#"[{"tick":12,"llm_short_term_goal":"mid"}]"#,
        ]
        .into_iter(),
    )
    .expect_err("mixed legacy and json switch options");
    assert!(err.contains("cannot combine --prompt-switches-json"));
}

#[test]
fn parse_options_rejects_missing_report_json_path() {
    let err = parse_options(["--report-json"].into_iter()).expect_err("missing report path");
    assert!(err.contains("file path"));
}

#[test]
fn parse_options_rejects_zero_ticks() {
    let err = parse_options(["--ticks", "0"].into_iter()).expect_err("reject zero");
    assert!(err.contains("positive integer"));
}

#[test]
fn parse_options_rejects_invalid_llm_io_max_chars() {
    let err = parse_options(["--llm-io-max-chars", "0"].into_iter())
        .expect_err("reject zero llm io max chars");
    assert!(err.contains("positive integer"));
}

#[test]
fn parse_options_rejects_switch_prompt_without_tick() {
    let err = parse_options(["--switch-llm-system-prompt", "sys2"].into_iter())
        .expect_err("switch prompt without tick");
    assert!(err.contains("--prompt-switch-tick"));
}

#[test]
fn parse_options_rejects_switch_tick_without_switch_prompt() {
    let err = parse_options(["--prompt-switch-tick", "4"].into_iter())
        .expect_err("switch tick without switch prompt");
    assert!(err.contains("--switch-llm-"));
}

#[test]
fn truncate_for_llm_io_log_marks_truncation() {
    let truncated = truncate_for_llm_io_log("abcdef", Some(3));
    assert!(truncated.starts_with("abc"));
    assert!(truncated.contains("truncated"));
}

#[test]
fn reject_reason_metric_key_uses_serde_tag_name() {
    let key = reject_reason_metric_key(&RejectReason::InvalidAmount { amount: 0 });
    assert_eq!(key, "invalid_amount");
}

#[test]
fn action_metric_key_uses_serde_tag_name() {
    let key = action_metric_key(&Action::BuildFactory {
        owner: agent_world::simulator::ResourceOwner::Agent {
            agent_id: "agent-0".to_string(),
        },
        location_id: "loc-0".to_string(),
        factory_id: "factory.alpha".to_string(),
        factory_kind: "factory.assembler.mk1".to_string(),
    });
    assert_eq!(key, "build_factory");
}

#[test]
fn observe_action_result_counts_reject_reason_breakdown() {
    let mut report = DemoRunReport::new("llm_bootstrap".to_string(), 1);
    let action_result = ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-0".to_string(),
            max_amount: 1,
        },
        action_id: 1,
        success: false,
        event: WorldEvent {
            id: 1,
            time: 1,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 0 },
            },
        },
    };

    report.observe_action_result(3, &action_result);

    assert_eq!(report.action_success, 0);
    assert_eq!(report.action_failure, 1);
    assert_eq!(report.action_kind_counts.get("harvest_radiation"), Some(&1));
    assert_eq!(
        report.action_kind_failure_counts.get("harvest_radiation"),
        Some(&1)
    );
    assert_eq!(report.first_action_tick.get("harvest_radiation"), Some(&3));
    assert_eq!(
        report.action_reject_reason_counts.get("invalid_amount"),
        Some(&1)
    );
}

#[test]
fn observe_action_result_counts_success_and_first_tick_per_action_kind() {
    let mut report = DemoRunReport::new("llm_bootstrap".to_string(), 1);
    let success = ActionResult {
        action: Action::BuildFactory {
            owner: agent_world::simulator::ResourceOwner::Agent {
                agent_id: "agent-0".to_string(),
            },
            location_id: "loc-0".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        },
        action_id: 7,
        success: true,
        event: WorldEvent {
            id: 7,
            time: 7,
            kind: WorldEventKind::FactoryBuilt {
                owner: agent_world::simulator::ResourceOwner::Agent {
                    agent_id: "agent-0".to_string(),
                },
                location_id: "loc-0".to_string(),
                factory_id: "factory.alpha".to_string(),
                factory_kind: "factory.assembler.mk1".to_string(),
                electricity_cost: 10,
                hardware_cost: 2,
            },
        },
    };
    let failure = ActionResult {
        action: Action::ScheduleRecipe {
            owner: agent_world::simulator::ResourceOwner::Agent {
                agent_id: "agent-0".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.logistics_drone".to_string(),
            batches: 1,
        },
        action_id: 8,
        success: false,
        event: WorldEvent {
            id: 8,
            time: 8,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::FacilityNotFound {
                    facility_id: "factory.alpha".to_string(),
                },
            },
        },
    };

    report.observe_action_result(5, &success);
    report.observe_action_result(9, &failure);

    assert_eq!(report.action_kind_counts.get("build_factory"), Some(&1));
    assert_eq!(report.action_kind_counts.get("schedule_recipe"), Some(&1));
    assert_eq!(
        report.action_kind_success_counts.get("build_factory"),
        Some(&1)
    );
    assert_eq!(
        report.action_kind_failure_counts.get("schedule_recipe"),
        Some(&1)
    );
    assert_eq!(report.first_action_tick.get("build_factory"), Some(&5));
    assert_eq!(report.first_action_tick.get("schedule_recipe"), Some(&9));
}
