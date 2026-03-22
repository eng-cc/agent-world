# game PRD Project

审计轮次: 8

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-GAME-001 (PRD-GAME-001) [test_tier_required]: 完成 game PRD 改写，建立玩法设计总入口。
- [x] TASK-GAME-002 (PRD-GAME-001/002) [test_tier_required]: 补齐核心玩法循环（新手/经济/战争）验收矩阵。
- [x] TASK-GAME-003 (PRD-GAME-002/003) [test_tier_required]: 建立可玩性问题分级与修复闭环模板。
- [x] TASK-GAME-004 (PRD-GAME-003) [test_tier_required]: 对接发布前可玩性门禁与回归节奏。
- [x] TASK-GAME-005 (PRD-GAME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-GAME-006 (PRD-GAME-004) [test_tier_required]: 新增微循环反馈可见性 PRD 与项目文档，完成文档树挂载。
- [x] TASK-GAME-007 (PRD-GAME-004) [test_tier_required]: 落地 runtime 协议与 viewer 反馈闭环并完成回归验证（子任务 `TASK-GAMEPLAY-MLF-001/002/003/004` 已全部完成，见 `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.project.md`）。
- [x] TASK-GAME-008 (PRD-GAME-005) [test_tier_required]: 新增“分布式执行共识/治理共识/身份与反女巫”专题 PRD 与项目管理文档，完成根文档追踪映射。
- [x] TASK-GAME-009 (PRD-GAME-005) [test_tier_required]: 落地 tick 证书链与 `state_root/events_hash` 一致性校验实现（含 replay/save-load 闭环）。
- [x] TASK-GAME-010 (PRD-GAME-005) [test_tier_required]: 落地治理 `timelock + epoch` 生效门禁与紧急刹车/否决约束。
- [x] TASK-GAME-011 (PRD-GAME-005) [test_tier_required + test_tier_full]: 落地身份信誉/抵押权重、女巫检测与惩罚申诉闭环。
- [x] TASK-GAME-012 (PRD-GAME-006) [test_tier_required]: 新增长期在线 P0 生产硬化专题 PRD 与项目管理文档，完成根文档追踪映射。
- [x] TASK-GAME-013 (PRD-GAME-006) [test_tier_required]: 落地状态权威分层（传播层/裁决层）与冲突仲裁拒绝路径。
- [x] TASK-GAME-014 (PRD-GAME-006) [test_tier_required + test_tier_full]: 补齐确定性回放 + 快照回滚 runbook 与演练门禁。
- [x] TASK-GAME-015 (PRD-GAME-006) [test_tier_required + test_tier_full]: 落地反作弊/反女巫对抗检测、惩罚与申诉证据链强化。
- [x] TASK-GAME-016 (PRD-GAME-006) [test_tier_required]: 建立经济源汇审计与通胀/套利告警阈值门禁。
- [x] TASK-GAME-017 (PRD-GAME-006) [test_tier_required]: 补齐可运维性能力（SLO、告警、灰度、灾备演练）与发布阻断规则。
- [x] TASK-GAME-018 (PRD-GAME-004) [test_tier_required]: 执行微循环可玩性视觉优化二期（控制结果显著化、玩家模式减负、世界可读性增强）并以手动截图闭环验收（见 `TASK-GAMEPLAY-MLF-005/006/007/008`）。
- [x] TASK-GAME-019 (PRD-GAME-001) [test_tier_required]: 同步 `doc/game/README.md` 与 `doc/game/prd.index.md` 的模块入口索引，补齐近期 gameplay 专题与根目录收口口径。
  - 收口证据:
    - `doc/game/gameplay/gameplay-micro-loop-visual-closure-evidence-2026-03-10-round009.md`
    - `doc/playability_test_result/card_2026_03_10_23_27_43.md`
    - `doc/game/gameplay/gameplay-visual-evidence-linkage-2026-03-10.md`
  - QA 结论:
    - `TASK-GAMEPLAY-MLF-005/006/007/008` 已全部完成，当前未见高优先级阻断；更长录屏留给后续 release gate 抽样继续观察。
- [x] TASK-GAME-020 (PRD-GAME-001/002) [test_tier_required]: 冻结前期工业引导闭环（首个制成品/工厂），并拆出 runtime / viewer / QA 落地任务与验收指标。
- [x] TASK-GAME-021 (PRD-GAME-007) [test_tier_required]: 新增 `PostOnboarding` 阶段目标链专题 PRD / design / project，并完成 `game` 根 PRD、顶层设计主文档、索引与 devlog 挂载。
- [x] TASK-GAME-022 (PRD-GAME-007) [test_tier_required]: 为 `#46 PostOnboarding` 补齐无 UI live-protocol smoke、测试手册入口与协议证据回写，明确 headed Web/UI 与 no-UI 验证边界。
- [x] TASK-GAME-023 (PRD-GAME-008) [test_tier_required]: 新增“纯 API 客户端等价”专题 PRD / design / project，并完成 `game` 根 PRD、顶层设计主文档、索引与 devlog 挂载。
- [x] TASK-GAME-024 (PRD-GAME-001/008) [test_tier_required]: 收口 `game` 根 PRD / project 中当前真值 `cargo -p` 命令与纯 API 客户端源码路径的 `oasis7` 品牌。
  - 产物文件:
    - `doc/game/prd.md`
    - `doc/game/project.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "cargo test -p oasis7|cargo test -p oasis7_viewer|crates/oasis7/src/bin/oasis7_pure_api_client.rs" doc/game/prd.md doc/game/project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-GAME-025 (PRD-GAME-001/004/006) [test_tier_required]: 收口 `gameplay` 专题中当前真值实现锚点与 `cargo -p` 命令的 `oasis7` 品牌。
  - 产物文件:
    - `doc/game/gameplay/gameplay-war-politics-mvp-baseline.md`
    - `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
    - `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.project.md`
    - `doc/game/project.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "cargo test -p oasis7|cargo test -p oasis7_viewer|crates/oasis7|crates/oasis7_builtin_wasm_modules" doc/game/gameplay/gameplay-war-politics-mvp-baseline.md doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-GAME-026 (PRD-GAME-001/004/005/007/008) [test_tier_required]: 收口其余活跃 `gameplay` 专题中的当前源码锚点与 `cargo -p` 命令，统一使用 `oasis7` / `oasis7_viewer` / `oasis7_proto` 口径。
  - 产物文件:
    - `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`
    - `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.prd.md`
    - `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.project.md`
    - `doc/game/gameplay/gameplay-release-production-closure.project.md`
    - `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md`
    - `doc/game/gameplay/gameplay-top-level-design.project.md`
    - `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "cargo test -p oasis7|crates/oasis7|crates/oasis7_viewer|crates/oasis7_proto" doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.prd.md doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.project.md doc/game/gameplay/gameplay-release-production-closure.project.md doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md doc/game/gameplay/gameplay-top-level-design.project.md doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-GAME-027 (PRD-GAME-001/002/005) [test_tier_required]: 收口更早期 `gameplay` 活跃专题中遗漏的 runtime crate 路径与 `cargo -p` 命令，统一到 `oasis7` / `crates/oasis7*`。
  - 产物文件:
    - `doc/game/gameplay/gameplay-base-runtime-wasm-layer-split.prd.md`
    - `doc/game/gameplay/gameplay-base-runtime-wasm-layer-split.project.md`
    - `doc/game/gameplay/gameplay-runtime-governance-closure.project.md`
    - `doc/game/gameplay/gameplay-beta-balance-hardening-2026-02-22.project.md`
    - `doc/game/project.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "cargo test -p oasis7|cargo check -p oasis7|crates/oasis7/src/runtime" doc/game/gameplay/gameplay-base-runtime-wasm-layer-split.prd.md doc/game/gameplay/gameplay-base-runtime-wasm-layer-split.project.md doc/game/gameplay/gameplay-runtime-governance-closure.project.md doc/game/gameplay/gameplay-beta-balance-hardening-2026-02-22.project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-GAME-028 (PRD-GAME-009) [test_tier_required]: 新增“封闭 Beta 准入门禁”专题 PRD / design / project，并完成 `game` 根 PRD、`gameplay-top-level-design` 主文档、索引、handoff 与 devlog 挂载。
  - 产物文件:
    - `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.prd.md`
    - `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.design.md`
    - `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.project.md`
    - `doc/game/prd.md`
    - `doc/game/project.md`
    - `doc/game/prd.index.md`
    - `doc/game/README.md`
    - `doc/game/gameplay/gameplay-top-level-design.project.md`
    - `doc/game/gameplay/producer-to-runtime-task-game-029-closed-beta-runtime-evidence-2026-03-21.md`
    - `doc/game/gameplay/producer-to-viewer-task-game-030-closed-beta-first-screen-2026-03-21.md`
    - `doc/game/gameplay/producer-to-qa-task-game-031-closed-beta-unified-gate-2026-03-21.md`
    - `doc/game/gameplay/producer-to-liveops-task-game-032-closed-beta-candidate-runbook-2026-03-21.md`
    - `doc/devlog/2026-03-21.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "PRD-GAME-009|internal_playable_alpha_late|closed_beta_candidate" doc/game/prd.md doc/game/project.md doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.prd.md doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [ ] TASK-GAME-029 (PRD-GAME-009) [test_tier_required + test_tier_full]: `runtime_engineer` 收口 five-node no-LLM soak、replay/rollback drill 与 longrun release gate 的候选版本证据，形成封闭 Beta 准入所需的 runtime 最小硬证据包。
- [ ] TASK-GAME-030 (PRD-GAME-009) [test_tier_required]: `viewer_engineer` 收口 `PostOnboarding` 首屏降噪、主目标优先级与玩家入口 full-coverage gate 的最小产品化包，确保核心首屏不再偏“工程工具”。
- [ ] TASK-GAME-031 (PRD-GAME-009) [test_tier_required + test_tier_full]: `qa_engineer` 建立统一 `closed_beta_candidate` release gate，串联 headed Web/UI、pure API、no-UI smoke、longrun/recovery 与 trend baseline，给出升阶或阻断结论。
- [x] TASK-GAME-032 (PRD-GAME-009) [test_tier_required]: `liveops_community` 收口封闭 Beta 候选 runbook、招募/反馈/事故回流模板与禁语清单；在 `producer_system_designer` 放行前继续保持 `technical preview` 口径。
  - 产物文件:
    - `doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.prd.md`
    - `doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.design.md`
    - `doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.project.md`
    - `doc/playability_test_result/templates/closed-beta-candidate-incident-templates-2026-03-22.md`
    - `doc/playability_test_result/templates/closed-beta-candidate-feedback-log-guide-2026-03-22.md`
    - `doc/readme/prd.md`
    - `doc/readme/project.md`
    - `doc/readme/prd.index.md`
    - `doc/readme/README.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "closed beta candidate|technical preview|incident template|candidate-feedback" doc/readme/governance/readme-closed-beta-candidate-runbook-2026-03-22.prd.md doc/playability_test_result/templates/closed-beta-candidate-incident-templates-2026-03-22.md doc/playability_test_result/templates/closed-beta-candidate-feedback-log-guide-2026-03-22.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [ ] TASK-GAME-033 (PRD-GAME-009) [test_tier_required]: `producer_system_designer` 基于 `TASK-GAME-029/030/031/032` 的统一证据执行阶段评审，决定继续保持 `internal_playable_alpha_late` 还是升级为 `closed_beta_candidate`。

## 依赖
- 模块设计总览：`doc/game/design.md`
- doc/game/prd.index.md
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-engineering-architecture.md`
- `doc/playability_test_result/prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-22
- 当前状态: in_progress
- 下一任务: `TASK-GAME-029`
- 最新完成: `TASK-GAME-032`（`liveops_community` 已交付 closed beta candidate runbook、feedback/incident 模板与 `technical preview` 禁语边界，并在 `readme` 模块正式追踪。）
- 最新完成: `TASK-GAME-028`（已冻结当前阶段为 `internal_playable_alpha_late`，新增 `PRD-GAME-009` 封闭 Beta 准入专题，并把下一阶段目标正式拆为 runtime / viewer / QA / liveops 四条线。）
- 最新完成: `TASK-GAME-027`（已完成更早期 `gameplay` 活跃专题中遗漏的 runtime crate 路径与 `cargo -p` 命令收口，统一切到 `oasis7` / `crates/oasis7*`。）
- 最新完成: `TASK-GAME-026`（已完成其余活跃 `gameplay` 专题中当前源码锚点与 `cargo -p` 命令的 `oasis7` / `oasis7_viewer` / `oasis7_proto` 收口，未改动历史证据与 release gate 记录。）
- 最新完成: `TASK-GAME-025`（已将 gameplay 专题中仍作为当前真值使用的实现锚点与 `cargo -p` 命令统一切换到 `oasis7` / `oasis7_viewer` / `crates/oasis7*` 口径）。
- 最新完成: `TASK-GAME-024`（已将根 PRD / project 的当前真值命令与 pure API 客户端源码路径统一切换到 `oasis7` 口径）。
- 最新完成: `TASK-GAMEPLAY-API-004`（pure API required/full 验收收口，结论升级为 `parity_verified`）。
- 最新完成: `TASK-GAMEPLAY-API-003`（`oasis7_pure_api_client` 纯 API 正式玩家动作面交付）。
- 最新完成: `TASK-GAMEPLAY-API-002`（live 协议 `WorldSnapshot.player_gameplay` canonical 玩家语义下沉）。
- 最新完成: `TASK-GAME-023`（纯 API 客户端等价专题 PRD / design / project 与根入口挂载）。
- 最新完成: `TASK-GAME-022`（`#46 PostOnboarding` 无 UI live-protocol smoke、证据与手册入口已补齐）。
- 最新完成: `TASK-GAME-021`（`PostOnboarding` 阶段目标链 PRD / design / project 与根入口挂载）。
- 最新完成: `TASK-GAME-020`（前期工业引导闭环的 runtime / viewer / QA 落地与 required-tier 证据链）。
- 最新完成: `TASK-GAME-019`（game 模块 README / PRD 索引入口同步）。
- 阶段收口优先级: `P0`
- 阶段 owner: `producer_system_designer`
- 当前阶段判断: `internal_playable_alpha_late`
- 下一阶段目标: `closed_beta_candidate`
- 阻断条件: 若统一 `closed_beta_candidate` release gate 尚未建立或任一关键 lane `block`，当前项目仍不得升级为 `closed beta` 对外口径。
- 承接约束: `TASK-GAME-029/030/031/032` 必须以同一候选版本互链，不能用不同批次专题 `pass` 拼凑升阶结论。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: gameplay 子簇主从化完成，`TASK-GAMEPLAY-MLF-001/002/003/004` 与 `TASK-GAME-007` 已闭环；分布式长期在线专题已完成设计建档（`TASK-GAME-008`）与执行共识首个实现切片（`TASK-GAME-009`）。
- ROUND-003 进展: `TASK-GAME-010` 已完成，治理 `Queued + timelock/epoch` 门禁与紧急控制（刹车/否决）状态机已落地并通过定向回归。
- ROUND-004 进展: `TASK-GAME-011` 已完成，`TASK-GAME-DCG-007/008`（身份权重快照 + 女巫惩罚申诉闭环）已落地并通过治理/协议/持久化/审计回归。
- ROUND-005 进展: `TASK-GAME-DCG-009` 已完成，P2P 长稳脚本新增共识哈希一致性门禁并通过 triad+chaos 烟测。
- ROUND-006 进展: `TASK-GAME-DCG-010` 已完成，发布门禁报告与回滚预案已输出（含 `soak_release` 基线证据）。
- ROUND-007 进展: `TASK-GAME-002` 已完成，根 PRD 新增“新手/经济/战争”三循环验收矩阵（含 Given/When/Then、规则边界、证据事件、`test_tier_required` 入口与失败处置）。
- ROUND-008 进展: `TASK-GAME-003` 已完成，根 PRD 新增 `P0~P3` 分级标准、`opened -> triaged -> fixing -> verified -> closed/deferred` 闭环模板与强制约束（含豁免门禁）。
- ROUND-009 进展: `TASK-GAME-004` 已完成，根 PRD 新增 `D/RC/D-1/D0` 发布门禁节奏与证据包字段，形成可直接执行的 go/no-go 流程。
- ROUND-010 进展: `TASK-GAME-012` 已完成，新增 `PRD-GAME-006` 长期在线 P0 生产硬化专题（状态权威分层、回放回滚、反作弊、经济闭环、可运维性）并挂载到文档树。
- ROUND-011 进展: `TASK-GAME-013` 已完成，runtime 新增 `authority_source/submission_role` 共识提交分层与冲突拒绝审计事件，完成状态权威分层首个实现切片。
- ROUND-012 进展: `TASK-GAME-014` 已完成，新增 `first_tick_consensus_drift` 漂移定位与 `rollback_to_snapshot_with_reconciliation` 回滚后自动对账能力，并补齐 runbook + 演练门禁。
- ROUND-013 进展: `TASK-GAME-015` 已完成，治理惩罚记录新增 `detection_* + evidence_chain_hash` 证据链字段，惩罚事件指纹防重放与误伤率监控快照能力已落地并通过定向回归。
- ROUND-014 进展: `TASK-GAME-016` 已完成，main token 新增经济源汇审计报表与通胀/套利阈值 gate（`main_token_economy_audit_report/enforce_main_token_economy_gate`），并通过定向回归。
- ROUND-015 进展: `TASK-GAME-017` 已完成，新增 long-run 可运维发布门禁模型（SLO/告警/灰度/灾备 + 经济告警联动）与阻断接口 `enforce_longrun_operability_release_gate`。
- ROUND-016 进展: `TASK-GAME-018` 已立项，进入 viewer 体验层改造与手动截图验收阶段。
- ROUND-017 进展: `TASK-GAME-018` 已进入执行中，`TASK-GAMEPLAY-MLF-005/006/007` 已完成（控制结果显著条 + 玩家模式默认减负 + 世界可读性增强首轮实现）；`TASK-GAMEPLAY-MLF-008` 已完成 runtime_live 节奏修正与 ROUND-009 viewer 侧视觉证据采集（baseline / 3D / 2D / console / state / 录屏），并已移交 `qa_engineer`。
- ROUND-018 进展: `qa_engineer` 已基于 `doc/game/gameplay/gameplay-micro-loop-visual-closure-evidence-2026-03-10-round009.md` 完成复核并回写 `doc/playability_test_result/card_2026_03_10_23_27_43.md`；`TASK-GAME-018` 已完成。
- ROUND-019 进展: `producer_system_designer` 已在 `doc/game/gameplay/gameplay-top-level-design.prd.md` 冻结前期工业引导闭环，把“首个制成品 -> 工厂”设为新手前期主成就链，并在 `doc/game/gameplay/gameplay-top-level-design.project.md` 拆出 `runtime_engineer / viewer_engineer / qa_engineer` 后续执行项。
- ROUND-020 进展: `runtime_engineer` 已补齐工厂 `blocked/resumed/completed` 状态与审计事件，`viewer_engineer` 已把工业运行态和玩家友好反馈接入主界面，`qa_engineer` 已新增 `doc/playability_test_result/topics/industrial-onboarding-required-tier-cards-2026-03-15.md` 并把 required-tier 手动回归链路挂入 `testing-manual.md`；`TASK-GAME-020` 收口完成。
- ROUND-021 进展: `producer_system_designer` 已新增 `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.{prd,design,project}.md`，将 `FirstSessionLoop -> PostOnboarding -> MidLoop` 的阶段承接口径正式挂入 `game` 根 PRD、`gameplay-top-level-design` 主文档、索引与 devlog，用于承接 `#46` 的后续实现与验收。
- ROUND-022 进展: `viewer_engineer` 已完成 `TASK-GAMEPLAY-POD-002/003` 的首个实现切片：Viewer 在 `4/4` 后切入 `PostOnboarding` 阶段卡，基于工业事件与控制反馈输出默认目标、阻塞解释、下一步建议与分支解锁；当前只剩 `qa_engineer` 的 `TASK-GAMEPLAY-POD-004` Web / playability 证据待补。
- ROUND-023 进展: `qa_engineer` 已通过 `scripts/viewer-post-onboarding-qa.sh --bundle-dir output/release/game-launcher-local --no-llm` 在 fresh bundle Web 会话中完成 `#46` 验证，证据包位于 `output/playwright/playability/post-onboarding-20260319-094056/`，正式卡片为 `doc/playability_test_result/card_2026_03_19_09_40_56.md`；人工复核确认 `4/4` 后 Mission HUD 切换到 `PostOnboarding`、首局总结显示进入下一阶段，`TASK-GAMEPLAY-POD-004` 已完成。
- ROUND-024 进展: `qa_engineer` 已新增 `scripts/viewer-post-onboarding-headless-smoke.sh`，并以 `./scripts/viewer-post-onboarding-headless-smoke.sh --bundle-dir output/release/game-launcher-local --no-llm --viewer-port 4273 --web-bind 127.0.0.1:5111 --live-bind 127.0.0.1:5123 --chain-status-bind 127.0.0.1:5231` 在 fresh bundle 无 UI 路径完成 live-protocol smoke；证据包位于 `output/playwright/playability/post-onboarding-headless-20260319-101444/`，补充确认 `#46` 的同会话控制推进、快照时间推进与 `RuntimeEvent` feed 不依赖浏览器 UI，但屏幕语义仍以 ROUND-023 的 headed Web 证据为准。
- ROUND-025 进展: `producer_system_designer` 已新增 `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.{prd,design,project}.md`，将“纯 API 客户端在信息粒度、动作能力和持续游玩上与 UI 等价”正式挂入 `game` 根 PRD、`gameplay-top-level-design` 主文档、索引与 devlog；下一步转入 `TASK-GAMEPLAY-API-002`，由 `viewer_engineer / runtime_engineer` 对齐协议级 canonical 玩家语义。
- ROUND-026 进展: `viewer_engineer` / `runtime_engineer` 已完成 `TASK-GAMEPLAY-API-002` 首个实现切片：live `WorldSnapshot` 新增 `player_gameplay` canonical 玩家快照，向纯 API 客户端直接暴露 `FirstSessionLoop -> PostOnboarding` 的阶段、目标、进度、阻塞、下一步建议、可执行动作和最近控制反馈，并通过 `cargo check`、定向单测与 wasm viewer 编译验证；下一步转入 `TASK-GAMEPLAY-API-003`，补齐纯 API 正式玩家动作面的持续游玩闭环。
- ROUND-027 进展: `runtime_engineer` / `agent_engineer` / `viewer_engineer` 已完成 `TASK-GAMEPLAY-API-003` 首个实现切片：新增 `crates/oasis7/src/bin/oasis7_pure_api_client.rs` 作为参考纯 API 客户端，直接复用 live TCP 协议与 Ed25519 玩家签名链路，支持 `snapshot / step / play / pause / agent_chat / prompt_control / reconnect_sync / rotate_session / revoke_session`，并在 fresh no-LLM live 会话上跑通 `snapshot / step / reconnect_sync`；下一步转入 `TASK-GAMEPLAY-API-004`，补 parity matrix 与 required/full 纯 API 长玩验收。
- ROUND-028 进展: `qa_engineer` 已新增 `scripts/oasis7-pure-api-parity-smoke.sh`，并完成 source required、fresh bundle required 与 bundle full 的纯 API 回归，证据位于 `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md` 与 `doc/playability_test_result/card_2026_03_19_13_25_04.md`；初始结论是 pure API 已达到 `playable`，但仍未到 `parity_verified`。
- ROUND-029 进展: `runtime_engineer` / `viewer_engineer` 已补齐 `TASK-GAMEPLAY-API-004` 的两个收口切片：`oasis7_pure_api_client reconnect-sync --with-snapshot` 现在会直接恢复 `player_gameplay`，Viewer Mission HUD / PostOnboarding 主卡也已优先消费 canonical `snapshot.player_gameplay`；当前剩余阻断只剩 pure API no-LLM required/full 仍未证明能到达首个持续能力里程碑，因此 `TASK-GAMEPLAY-API-004` 继续保持 in_progress。
- ROUND-030 进展: `runtime_engineer` 已修复 pure API fresh no-LLM 路径中 `runtime_snapshot` 数值键 map 的反序列化阻断，`qa_engineer` 随后通过 `./scripts/oasis7-pure-api-parity-smoke.sh --tier required --no-llm --viewer-port 4297 --web-bind 127.0.0.1:5161 --live-bind 127.0.0.1:5163 --chain-status-bind 127.0.0.1:5249` 与 `./scripts/oasis7-pure-api-parity-smoke.sh --tier full --no-llm --viewer-port 4299 --web-bind 127.0.0.1:5171 --live-bind 127.0.0.1:5173 --chain-status-bind 127.0.0.1:5251` 完成 source required/full 收口复验；pure API 现已能以正式 `gameplay_action` 路径到达 `post_onboarding.choose_midloop_path`，`TASK-GAMEPLAY-API-004` 已完成，专题结论升级为 `parity_verified`。
- ROUND-031 进展: `producer_system_designer` 已完成 `TASK-GAME-024`，将 `game` 根 PRD / project 中仍作为当前真值使用的 `cargo -p oasis7*` 命令与 pure API 客户端源码路径统一收口到 `oasis7` / `oasis7_viewer` / `crates/oasis7/src/bin/oasis7_pure_api_client.rs`，确保 `game` 模块入口文档不再把旧品牌当作默认口径。
- ROUND-032 进展: `producer_system_designer` 已完成 `TASK-GAME-025`，将 `gameplay` 专题里仍作为当前真值使用的 runtime 源码锚点、builtin wasm 模块路径与 `cargo -p oasis7*` 命令统一收口到 `oasis7` / `oasis7_viewer` / `crates/oasis7*`，且保留历史证据、手动量化记录与 devlog 引用不变。
- ROUND-033 进展: `producer_system_designer` 已新增 `doc/game/gameplay/gameplay-closed-beta-readiness-2026-03-21.{prd,design,project}.md`，正式冻结当前阶段为 `internal_playable_alpha_late`，并把冲 `closed_beta_candidate` 的工作拆为 `TASK-GAME-029/030/031/032/033`；对应 handoff 已分别发给 `runtime_engineer`、`viewer_engineer`、`qa_engineer` 与 `liveops_community`。
- ROUND-034 进展: `liveops_community` 已完成 `TASK-GAME-032`，新增 closed beta candidate runbook、feedback/incident 模板与 FAQ/禁语边界，明确在 `producer_system_designer` 放行前继续维持 `technical preview / not playable yet` 口径。
- ROUND-035 进展: `runtime_engineer` 已为 `scripts/s10-five-node-game-soak.sh` 增加 `exit-status.txt` 级别的失败签名记录，并确认 `TASK-GAME-029` 当前主阻断更接近“默认端口环境污染/外部 SIGTERM”而非 runtime 明确 panic；`--base-port 5910` 的 60 秒诊断样本已通过，但正式 `closed_beta_candidate` 所需的 600 秒候选 soak + replay/rollback 仍未完成，因此阶段继续保持 `internal_playable_alpha_late`。
- 说明: 本文档仅维护 game 设计执行状态；过程记录在 `doc/devlog/2026-03-05.md`、`doc/devlog/2026-03-06.md`、`doc/devlog/2026-03-07.md`、`doc/devlog/2026-03-15.md` 与 `doc/devlog/2026-03-18.md`。
  - 最新过程记录补充见 `doc/devlog/2026-03-21.md`。

## 阶段收口角色交接
### Meta
- Handoff ID: `HO-CORE-20260310-GAME-001`
- Date: `2026-03-10`
- From Role: `producer_system_designer`
- To Role: `viewer_engineer`
- Related Module: `game`
- Related PRD-ID: `PRD-GAME-004`
- Related Task ID: `TASK-GAME-018`
- Priority: `P0`
- Expected ETA: `待接收方确认`

### Objective
- 目标描述：完成微循环可玩性视觉优化二期，并把可直接进入发布评审的截图闭环证据沉淀到跨模块证据链。
- 成功标准：控制结果显著化、玩家模式减负、世界可读性增强三项已完成，且 `qa_engineer` 已基于证据完成复核。
- 非目标：不在本轮新增 launcher / explorer 体验功能，不扩展与微循环无关的 Viewer 大改。

### Current State
- 当前实现 / 文档状态：`TASK-GAME-018` 已完成，`TASK-GAMEPLAY-MLF-005/006/007/008` 均已闭环，当前待做的是把已完成结论回填到 release gate 证据链。
- 已确认事实：core 阶段收口将玩法微循环列为 `P0`；虽然任务已关闭，但若缺少跨模块证据互链，仍不得给出最终发布 `go` 结论。
- 待确认假设：现有 ROUND-009 录屏是否足以覆盖发布评审抽样；若不足，则在 release gate 阶段补拍，不回滚当前任务关闭结论。
- 当前失败信号 / 用户反馈：当前项目仍偏“能展示”，需要把“更好玩”变成明确证据。

### Scope
- In Scope: `TASK-GAME-018`、`TASK-GAMEPLAY-MLF-005/006/007/008`、截图 / 视频 / 结论证据回写。
- Out of Scope: 新玩法分支、新区块链浏览器功能、与微循环无关的全局 UI 重构。

### Inputs
- 关键文件：`doc/game/project.md`、`doc/game/prd.md`、相关 `gameplay-micro-loop-*` 专题文档。
- 关键命令：沿用现有 Viewer / playability 截图闭环命令与手动验收流程。
- 上游依赖：`producer_system_designer` 已在 `core` 层确定该项为 `P0`；`qa_engineer` 后续复核证据。
- 现有测试 / 证据：现有手动截图验收记录与 `runtime_live` 节奏修正结果。

### Requested Work
- 工作项 1：由 `qa_engineer` 复核 `doc/game/gameplay/gameplay-micro-loop-visual-closure-evidence-2026-03-10-round009.md` 的截图、录屏与语义状态。
- 工作项 2：刷新 playability 卡片与 `TASK-GAME-018` 阻断结论。
- 工作项 3：若结论通过，把 evidence linkage 回填到 playability / testing / core 证据链。

### Expected Outputs
- 代码改动：如需，仅限支撑 `TASK-GAME-018` 的 Viewer 表达层改动。
- 文档回写：`doc/game/project.md`、必要时相关专题 `project/prd`。
- 测试记录：至少补齐 `test_tier_required` 的截图闭环与结论。
- devlog 记录：在 `doc/devlog/YYYY-MM-DD.md` 中记载结果与遗留项。

### Done Definition
- [ ] 输出满足目标与成功标准
- [ ] 影响面已核对 `producer_system_designer` / `qa_engineer`
- [ ] 对应 `prd.md` / `project.md` 已回写
- [ ] 对应 `doc/devlog/YYYY-MM-DD.md` 已记录
- [ ] required 证据已补齐

### Risks / Decisions
- 已知风险：如果只做视觉 polish 而不统一证据格式，玩法收口仍无法进入 go/no-go 评审。
- 待拍板事项：是否需要把 `TASK-GAMEPLAY-MLF-007` 进一步拆小给 `viewer_engineer`。
- 建议决策：先以最小体验闭环完成 `TASK-GAME-018`，不引入额外玩法范围扩张。

### Validation Plan
- 测试层级：`test_tier_required`
- 验证命令：沿用现有截图闭环与手动验收命令，并回写证据路径。
- 预期结果：微循环视觉增强可被截图 / 视频直接观察到，且 QA 可复核。
- 回归影响范围：game / viewer / playability 体验层。

### Handoff Acknowledgement
- 接收方确认范围：`qa_engineer 已接收 ROUND-009 证据并完成复核；TASK-GAME-018 已具备任务关闭结论`
- 接收方确认 ETA：`TASK-GAME-018 已完成；下一步转入 evidence linkage 回填`
- 接收方新增风险：`更长录屏仍建议在后续 release gate 抽样中复看，但不构成当前任务阻断`
