# testing PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-TESTING-001 (PRD-TESTING-001) [test_tier_required]: 完成 testing PRD 改写，建立分层测试设计入口。
- [x] TASK-TESTING-002 (PRD-TESTING-001/002) [test_tier_required]: 对齐 S0~S10 与改动路径触发矩阵。
  - 产物文件:
    - `testing-manual.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "套件触发总表|改动路径矩阵|选择规则|S0|S10" testing-manual.md`
- [x] TASK-TESTING-003 (PRD-TESTING-002/003) [test_tier_required]: 建立发布证据包模板（命令、日志、截图、结论）。
  - 产物文件:
    - `doc/testing/templates/release-evidence-bundle-template.md`
  - 验收命令 (`test_tier_required`):
    - `test -f doc/testing/templates/release-evidence-bundle-template.md`
    - `rg -n "执行命令|UI / 体验证据|长跑 / 在线证据|结论摘要|PRD-ID" doc/testing/templates/release-evidence-bundle-template.md`
- [ ] TASK-TESTING-004 (PRD-TESTING-003) [test_tier_required]: 建立测试质量趋势跟踪（通过率/逃逸率/修复时长）。
- [x] TASK-TESTING-005 (PRD-TESTING-002/003) [test_tier_required]: 建立模块级专题任务映射索引（2026-03-02 批次）。
- [x] TASK-TESTING-006 (PRD-TESTING-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-TESTING-007 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-wasm32-target-install` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-008 (PRD-TESTING-004) [test_tier_required]: 继续按批次迁移 testing 活跃 legacy 专题文档（优先 `governance/launcher/longrun/performance/manual`）。
- [x] TASK-TESTING-009 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-testcase-tiering` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-010 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-tiered-execution` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-011 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-test-coverage` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-012 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-builtin-wasm-m1-multi-runner` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-013 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-m1-multi-runner-required-check-protection` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-014 (PRD-TESTING-004) [test_tier_required]: 完成 `ci-remove-builtin-wasm-hash-checks-from-base-gate` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-015 (PRD-TESTING-004) [test_tier_required]: 完成 `wasm-build-determinism-guard` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-016 (PRD-TESTING-004) [test_tier_required]: 完成 `release-gate-metric-policy-alignment-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-017 (PRD-TESTING-004) [test_tier_required]: 完成 `llm-skip-tick-ratio-metric` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-018 (PRD-TESTING-004) [test_tier_required]: 完成 `launcher-chain-script-migration-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-019 (PRD-TESTING-004) [test_tier_required]: 完成 `launcher-lifecycle-hardening-2026-03-01` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-020 (PRD-TESTING-004) [test_tier_required]: 完成 `launcher-viewer-auth-node-config-autowire-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-021 (PRD-TESTING-004) [test_tier_required]: 完成 `chain-runtime-feedback-replication-network-autowire-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-022 (PRD-TESTING-004) [test_tier_required]: 完成 `chain-runtime-soak-script-reactivation-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-023 (PRD-TESTING-004) [test_tier_required]: 完成 `p2p-longrun-continuous-chaos-injection-2026-02-24` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-024 (PRD-TESTING-004) [test_tier_required]: 完成 `p2p-longrun-endurance-chaos-template-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-025 (PRD-TESTING-004) [test_tier_required]: 完成 `p2p-storage-consensus-longrun-online-stability-2026-02-24` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-026 (PRD-TESTING-004) [test_tier_required]: 完成 `p2p-longrun-feedback-event-injection-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-027 (PRD-TESTING-004) [test_tier_required]: 完成 `s10-distfs-probe-bootstrap-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-028 (PRD-TESTING-004) [test_tier_required]: 完成 `s10-five-node-real-game-soak` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-029 (PRD-TESTING-004) [test_tier_required]: 完成 `runtime-performance-observability-foundation-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-030 (PRD-TESTING-004) [test_tier_required]: 完成 `runtime-performance-observability-llm-api-decoupling-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-031 (PRD-TESTING-004) [test_tier_required]: 完成 `viewer-perf-bottleneck-observability-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-032 (PRD-TESTING-004) [test_tier_required]: 完成 `viewer-performance-methodology-closure-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-033 (PRD-TESTING-004) [test_tier_required]: 完成 `systematic-application-testing-manual` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-034 (PRD-TESTING-004) [test_tier_required]: 完成 `web-ui-agent-browser-closure-manual` 专题文档逐篇人工迁移到 strict schema，并补齐 `.project.md` 管理文档。
- [x] TASK-TESTING-042 (PRD-TESTING-002/003/004) [test_tier_required]: 将 Web UI 闭环默认工具口径统一收口到 `agent-browser`，同步主手册、专题分册、Viewer 手册、站内镜像与门禁脚本。
- [x] TASK-TESTING-035 (PRD-TESTING-004) [test_tier_required]: 完成 archive 专题 `ci-required-m1-wasm-hash-check` 文档迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-036 (PRD-TESTING-004) [test_tier_required]: 完成 archive 专题 `wasm-platform-canonical-hash-manifest` 文档迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-037 (PRD-TESTING-005) [test_tier_required]: 完成 `ci-builtin-wasm-m4-m5-hash-drift-hardening` 专题 PRD 与项目管理文档建档，建立 1-6 治理项映射。
- [x] TASK-TESTING-038 (PRD-TESTING-005) [test_tier_required]: 落地 m4/m5 keyed hash manifest 迁移与 sync strict 模式（禁 legacy 写回）。
- [x] TASK-TESTING-039 (PRD-TESTING-005) [test_tier_required]: 收敛 builtin wasm identity 的 `source_hash` 输入范围并移除 workspace 根 `Cargo.lock` 依赖。
- [x] TASK-TESTING-040 (PRD-TESTING-005) [test_tier_required]: 增加 m4/m5 多 runner 对账 workflow、required checks 保护与本地只读校验策略。
- [x] TASK-TESTING-041 (PRD-TESTING-002/003) [test_tier_required]: 完成“启动器全功能可用性审查与闭环验收（2026-03-08）”专题执行（脚本审查 + agent-browser 真闭环 + 风险分级结论）。

## 专题任务映射（2026-03-02 批次）
- [x] SUBTASK-TESTING-20260302-001 (PRD-TESTING-002/003) [test_tier_required]: `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260302-002 (PRD-TESTING-002/003) [test_tier_required]: `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260302-003 (PRD-TESTING-002/003) [test_tier_required]: `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.project.md`

## 专题任务映射（2026-03-03 批次）
- [x] SUBTASK-TESTING-20260303-001 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-wasm32-target-install.project.md`
- [x] SUBTASK-TESTING-20260303-002 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-testcase-tiering.project.md`
- [x] SUBTASK-TESTING-20260303-003 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-tiered-execution.project.md`
- [x] SUBTASK-TESTING-20260303-004 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-test-coverage.project.md`
- [x] SUBTASK-TESTING-20260303-005 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.project.md`
- [x] SUBTASK-TESTING-20260303-006 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-m1-multi-runner-required-check-protection.project.md`
- [x] SUBTASK-TESTING-20260303-007 (PRD-TESTING-004) [test_tier_required]: `doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.project.md`
- [x] SUBTASK-TESTING-20260303-008 (PRD-TESTING-004) [test_tier_required]: `doc/testing/governance/wasm-build-determinism-guard.project.md`
- [x] SUBTASK-TESTING-20260303-009 (PRD-TESTING-004) [test_tier_required]: `doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.project.md`
- [x] SUBTASK-TESTING-20260303-010 (PRD-TESTING-004) [test_tier_required]: `doc/testing/governance/llm-skip-tick-ratio-metric.project.md`
- [x] SUBTASK-TESTING-20260303-011 (PRD-TESTING-004) [test_tier_required]: `doc/testing/launcher/launcher-chain-script-migration-2026-02-28.project.md`
- [x] SUBTASK-TESTING-20260303-012 (PRD-TESTING-004) [test_tier_required]: `doc/testing/launcher/launcher-lifecycle-hardening-2026-03-01.project.md`
- [x] SUBTASK-TESTING-20260303-013 (PRD-TESTING-004) [test_tier_required]: `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260303-014 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260303-015 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.project.md`
- [x] SUBTASK-TESTING-20260303-016 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.project.md`
- [x] SUBTASK-TESTING-20260303-017 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.project.md`
- [x] SUBTASK-TESTING-20260303-018 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.project.md`
- [x] SUBTASK-TESTING-20260303-019 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260303-020 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.project.md`
- [x] SUBTASK-TESTING-20260303-021 (PRD-TESTING-004) [test_tier_required]: `doc/testing/longrun/s10-five-node-real-game-soak.project.md`
- [x] SUBTASK-TESTING-20260303-022 (PRD-TESTING-004) [test_tier_required]: `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.project.md`
- [x] SUBTASK-TESTING-20260303-023 (PRD-TESTING-004) [test_tier_required]: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.project.md`
- [x] SUBTASK-TESTING-20260303-024 (PRD-TESTING-004) [test_tier_required]: `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.project.md`
- [x] SUBTASK-TESTING-20260303-025 (PRD-TESTING-004) [test_tier_required]: `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.project.md`
- [x] SUBTASK-TESTING-20260303-026 (PRD-TESTING-004) [test_tier_required]: `doc/testing/manual/systematic-application-testing-manual.project.md`
- [x] SUBTASK-TESTING-20260303-027 (PRD-TESTING-004) [test_tier_required]: `doc/testing/manual/web-ui-agent-browser-closure-manual.project.md`

## 专题任务映射（2026-03-06 批次）
- [x] SUBTASK-TESTING-20260306-001 (PRD-TESTING-005) [test_tier_required]: `doc/testing/ci/ci-builtin-wasm-m4-m5-hash-drift-hardening.project.md`

## 专题任务映射（2026-03-08 批次）
- [x] SUBTASK-TESTING-20260308-001 (PRD-TESTING-002/003) [test_tier_required]: `doc/testing/launcher/launcher-full-usability-closure-audit-2026-03-08.project.md`

## 专题任务映射（2026-03-10 批次）
- [x] SUBTASK-TESTING-20260310-001 (PRD-TESTING-LAUNCHER-MANUAL-001/002/003) [test_tier_required]: `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.project.md`

## 依赖
- 模块设计总览：`doc/testing/design.md`
- doc/testing/prd.index.md
- `testing-manual.md`
- `doc/testing/manual/web-ui-agent-browser-closure-manual.prd.md`
- `scripts/ci-tests.sh`
- `.github/workflows/*`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-10
- 当前状态: active
- 下一任务: TASK-TESTING-004
- 阶段收口优先级: `P0`
- 阶段 owner: `qa_engineer`（联审：`producer_system_designer`）
- 阻断条件: 在 `TASK-TESTING-002/003` 完成前，跨模块发布评审不得声称“测试范围明确且证据齐备”。
- 承接约束: `TASK-TESTING-002` 完成后进入 `TASK-TESTING-003`；`TASK-TESTING-004` 作为趋势化建设保留在其后。
- 专题映射状态: 2026-03-02 批次 3/3 已纳入模块项目管理文档。
- 专题映射状态补充: 2026-03-06 批次 1/1 已纳入模块项目管理文档。
- 专题映射状态补充: 2026-03-08 批次 1/1 已完成（启动器全功能可用性审查）。
- 专题映射状态补充: 2026-03-10 批次 1/1 已完成（启动器人工测试清单建档）。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 testing 模块设计执行状态；过程记录在 `doc/devlog/2026-03-10.md`。

## 阶段收口角色交接
## Meta
- Handoff ID: `HO-CORE-20260310-TEST-001`
- Date: `2026-03-10`
- From Role: `producer_system_designer`
- To Role: `qa_engineer`
- Related Module: `testing`
- Related PRD-ID: `PRD-TESTING-001/002/003`
- Related Task ID: `TASK-TESTING-002/003`
- Priority: `P0`
- Expected ETA: `待接收方确认`

## Objective
- 目标描述：建立统一的测试触发矩阵与发布证据包模板，使发布评审不再依赖临时判断。
- 成功标准：任一任务都能反推必跑测试，证据包字段统一且可映射到 PRD-ID / 任务 / 结论。
- 非目标：本轮不要求先完成长期趋势统计。

## Current State
- 当前实现 / 文档状态：`TASK-TESTING-002/003/004` 仍未完成，但近期 launcher / viewer 闭环证据已有积累。
- 已确认事实：core 已将 testing 触发矩阵与证据包列为 `P0`。
- 待确认假设：S0~S10 触发矩阵是否需要对现有专题任务映射做进一步合并。
- 当前失败信号 / 用户反馈：测试可跑但“该跑什么、结果怎么看、能不能放”仍缺统一模板。

## Scope
- In Scope: `TASK-TESTING-002`、`TASK-TESTING-003`。
- Out of Scope: 本轮不优先做 `TASK-TESTING-004` 趋势统计面板。

## Inputs
- 关键文件：`doc/testing/project.md`、`doc/testing/prd.md`、`testing-manual.md`。
- 关键命令：`scripts/ci-tests.sh`、现有 viewer / launcher / playability 闭环命令。
- 上游依赖：各模块现有 `test_tier_required/full` 定义与证据产物。
- 现有测试 / 证据：`2026-03-08` / `2026-03-10` 的 launcher / viewer 闭环与人工清单结果。

## Requested Work
- 工作项 1：完成 S0~S10 与改动路径触发矩阵。
- 工作项 2：建立发布证据包模板（命令、日志、截图、结论）。
- 工作项 3：与 core PRD-ID 映射模板对齐引用方式。

## Expected Outputs
- 代码改动：如需，仅限测试脚本或模板支撑变更。
- 文档回写：`doc/testing/project.md`、相关 testing 分册。
- 测试记录：补齐 `test_tier_required` 的模板验证证据。
- devlog 记录：记录矩阵、模板和遗留趋势项。

## Done Definition
- [ ] 输出满足目标与成功标准
- [ ] 影响面已核对 `producer_system_designer` 与关键模块 owner
- [ ] 对应 `prd.md` / `project.md` 已回写
- [ ] 对应 `doc/devlog/YYYY-MM-DD.md` 已记录
- [ ] required 证据已补齐

## Risks / Decisions
- 已知风险：若先做趋势统计而不先统一触发矩阵与证据包，数据口径会继续漂移。
- 待拍板事项：证据包目录结构是否需要与现有 `output/` 产物强绑定。
- 建议决策：先完成 `002/003`，再推进 `004` 趋势统计。

## Validation Plan
- 测试层级：`test_tier_required`
- 验证命令：以 `rg` 抽样矩阵 / 模板字段，并结合现有闭环产物路径做引用验证。
- 预期结果：任一阶段收口任务都能映射到统一测试范围与证据包格式。
- 回归影响范围：全模块测试治理与发布评审流程。

## Handoff Acknowledgement
- 接收方确认范围：`已接收 TASK-TESTING-002/003；本轮覆盖触发矩阵与发布证据包模板，不含趋势统计`
- 接收方确认 ETA：`TASK-TESTING-002/003 已完成，下一步进入 TASK-TESTING-004`
- 接收方新增风险：`长跑 / UI 产物目录在不同脚本间仍有差异，当前模板先统一字段，不强制统一物理目录`
