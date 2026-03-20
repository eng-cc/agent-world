use oasis7::simulator::{RejectReason, WorldEventKind};
use std::collections::BTreeMap;

use crate::web_test_api::WebTestApiControlFeedbackSnapshot;
use crate::ViewerState;

#[cfg(test)]
use oasis7::simulator::RunnerMetrics;

const WAR_BASE_DURATION_TICKS: u64 = 6;
const WAR_DURATION_PER_INTENSITY_TICKS: u64 = 2;
const ACTION_DELAY_GRACE_TICKS: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerMicroLoopTone {
    Positive,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerActionStatusSnapshot {
    pub(super) headline: String,
    pub(super) detail: String,
    pub(super) tone: PlayerMicroLoopTone,
    pub(super) pending_eta_ticks: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerDueTimerSnapshot {
    pub(super) label: String,
    pub(super) remaining_ticks: u64,
    pub(super) overdue_ticks: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerMicroLoopSnapshot {
    pub(super) action_status: PlayerActionStatusSnapshot,
    pub(super) due_timers: Vec<PlayerDueTimerSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerNoProgressDiagnosis {
    pub(super) reason: String,
    pub(super) suggestion: String,
}

#[derive(Clone, Debug)]
struct PendingActionAck {
    action_id: u64,
    action_kind: String,
    actor_id: String,
    accepted_at_tick: u64,
    eta_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DueTimerKind {
    War,
    Governance,
    Crisis,
    Contract,
}

#[derive(Clone, Debug)]
struct DueTimerState {
    kind: DueTimerKind,
    id: String,
    due_tick: u64,
}

pub(super) fn build_player_micro_loop_snapshot(
    state: &ViewerState,
    locale: crate::i18n::UiLocale,
) -> PlayerMicroLoopSnapshot {
    let current_tick = player_current_tick(state);
    let mut pending_action = None::<PendingActionAck>;
    let mut latest_reject = None::<(u64, String)>;
    let mut latest_gameplay_runtime_kind = None::<(u64, String)>;

    let mut due_timers = BTreeMap::<(DueTimerKind, String), DueTimerState>::new();

    for event in &state.events {
        match &event.kind {
            WorldEventKind::RuntimeEvent { kind, domain_kind } => {
                if kind == "runtime.action_accepted" {
                    if let Some(summary) = domain_kind.as_deref() {
                        let action_id = summary_u64(summary, "action_id").unwrap_or(event.id);
                        let action_kind = summary_value(summary, "action_kind")
                            .unwrap_or("unknown_action")
                            .to_string();
                        let actor_id = summary_value(summary, "actor_id")
                            .unwrap_or("system")
                            .to_string();
                        let eta_ticks = summary_u64(summary, "eta_ticks").unwrap_or(0);
                        pending_action = Some(PendingActionAck {
                            action_id,
                            action_kind,
                            actor_id,
                            accepted_at_tick: event.time,
                            eta_ticks,
                        });
                    }
                    continue;
                }

                if !kind.starts_with("runtime.gameplay.") {
                    continue;
                }
                latest_gameplay_runtime_kind = Some((event.time, kind.clone()));

                let Some(summary) = domain_kind.as_deref() else {
                    continue;
                };

                match kind.as_str() {
                    "runtime.gameplay.war_declared" => {
                        let war_id = summary_value(summary, "war_id").unwrap_or("unknown_war");
                        let intensity = summary_u64(summary, "intensity").unwrap_or(1);
                        let due_tick = event
                            .time
                            .saturating_add(WAR_BASE_DURATION_TICKS)
                            .saturating_add(
                                intensity.saturating_mul(WAR_DURATION_PER_INTENSITY_TICKS),
                            );
                        due_timers.insert(
                            (DueTimerKind::War, war_id.to_string()),
                            DueTimerState {
                                kind: DueTimerKind::War,
                                id: war_id.to_string(),
                                due_tick,
                            },
                        );
                    }
                    "runtime.gameplay.war_concluded" => {
                        if let Some(war_id) = summary_value(summary, "war_id") {
                            due_timers.remove(&(DueTimerKind::War, war_id.to_string()));
                        }
                    }
                    "runtime.gameplay.governance_proposal_opened" => {
                        if let (Some(proposal_key), Some(closes_at)) = (
                            summary_value(summary, "proposal_key"),
                            summary_u64(summary, "closes_at"),
                        ) {
                            due_timers.insert(
                                (DueTimerKind::Governance, proposal_key.to_string()),
                                DueTimerState {
                                    kind: DueTimerKind::Governance,
                                    id: proposal_key.to_string(),
                                    due_tick: closes_at,
                                },
                            );
                        }
                    }
                    "runtime.gameplay.governance_proposal_finalized" => {
                        if let Some(proposal_key) = summary_value(summary, "proposal_key") {
                            due_timers
                                .remove(&(DueTimerKind::Governance, proposal_key.to_string()));
                        }
                    }
                    "runtime.gameplay.crisis_spawned" => {
                        if let (Some(crisis_id), Some(expires_at)) = (
                            summary_value(summary, "crisis_id"),
                            summary_u64(summary, "expires_at"),
                        ) {
                            due_timers.insert(
                                (DueTimerKind::Crisis, crisis_id.to_string()),
                                DueTimerState {
                                    kind: DueTimerKind::Crisis,
                                    id: crisis_id.to_string(),
                                    due_tick: expires_at,
                                },
                            );
                        }
                    }
                    "runtime.gameplay.crisis_resolved" | "runtime.gameplay.crisis_timed_out" => {
                        if let Some(crisis_id) = summary_value(summary, "crisis_id") {
                            due_timers.remove(&(DueTimerKind::Crisis, crisis_id.to_string()));
                        }
                    }
                    "runtime.gameplay.economic_contract_opened" => {
                        if let (Some(contract_id), Some(expires_at)) = (
                            summary_value(summary, "contract_id"),
                            summary_u64(summary, "expires_at"),
                        ) {
                            due_timers.insert(
                                (DueTimerKind::Contract, contract_id.to_string()),
                                DueTimerState {
                                    kind: DueTimerKind::Contract,
                                    id: contract_id.to_string(),
                                    due_tick: expires_at,
                                },
                            );
                        }
                    }
                    "runtime.gameplay.economic_contract_settled"
                    | "runtime.gameplay.economic_contract_expired" => {
                        if let Some(contract_id) = summary_value(summary, "contract_id") {
                            due_timers.remove(&(DueTimerKind::Contract, contract_id.to_string()));
                        }
                    }
                    _ => {}
                }
            }
            WorldEventKind::ActionRejected { reason } => {
                latest_reject = Some((event.time, reject_reason_summary(reason, locale)));
            }
            _ => {}
        }
    }

    let action_status = build_action_status(
        current_tick,
        pending_action.as_ref(),
        latest_reject,
        latest_gameplay_runtime_kind,
        locale,
    );

    let mut timer_values = due_timers.into_values().collect::<Vec<_>>();
    timer_values.sort_by(|left, right| {
        left.due_tick
            .cmp(&right.due_tick)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    let due_timers = timer_values
        .into_iter()
        .map(|timer| {
            let remaining_ticks = timer.due_tick.saturating_sub(current_tick);
            let overdue_ticks = current_tick.saturating_sub(timer.due_tick);
            PlayerDueTimerSnapshot {
                label: timer_label(timer.kind, timer.id.as_str(), locale),
                remaining_ticks,
                overdue_ticks,
            }
        })
        .collect::<Vec<_>>();

    PlayerMicroLoopSnapshot {
        action_status,
        due_timers,
    }
}

pub(super) fn build_player_no_progress_diagnosis(
    control_feedback: Option<&WebTestApiControlFeedbackSnapshot>,
    micro_loop: &PlayerMicroLoopSnapshot,
    locale: crate::i18n::UiLocale,
) -> PlayerNoProgressDiagnosis {
    if let Some(feedback) = control_feedback {
        if matches!(feedback.stage.as_str(), "blocked" | "completed_no_progress") {
            let reason = feedback
                .reason
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    if locale.is_zh() {
                        "控制链路没有产出推进增量".to_string()
                    } else {
                        "Control path produced no progression delta".to_string()
                    }
                });
            let suggestion = feedback
                .hint
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| default_recovery_suggestion(locale));
            return PlayerNoProgressDiagnosis { reason, suggestion };
        }
    }

    match micro_loop.action_status.tone {
        PlayerMicroLoopTone::Warning => {
            let suggestion = if locale.is_zh() {
                "优先修复阻断原因，再点击“恢复：play”或“重试：step x8”验证。".to_string()
            } else {
                "Resolve the blocker first, then run Recover: play or Retry: step x8.".to_string()
            };
            PlayerNoProgressDiagnosis {
                reason: micro_loop.action_status.detail.clone(),
                suggestion,
            }
        }
        PlayerMicroLoopTone::Positive | PlayerMicroLoopTone::Info => {
            if micro_loop.action_status.pending_eta_ticks.is_some() {
                let suggestion = if locale.is_zh() {
                    "动作仍在 ETA 窗口内：先继续推进 tick，再观察是否进入落地事件。".to_string()
                } else {
                    "Action is still inside ETA window: advance ticks and watch for resolution events."
                        .to_string()
                };
                PlayerNoProgressDiagnosis {
                    reason: micro_loop.action_status.detail.clone(),
                    suggestion,
                }
            } else if micro_loop
                .action_status
                .headline
                .contains(if locale.is_zh() { "延迟" } else { "Delayed" })
            {
                PlayerNoProgressDiagnosis {
                    reason: micro_loop.action_status.detail.clone(),
                    suggestion: default_recovery_suggestion(locale),
                }
            } else {
                let reason = if locale.is_zh() {
                    "未检测到新增推进事件".to_string()
                } else {
                    "No new progression event detected".to_string()
                };
                PlayerNoProgressDiagnosis {
                    reason,
                    suggestion: default_recovery_suggestion(locale),
                }
            }
        }
    }
}

pub(super) fn format_due_timer_line(
    timer: &PlayerDueTimerSnapshot,
    locale: crate::i18n::UiLocale,
) -> String {
    if timer.overdue_ticks > 0 {
        if locale.is_zh() {
            format!("{}：已到期 +{} tick", timer.label, timer.overdue_ticks)
        } else if timer.overdue_ticks == 1 {
            format!("{}: due +1 tick", timer.label)
        } else {
            format!("{}: due +{} ticks", timer.label, timer.overdue_ticks)
        }
    } else if locale.is_zh() {
        format!("{}：T-{} tick", timer.label, timer.remaining_ticks)
    } else {
        format!("{}: T-{} ticks", timer.label, timer.remaining_ticks)
    }
}

fn build_action_status(
    current_tick: u64,
    pending_action: Option<&PendingActionAck>,
    latest_reject: Option<(u64, String)>,
    latest_gameplay_runtime_kind: Option<(u64, String)>,
    locale: crate::i18n::UiLocale,
) -> PlayerActionStatusSnapshot {
    if let Some((reject_tick, reason)) = latest_reject.as_ref() {
        let reject_after_accept = pending_action
            .map(|accepted| *reject_tick >= accepted.accepted_at_tick)
            .unwrap_or(true);
        if reject_after_accept {
            return PlayerActionStatusSnapshot {
                headline: if locale.is_zh() {
                    "最近动作：已阻断".to_string()
                } else {
                    "Recent Action: Blocked".to_string()
                },
                detail: reason.clone(),
                tone: PlayerMicroLoopTone::Warning,
                pending_eta_ticks: None,
            };
        }
    }

    if let Some(ack) = pending_action {
        let due_tick = ack.accepted_at_tick.saturating_add(ack.eta_ticks);
        if let Some((event_tick, event_kind)) = latest_gameplay_runtime_kind {
            if event_tick >= ack.accepted_at_tick {
                return PlayerActionStatusSnapshot {
                    headline: if locale.is_zh() {
                        "最近动作：已落地".to_string()
                    } else {
                        "Recent Action: Resolved".to_string()
                    },
                    detail: if locale.is_zh() {
                        format!(
                            "action#{}, {} -> {}",
                            ack.action_id,
                            ack.action_kind,
                            gameplay_kind_zh(&event_kind)
                        )
                    } else {
                        format!(
                            "action#{}, {} -> {}",
                            ack.action_id,
                            ack.action_kind,
                            gameplay_kind_en(&event_kind)
                        )
                    },
                    tone: PlayerMicroLoopTone::Positive,
                    pending_eta_ticks: None,
                };
            }
        }

        if current_tick <= due_tick.saturating_add(ACTION_DELAY_GRACE_TICKS) {
            let pending_eta_ticks = due_tick.saturating_sub(current_tick);
            return PlayerActionStatusSnapshot {
                headline: if locale.is_zh() {
                    "最近动作：已接受".to_string()
                } else {
                    "Recent Action: Accepted".to_string()
                },
                detail: if locale.is_zh() {
                    format!(
                        "action#{} {} (actor={})，ETA {} tick",
                        ack.action_id, ack.action_kind, ack.actor_id, pending_eta_ticks
                    )
                } else {
                    format!(
                        "action#{} {} (actor={}), ETA {} ticks",
                        ack.action_id, ack.action_kind, ack.actor_id, pending_eta_ticks
                    )
                },
                tone: PlayerMicroLoopTone::Info,
                pending_eta_ticks: Some(pending_eta_ticks),
            };
        }

        let overdue_ticks = current_tick.saturating_sub(due_tick);
        return PlayerActionStatusSnapshot {
            headline: if locale.is_zh() {
                "最近动作：延迟".to_string()
            } else {
                "Recent Action: Delayed".to_string()
            },
            detail: if locale.is_zh() {
                format!(
                    "action#{} {} 超过 ETA +{} tick 仍未见 gameplay 回执",
                    ack.action_id, ack.action_kind, overdue_ticks
                )
            } else {
                format!(
                    "action#{} {} exceeded ETA by +{} ticks without gameplay resolution",
                    ack.action_id, ack.action_kind, overdue_ticks
                )
            },
            tone: PlayerMicroLoopTone::Warning,
            pending_eta_ticks: None,
        };
    }

    if let Some((_, event_kind)) = latest_gameplay_runtime_kind {
        PlayerActionStatusSnapshot {
            headline: if locale.is_zh() {
                "最近动作：世界已更新".to_string()
            } else {
                "Recent Action: World Updated".to_string()
            },
            detail: if locale.is_zh() {
                format!("最近 gameplay 事件：{}", gameplay_kind_zh(&event_kind))
            } else {
                format!("Latest gameplay event: {}", gameplay_kind_en(&event_kind))
            },
            tone: PlayerMicroLoopTone::Positive,
            pending_eta_ticks: None,
        }
    } else {
        PlayerActionStatusSnapshot {
            headline: if locale.is_zh() {
                "最近动作：暂无".to_string()
            } else {
                "Recent Action: None".to_string()
            },
            detail: if locale.is_zh() {
                "尚未接收到 ActionAccepted 或 gameplay 事件。".to_string()
            } else {
                "No ActionAccepted or gameplay runtime event observed yet.".to_string()
            },
            tone: PlayerMicroLoopTone::Info,
            pending_eta_ticks: None,
        }
    }
}

fn timer_label(kind: DueTimerKind, id: &str, locale: crate::i18n::UiLocale) -> String {
    let short_id = super::truncate_observe_text(id, 22);
    match (kind, locale.is_zh()) {
        (DueTimerKind::War, true) => format!("战争 {}", short_id),
        (DueTimerKind::War, false) => format!("War {}", short_id),
        (DueTimerKind::Governance, true) => format!("治理 {}", short_id),
        (DueTimerKind::Governance, false) => format!("Governance {}", short_id),
        (DueTimerKind::Crisis, true) => format!("危机 {}", short_id),
        (DueTimerKind::Crisis, false) => format!("Crisis {}", short_id),
        (DueTimerKind::Contract, true) => format!("合约 {}", short_id),
        (DueTimerKind::Contract, false) => format!("Contract {}", short_id),
    }
}

fn gameplay_kind_zh(kind: &str) -> &'static str {
    match kind {
        "runtime.gameplay.war_declared" => "战争开启",
        "runtime.gameplay.war_concluded" => "战争结算",
        "runtime.gameplay.governance_proposal_opened" => "提案开启",
        "runtime.gameplay.governance_proposal_finalized" => "提案结算",
        "runtime.gameplay.crisis_spawned" => "危机生成",
        "runtime.gameplay.crisis_resolved" => "危机解决",
        "runtime.gameplay.crisis_timed_out" => "危机超时",
        "runtime.gameplay.economic_contract_opened" => "合约开启",
        "runtime.gameplay.economic_contract_settled" => "合约结算",
        "runtime.gameplay.economic_contract_expired" => "合约过期",
        "runtime.gameplay.meta_progress_granted" => "元进度发放",
        _ => "gameplay 更新",
    }
}

fn gameplay_kind_en(kind: &str) -> &'static str {
    match kind {
        "runtime.gameplay.war_declared" => "war declared",
        "runtime.gameplay.war_concluded" => "war concluded",
        "runtime.gameplay.governance_proposal_opened" => "proposal opened",
        "runtime.gameplay.governance_proposal_finalized" => "proposal finalized",
        "runtime.gameplay.crisis_spawned" => "crisis spawned",
        "runtime.gameplay.crisis_resolved" => "crisis resolved",
        "runtime.gameplay.crisis_timed_out" => "crisis timed out",
        "runtime.gameplay.economic_contract_opened" => "contract opened",
        "runtime.gameplay.economic_contract_settled" => "contract settled",
        "runtime.gameplay.economic_contract_expired" => "contract expired",
        "runtime.gameplay.meta_progress_granted" => "meta progress granted",
        _ => "gameplay update",
    }
}

fn summary_value<'a>(summary: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{key}=");
    let start = summary.find(needle.as_str())?;
    let value_start = start + needle.len();
    let rest = &summary[value_start..];
    let value_end = rest.find(' ').unwrap_or(rest.len());
    let value = rest[..value_end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn summary_u64(summary: &str, key: &str) -> Option<u64> {
    summary_value(summary, key)?.parse::<u64>().ok()
}

fn player_current_tick(state: &ViewerState) -> u64 {
    state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0)
}

fn reject_reason_summary(reason: &RejectReason, locale: crate::i18n::UiLocale) -> String {
    match (reason, locale.is_zh()) {
        (
            RejectReason::InsufficientResource {
                owner,
                kind,
                requested,
                available,
            },
            true,
        ) => format!("资源不足: {:?} {:?} 需{} 可用{}", owner, kind, requested, available),
        (
            RejectReason::InsufficientResource {
                owner,
                kind,
                requested,
                available,
            },
            false,
        ) => format!(
            "Insufficient resource: owner={owner:?} kind={kind:?} requested={requested} available={available}"
        ),
        (RejectReason::RuleDenied { notes }, true) => {
            if notes.is_empty() {
                "规则拒绝：未提供详细说明".to_string()
            } else {
                format!("规则拒绝：{}", notes.join("; "))
            }
        }
        (RejectReason::RuleDenied { notes }, false) => {
            if notes.is_empty() {
                "Rule denied without details".to_string()
            } else {
                format!("Rule denied: {}", notes.join("; "))
            }
        }
        (RejectReason::AgentNotFound { agent_id }, true) => {
            format!("目标 Agent 不存在: {agent_id}")
        }
        (RejectReason::AgentNotFound { agent_id }, false) => {
            format!("Target agent not found: {agent_id}")
        }
        (other, true) => format!("动作被拒绝: {other:?}"),
        (other, false) => format!("Action rejected: {other:?}"),
    }
}

fn default_recovery_suggestion(locale: crate::i18n::UiLocale) -> String {
    if locale.is_zh() {
        "点击“恢复：play”后重试“step x8”，若仍无变化请发一条更明确的指令。".to_string()
    } else {
        "Click Recover: play, then retry step x8. If still unchanged, send a more explicit command."
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConnectionStatus, ViewerState};
    use oasis7::simulator::WorldEvent;

    fn runtime_event(id: u64, time: u64, kind: &str, domain_kind: Option<&str>) -> WorldEvent {
        WorldEvent {
            id,
            time,
            kind: WorldEventKind::RuntimeEvent {
                kind: kind.to_string(),
                domain_kind: domain_kind.map(|value| value.to_string()),
            },
            runtime_event: None,
        }
    }

    fn viewer_state(total_ticks: u64, events: Vec<WorldEvent>) -> ViewerState {
        ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: None,
            events,
            decision_traces: Vec::new(),
            metrics: Some(RunnerMetrics {
                total_ticks,
                ..RunnerMetrics::default()
            }),
        }
    }

    #[test]
    fn micro_loop_snapshot_tracks_pending_ack_status() {
        let state = viewer_state(
            12,
            vec![runtime_event(
                1,
                10,
                "runtime.action_accepted",
                Some("action_id=44 action_kind=declare_war actor_id=agent-1 eta_ticks=5"),
            )],
        );

        let snapshot = build_player_micro_loop_snapshot(&state, crate::i18n::UiLocale::EnUs);
        assert_eq!(snapshot.action_status.headline, "Recent Action: Accepted");
        assert_eq!(snapshot.action_status.pending_eta_ticks, Some(3));
        assert!(snapshot.action_status.detail.contains("action#44"));
    }

    #[test]
    fn micro_loop_snapshot_tracks_due_timers_and_clears_resolved_ones() {
        let state = viewer_state(
            20,
            vec![
                runtime_event(
                    1,
                    5,
                    "runtime.gameplay.war_declared",
                    Some("war_id=war.alpha objective=frontline intensity=3"),
                ),
                runtime_event(
                    2,
                    6,
                    "runtime.gameplay.crisis_spawned",
                    Some("crisis_id=crisis.beta kind=supply_shock severity=2 expires_at=14"),
                ),
                runtime_event(
                    3,
                    7,
                    "runtime.gameplay.governance_proposal_opened",
                    Some("proposal_key=gov-1 title=budget closes_at=26"),
                ),
                runtime_event(
                    4,
                    8,
                    "runtime.gameplay.economic_contract_opened",
                    Some(
                        "contract_id=contract-1 counterparty=agent-x settlement_amount=6 expires_at=25",
                    ),
                ),
                runtime_event(
                    5,
                    9,
                    "runtime.gameplay.crisis_resolved",
                    Some("crisis_id=crisis.beta strategy=reroute success=true impact=8"),
                ),
            ],
        );

        let snapshot = build_player_micro_loop_snapshot(&state, crate::i18n::UiLocale::EnUs);
        let labels = snapshot
            .due_timers
            .iter()
            .map(|timer| timer.label.clone())
            .collect::<Vec<_>>();
        assert_eq!(labels.len(), 3);
        assert!(labels.iter().any(|label| label.contains("War war.alpha")));
        assert!(labels
            .iter()
            .any(|label| label.contains("Governance gov-1")));
        assert!(labels
            .iter()
            .any(|label| label.contains("Contract contract-1")));
        assert!(!labels.iter().any(|label| label.contains("Crisis")));
    }

    #[test]
    fn no_progress_diagnosis_prefers_control_feedback_reason_and_hint() {
        let snapshot = PlayerMicroLoopSnapshot {
            action_status: PlayerActionStatusSnapshot {
                headline: "Recent Action: Delayed".to_string(),
                detail: "pending".to_string(),
                tone: PlayerMicroLoopTone::Warning,
                pending_eta_ticks: None,
            },
            due_timers: Vec::new(),
        };
        let feedback = WebTestApiControlFeedbackSnapshot {
            action: "play".to_string(),
            stage: "completed_no_progress".to_string(),
            reason: Some("Cause: completion ack timeout_no_progress".to_string()),
            hint: Some("Next: click Recover: play".to_string()),
            effect: "none".to_string(),
            delta_logical_time: 0,
            delta_event_seq: 0,
            delta_trace_count: 0,
        };
        let diagnosis = build_player_no_progress_diagnosis(
            Some(&feedback),
            &snapshot,
            crate::i18n::UiLocale::EnUs,
        );
        assert_eq!(
            diagnosis.reason,
            "Cause: completion ack timeout_no_progress"
        );
        assert_eq!(diagnosis.suggestion, "Next: click Recover: play");
    }
}
