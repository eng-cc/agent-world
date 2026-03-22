# Unified Closed-Beta Candidate Release Gate Evidence (2026-03-22)

审计轮次: 6

## Meta
- Gate ID: `GATE-RESET-20260322-CLOSEDBETA`
- Subject: `closed_beta_candidate` release gate that must run on the same candidate for headed Web/UI, pure API, no-UI smoke, longrun/recovery, and QA trend baseline before any stage upgrade.
- Owner role: `qa_engineer`
- Supporting roles: `runtime_engineer` / `viewer_engineer` / `liveops_community`
- Evidence anchors:
  - `doc/playability_test_result/card_2026_03_22_15_56_13.md`
  - `doc/playability_test_result/card_2026_03_19_09_40_56.md`
  - `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md`
  - `doc/testing/evidence/post-onboarding-headless-smoke-2026-03-19.md`
  - `doc/testing/evidence/closed-beta-runtime-s10-2026-03-22.md`
  - `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
  - `doc/testing/evidence/testing-quality-trend-baseline-2026-03-11.md`

## Gate Status Table
| Lane | Owner | Marker Evidence | Current Status | Next Action |
| --- | --- | --- | --- | --- |
| Headed Web/UI `#46` | `viewer_engineer` / `qa_engineer` | `doc/playability_test_result/card_2026_03_22_15_56_13.md` / `output/playwright/playability/closed-beta-20260322/post-onboarding-20260322-155613/post-onboarding-summary.md` | `pass`。同候选 fresh bundle rerun `output/playwright/playability/closed-beta-20260322/post-onboarding-20260322-155613` 自动检查全绿，人工复核确认 `PostOnboarding` 主目标与顶部总结保持首屏焦点，`AgentNotFound` 历史噪音已不再占据右侧 chatter 焦点。 | 保持该 lane 为 `pass`；仅在 candidate 或 viewer 首屏布局再次变更时补跑。右侧操作反馈栏仍保留历史 `AgentNotFound` 记录，当前列为非阻断观察项。 |
| Pure API parity | `runtime_engineer` / `qa_engineer` | `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md` / `output/playwright/playability/pure-api-required-20260322-183650/pure-api-summary.md` / `output/playwright/playability/pure-api-full-20260322-183750/pure-api-summary.md` | `pass`。同候选 fresh bundle 已完成 no-LLM required/full rerun；`output/playwright/playability/pure-api-required-20260322-183650/` 与 `output/playwright/playability/pure-api-full-20260322-183750/` 均到达 `post_onboarding.choose_midloop_path`、`progress=100`，并继续保持 `reconnect-sync` 恢复能力。对应 bootstrap 日志位于 `run-game-test.log`，底层启动日志目录分别为 `output/playwright/playability/startup-20260322-183721/` 与 `output/playwright/playability/startup-20260322-183750/`。 | 保持该 lane 为 `pass`；仅在 candidate、canonical `player_gameplay` 语义或正式 `gameplay_action` / `reconnect-sync` 路径再次变更时补跑。 |
| No-UI live smoke | `liveops_community` / `qa_engineer` | `doc/testing/evidence/post-onboarding-headless-smoke-2026-03-19.md` / `output/playwright/playability/post-onboarding-headless-20260322-183832/post-onboarding-headless-summary.md` | `pass`。同候选 fresh bundle `output/playwright/playability/post-onboarding-headless-20260322-183832/` 已重放无 UI live-protocol smoke，确认同会话 `step(8) -> step(24)` 继续返回 `advanced`，时间线为 `1 -> 9 -> 33`，且 event stream 非空并包含 `RuntimeEvent`。 | 保持该 lane 为 `pass`；仅在 candidate、viewer live 协议或 `PostOnboarding` 阶段承接语义再次变更时补跑。 |
| Longrun & recovery | `runtime_engineer` | `doc/testing/longrun/s10-five-node-real-game-soak.prd.md` / `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md` / `doc/testing/evidence/closed-beta-runtime-s10-2026-03-22.md` | `pass`。clean-room `600s+` 候选样本 `output/longrun/closed-beta-candidate-20260322/20260322-121320` 已 `process_status=ok / metric_gate=pass`，两条 replay/rollback required-tier drill 也均已通过，runtime lane 证据包已可作为 unified gate 输入。 | 保持该 lane 为 `pass`，仅在 candidate 变更或 runtime 行为回归时补跑。 |
| Trend baseline | `qa_engineer` | `doc/testing/evidence/testing-quality-trend-baseline-2026-03-11.md` | `pass`。最近 7 天窗口（`2026-03-19` ~ `2026-03-22`）已刷新为 7 个样本、`first-pass=100%`、`escape=0%`、`fix-time=0d`，当前阶段评审使用的 trend baseline 已达到升级阈值。 | 保持该 lane 为 `pass`；按周续写最近 7 天窗口，若后续样本把指标打回阈值以下，需同步把 unified gate 回退为 `block`。 |

## QA Verdict
- 当前统一 gate 结论: `pass`
- 允许的结论:
  - 可以将本 gate 文档作为 `TASK-GAME-031` 的 QA 汇总入口，并交 `producer_system_designer` 执行 `TASK-GAME-033` 阶段评审。
  - 不可以把当前 gate 文档当成 `closed_beta_candidate approved` 或 `TASK-GAME-033 go` 证据。

## Gate Execution Notes
- Candidate definition: use the fresh bundle that passes `TASK-GAME-018` evidence (see `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md`) plus pure API parity smoke artifacts; reference this candidate in all lane logs.
- Gate rule: every lane must run on the same candidate tag/version/date; mixing old evidence is not allowed. Log command + stdout path for each lane (build the evidence bundle folder under `output/playwright/playability/closed-beta-...`).
- Blocking conditions:
  - Any `blocking` failure from the gate (e.g., longrun soak timed out, headed Web noise persists, parity regression) immediately keeps stage at `internal_playable_alpha_late`.
  - 若最近 7 天 trend baseline 再次跌破阈值，必须把 unified gate 从 `pass` 回退为 `block`。
  - unified gate `pass` 只代表 QA 技术门已收口；项目阶段是否升级仍由 `producer_system_designer` 在 `TASK-GAME-033` 中拍板。

## Next Steps
1. 将本 gate 文档交 `producer_system_designer`，执行 `TASK-GAME-033 / TASK-GAMEPLAY-CB-006` 阶段评审。
2. 在 producer 正式拍板前，继续维持 `internal_playable_alpha_late` 与 `technical preview` 对外口径，不得提前宣称 `closed_beta_candidate approved`。
3. `qa_engineer` 按周续写最近 7 天窗口 baseline；若指标回落或任一 lane 回归，立即把 unified gate 回退为 `block`。

## Outstanding Inputs
- Producer stage decision for `TASK-GAME-033 / TASK-GAMEPLAY-CB-006`.
- Confirmation from `liveops_community` that no new high-visibility communication (e.g., “closed beta” or “play now”) leaked before the gate passes.
