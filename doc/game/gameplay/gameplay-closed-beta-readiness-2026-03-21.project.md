# Gameplay 封闭 Beta 准入门禁（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-CB-001 (`PRD-GAME-009`) [test_tier_required]: 冻结“当前处于 internal_playable_alpha_late、下一阶段目标为 closed_beta_candidate”的正式口径，并完成 `game` 根 PRD / project、`gameplay-top-level-design` 主文档、索引、handoff 与 devlog 挂载。
- [ ] TASK-GAMEPLAY-CB-002 (`PRD-GAME-009`) [test_tier_required + test_tier_full]: `runtime_engineer` 收口 five-node no-LLM soak、replay/rollback drill 与 longrun release gate 的候选版本证据，证明当前技术底座已达到封闭 Beta 准入下限。
- [ ] TASK-GAMEPLAY-CB-003 (`PRD-GAME-009`) [test_tier_required]: `viewer_engineer` 收口 `PostOnboarding` 首屏降噪、主目标优先级与玩家入口 full-coverage gate 的最小产品化包。
- [ ] TASK-GAMEPLAY-CB-004 (`PRD-GAME-009`) [test_tier_required + test_tier_full]: `qa_engineer` 建立统一 `closed_beta_candidate` release gate，汇总 headed Web/UI、pure API、no-UI smoke、longrun/recovery 与趋势基线，输出阻断或放行建议。
- [x] TASK-GAMEPLAY-CB-005 (`PRD-GAME-009`) [test_tier_required]: `liveops_community` 收口封闭 Beta 候选招募/反馈/事故回流 runbook 与对外禁语清单，在 producer 放行前保持 `technical preview` 口径。
- [ ] TASK-GAMEPLAY-CB-006 (`PRD-GAME-009`) [test_tier_required]: `producer_system_designer` 基于 `TASK-GAMEPLAY-CB-002/003/004/005` 的统一证据执行阶段评审，决定继续保持 `internal_playable_alpha_late` 还是升级为 `closed_beta_candidate`。

## 依赖

- `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`
- `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.prd.md`
- `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- `doc/testing/evidence/testing-quality-trend-baseline-2026-03-11.md`
- `doc/world-simulator/viewer/viewer-release-full-coverage-gate.prd.md`
- `doc/readme/governance/readme-moltbook-liveops-runbook-2026-03-21.md`
- `testing-manual.md`

## 状态

- 更新日期: 2026-03-22
- 当前状态: in_progress
- 当前 owner: `producer_system_designer`
- 下一任务: `TASK-GAMEPLAY-CB-002`
- 已完成补充:
  - `TASK-GAMEPLAY-CB-005` 已交付 `doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.{prd,design,project}.md`、`doc/playability_test_result/templates/closed-beta-candidate-{feedback-log-guide,incident-templates}-2026-03-22.md`，并把 `technical preview` 口径边界回写到 `readme` 模块正式追踪。
- 阻断条件:
  - 若 `TASK-GAMEPLAY-CB-002` 仍只拿到“隔离端口诊断 pass、默认端口候选样本 fail”的混合证据，则不得把 runtime lane 视为收口。
  - 若统一 release gate 尚未建立，则不得使用 `closed beta` 口径。
  - 若 trend baseline 未达标，则不得把当前阶段从 `internal_playable_alpha_late` 提升到 `closed_beta_candidate`。
  - 若 liveops 对外 runbook 仍缺招募/反馈/事故回流模板，则不得扩大对外承诺。
- 说明:
  - 本专题的目标是“阶段准入治理”，不是新增玩法功能。
  - 允许保持当前阶段，只要阻断项被如实记录并重新拆回 owner。
  - `TASK-GAMEPLAY-CB-002` 当前最新事实：默认端口 five-node soak 样本仍会被外部 `SIGTERM`/脏端口环境污染；`--base-port 5910` 的 60 秒诊断样本可过，但尚不足以作为正式 candidate 放行证据。
