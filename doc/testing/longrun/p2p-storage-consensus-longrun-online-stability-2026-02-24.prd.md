# Agent World：P2P/存储/共识在线长跑稳定性测试方案（2026-02-24）

- 对应设计文档: `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.design.md`
- 对应项目管理文档: `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: 短时功能测试无法覆盖持续在线运行下的稳定性退化，缺少可重复执行的长跑方案来验证 P2P、DistFS 与共识链路在长期运行中的健康度。
- Proposed Solution: 建立 S9 长跑方案与统一脚本入口，按 `soak_smoke/soak_endurance/soak_release` 分档执行，输出可审计产物并用门禁规则评估稳定性、恢复能力与不变量状态。
- Success Criteria:
  - SC-1: `scripts/p2p-longrun-soak.sh` 能稳定编排 `triad` 与 `triad_distributed` 长跑执行。
  - SC-2: 统一产出 `timeline.csv`、`summary.json`、`summary.md`、`chaos_events.log` 与失败报告。
  - SC-3: 门禁覆盖 stall、lag、distfs failure ratio、invariant 状态。
  - SC-4: `testing-manual.md` S9 完成触发矩阵、执行剧本与证据规范接线。
  - SC-5: 至少完成一组 `soak_smoke` 与一组 `soak_endurance` 收口实跑留档。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：需要标准化长跑脚本与分档门禁。
  - 发布负责人：需要可审计稳定性证据支撑放行决策。
  - 基础设施/内核开发者：需要在高风险改动后快速识别长稳回归。
- User Scenarios & Frequency:
  - 高风险改动回归：执行 `soak_smoke` 快速判断是否存在明显退化。
  - 发布前验收：执行 `soak_endurance/soak_release` 做长稳与恢复能力评估。
  - 夜间巡检：周期执行分布式长跑并累计趋势数据。
- User Stories:
  - PRD-TESTING-LONGRUN-S9SOAK-001: As a 测试维护者, I want repeatable longrun profiles for p2p/storage/consensus, so that stability regressions are detectable beyond short tests.
  - PRD-TESTING-LONGRUN-S9SOAK-002: As a 发布负责人, I want auditable summary artifacts and deterministic gate rules, so that release decisions are evidence-driven.
  - PRD-TESTING-LONGRUN-S9SOAK-003: As a 开发者, I want controlled chaos injection with recovery tracking, so that resilience under disturbance is validated.
- Critical User Flows:
  1. Flow-S9SOAK-001: `选择 soak profile -> 启动拓扑（triad/triad_distributed） -> 周期采样状态与报表`
  2. Flow-S9SOAK-002: `汇总 epoch 报表/进程指标/日志 -> 计算门禁 -> 输出 summary 与 failures`
  3. Flow-S9SOAK-003: `执行 chaos plan -> 记录 chaos_events 时间线 -> 判定恢复与追平表现`
  4. Flow-S9SOAK-004: `将结果回填 testing-manual S9 与 devlog -> 用于回归与发布复盘`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 长跑入口脚本 | `--profile --duration-secs --topology --scenario --enable-chaos --chaos-plan --out-dir --max-stall-secs --max-lag-p95 --max-distfs-failure-ratio` | 启动长跑、采样、收口并输出产物 | `planned -> running -> passed/failed` | profile 决定默认阈值，CLI 可覆盖 | 测试维护者可配置 |
| 分档策略 | `soak_smoke/soak_endurance/soak_release` | 按场景选择执行强度 | `smoke -> endurance -> release` | `soak_*` 仅表示长跑档位，不等同 `test_tier_*` | 发布/QA 审核档位选择 |
| 拓扑矩阵 | `triad`、`triad_distributed`、`triad_distributed+chaos` | 按 case 运行并采样 | `boot -> steady -> teardown` | 默认统一启用 reward runtime 相关能力 | 维护者可扩展 case |
| 门禁判定 | stall、lag p95、distfs failure ratio、invariant | 根据 summary 规则输出 pass/fail/insufficient_data | `collecting -> evaluating -> final` | 指标按窗口聚合，支持 profile 差异阈值 | 发布门禁消费 |
| 证据产物 | `run_config.json/timeline.csv/summary.json/summary.md/failures.md/chaos_events.log/nodes/*` | 归档并供审计/复盘 | `generated -> archived` | total 与明细一致，失败必须有 failures | QA/发布审阅 |
- Acceptance Criteria:
  - AC-1: S9 脚本支持最小闭环（启动、停止、超时、清理、产物目录）。
  - AC-2: 支持 epoch 报表聚合与门禁判定输出。
  - AC-3: 支持 `--chaos-plan` 注入并输出事件时间线。
  - AC-4: testing-manual S9 包含执行建议、触发矩阵与证据规范。
  - AC-5: 完成 smoke/endurance 样本运行并记录结论与边界语义。
- Non-Goals:
  - 不做跨物理机/跨地域压测编排。
  - 不引入新共识算法或新存储协议语义改造。
  - 不覆盖 UI/Web 交互验证（由 S6/S8 覆盖）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为运行稳定性与脚本门禁方案，不涉及 AI 推理链路改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `p2p-longrun-soak.sh` 作为长跑编排入口，复用 `world_viewer_live` 分布式拓扑与 reward runtime 报表能力，聚合共识/存储/网络/不变量指标并执行门禁判定。
- Integration Points:
  - `scripts/p2p-longrun-soak.sh`
  - `testing-manual.md`（S9）
  - `scripts/ci-tests.sh`
  - `scripts/viewer-owr4-stress.sh`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world_node/src/types.rs`
  - `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.project.md`
- Edge Cases & Error Handling:
  - 报表频率不一致：采用时间窗口容错，先确保关键字段可用。
  - 环境资源噪声：记录机器上下文并按 profile 区分阈值。
  - 长跑耗时高：通过分档执行控制成本（smoke 快速回归，endurance/release 发布前验证）。
  - chaos 与真实故障混淆：通过 `chaos_events.log` 时间线对注入窗口做关联判定。
  - 无 epoch 报表：标记 `metric_gate=insufficient_data`（`soak_smoke` 告警、`soak_endurance/soak_release` 失败）。
- Non-Functional Requirements:
  - NFR-S9SOAK-1: 长跑脚本具备可重复执行与可审计产物。
  - NFR-S9SOAK-2: 指标门禁在不同拓扑和档位下口径一致且可解释。
  - NFR-S9SOAK-3: 故障注入与恢复结果可在单次 run 内回溯。
  - NFR-S9SOAK-4: 手册、脚本与项目文档口径保持一致。
- Security & Privacy: 长跑日志需避免敏感信息泄露；故障注入仅限受控测试环境。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (S9SOAK-1): 方案建档与项目管理文档建档。
  - v1.1 (S9SOAK-2): 脚本最小闭环（启动/停止/超时/目录）。
  - v2.0 (S9SOAK-3): 指标聚合与门禁判定（summary/failures）。
  - v2.1 (S9SOAK-4): chaos 注入与恢复验证接入。
  - v2.2 (S9SOAK-5): testing-manual S9 接线与执行剧本。
  - v2.3 (S9SOAK-6): smoke/endurance 样本实跑收口。
  - v2.4 (S9SOAK-7): 按 strict schema 人工迁移并统一 `.prd` 命名。
- Technical Risks:
  - 风险-1: 指标口径随拓扑波动，误判概率上升。
  - 风险-2: 单机资源竞争导致抖动放大。
  - 风险-3: 注入窗口与真实异常时间线混叠。
  - 风险-4: 长跑耗时对开发节奏造成压力。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-S9SOAK-001 | S9SOAK-1/2/5 | `test_tier_required` | S9 脚本闭环、拓扑启动与手册触发矩阵核验 | 长跑执行体系可用性 |
| PRD-TESTING-LONGRUN-S9SOAK-002 | S9SOAK-2/3/6 | `test_tier_required` | summary/failures 门禁判定与样本 run 结果校验 | 发布证据与门禁一致性 |
| PRD-TESTING-LONGRUN-S9SOAK-003 | S9SOAK-3/4/6/7 | `test_tier_required` | chaos 注入时间线与恢复链路验证 + 文档治理检查 | 韧性验证与追溯一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-S9SOAK-001 | 引入 `soak_*` 分档策略 | 所有长跑统一单档执行 | 控制执行成本并覆盖不同验证场景。 |
| DEC-S9SOAK-002 | 复用 `world_viewer_live` 现有报表能力 | 新建独立报表链路 | 降低实现成本并保持口径连续性。 |
| DEC-S9SOAK-003 | chaos 注入与门禁判定联动，记录时间线 | 仅看最终状态不记录注入细节 | 提升故障复盘与审计可解释性。 |
| DEC-S9SOAK-004 | `insufficient_data` 在 smoke 告警、在 endurance/release 失败 | 所有档位统一失败 | 平衡快速回归效率与发布风险控制。 |

## 原文约束点映射（内容保真）
- 原“目标（可重复长跑 + 三链路覆盖 + 分档验收）” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“入口参数、分档策略、拓扑矩阵、证据源、通过标准、产物目录、手册接线” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M0~M5” -> 第 5 章 Phased Rollout（S9SOAK-1~7）。
- 原“风险（口径、噪声、耗时、误报）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
- 原“当前状态（smoke/endurance 命令与产物、insufficient_data 语义）” -> 第 6 章验证与判定策略。
