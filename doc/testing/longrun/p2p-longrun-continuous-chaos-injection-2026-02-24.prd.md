# Agent World：P2P 长跑持续 Chaos 注入方案（2026-02-24）

审计轮次: 4

## 1. Executive Summary
- Problem Statement: 现有 `--chaos-plan` 只支持固定注入序列，无法模拟线上长期抖动与故障组合，导致 S9 长跑对未知风险覆盖不足。
- Proposed Solution: 在 `scripts/p2p-longrun-soak.sh` 增加持续 chaos 注入能力，并支持与固定计划混合运行，输出完整注入时间线与统计证据以保持门禁可审计。
- Success Criteria:
  - SC-1: 支持持续注入开关、节奏、动作集合、seed 与事件上限配置。
  - SC-2: 固定计划与持续注入可共存并复用同一执行链路。
  - SC-3: `run_config.json`、`summary.json`、`summary.md` 包含 continuous chaos 配置与 plan/continuous/total 计数字段。
  - SC-4: `testing-manual.md` S9 提供持续注入示例与验收口径。
  - SC-5: 完成短窗混合注入实跑并保留可复现证据。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：需要在固定回归与随机探索之间切换。
  - 发布负责人：需要基于注入统计和门禁结果做放行判断。
  - 脚本维护者：需要可复现、可审计、可控强度的 chaos 注入机制。
- User Scenarios & Frequency:
  - 日常回归：固定计划为主，按需叠加少量持续注入。
  - 版本压测：开启持续注入评估系统长期鲁棒性。
  - 故障复盘：使用固定 seed 复跑定位问题。
- User Stories:
  - PRD-TESTING-LONGRUN-CHAOS-001: As a 测试维护者, I want continuous chaos injection on top of fixed plans, so that longrun can cover unknown failure combinations.
  - PRD-TESTING-LONGRUN-CHAOS-002: As a 发布负责人, I want auditable chaos evidence in run configs and summaries, so that gate decisions are defensible.
  - PRD-TESTING-LONGRUN-CHAOS-003: As a 脚本维护者, I want seed-based reproducibility and throttled injection cadence, so that chaos runs are controllable and repeatable.
- Critical User Flows:
  1. Flow-CHAOS-001: `解析 continuous chaos CLI -> 校验参数 -> 生成调度策略`
  2. Flow-CHAOS-002: `运行固定计划与持续注入混合循环 -> 串行执行事件 -> 记录成功/失败`
  3. Flow-CHAOS-003: `汇总 run_config/summary/markdown -> 输出 plan/continuous/total 计数 -> 更新手册与门禁结论`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 持续注入 CLI | `--chaos-continuous-enable`、`--chaos-continuous-interval-secs`、`--chaos-continuous-start-sec`、`--chaos-continuous-max-events`、`--chaos-continuous-actions`、`--chaos-continuous-seed`、`--chaos-continuous-restart-down-secs`、`--chaos-continuous-pause-duration-secs` | 启用后周期触发注入事件 | `configured -> scheduled -> executed` | 从 `start-sec` 起按 `interval-secs` 触发，`max-events=0` 表示直到 run 结束 | 脚本维护者可配置 |
| 混合注入执行 | 固定计划事件 + 持续注入事件 | 共用执行器串行处理，失败按现有 `chaos_failed` 收口 | `idle -> running -> failed/stopped` | 单次只执行一个事件，避免并发注入干扰 | QA/发布可审计 |
| 伪随机选择 | 节点集合与动作集合、seed | 按 seed 伪随机选择目标节点与动作 | `seeded -> deterministic-sequence` | 同 seed + 同输入应复现同序列 | 维护者可复跑 |
| 证据扩展 | `run_config.json`、`summary.json`、`summary.md`、`chaos_events.log` | 输出配置、计数、时间线与结论 | `collected -> exported` | 计数拆分 plan/continuous/total，totals 与明细一致 | 门禁消费方读取 |
- Acceptance Criteria:
  - AC-1: 持续注入 CLI 参数具备解析与校验能力。
  - AC-2: 混合模式下固定计划与持续注入均可执行并被正确计数。
  - AC-3: summary 与 markdown 展示 continuous chaos 配置及统计。
  - AC-4: S9 手册完成示例命令与验收口径更新。
  - AC-5: 短窗实跑证据存在且计数关系正确。
- Non-Goals:
  - 不新增内核级故障类型自动化注入（如 tc 丢包、磁盘注满）。
  - 不实现跨主机/跨地域故障编排。
  - 不修改共识与存储协议语义。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为脚本与门禁可观测性增强，不涉及 AI 推理系统改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 `p2p-longrun-soak.sh` 注入层新增 continuous 调度器，复用既有 chaos 执行器与失败处理路径，并将配置与统计扩展写入标准产物。
- Integration Points:
  - `scripts/p2p-longrun-soak.sh`
  - `testing-manual.md`（S9）
  - `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md`
  - `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.project.md`
- Edge Cases & Error Handling:
  - 过度注入：默认保守参数，支持上限控制防止“人为打爆”。
  - 随机不可复现：记录 seed 并支持显式 seed 复跑。
  - 固定计划与持续注入过密：采用串行执行，失败即落盘 `chaos_events.log` 与 `failures.md`。
  - 参数无效：CLI 校验阶段失败并给出明确错误。
- Non-Functional Requirements:
  - NFR-CHAOS-1: 持续注入逻辑不破坏现有 S9 门禁兼容性。
  - NFR-CHAOS-2: 注入证据具备可审计性（配置、时间线、计数、结果）。
  - NFR-CHAOS-3: 指定 seed 的 run 具备可复现调度序列。
  - NFR-CHAOS-4: 计数字段在 per-topology 与 totals 层保持一致性。
- Security & Privacy: 注入脚本仅用于受控测试环境；日志需避免泄露敏感环境变量。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (CHAOSCNT-1): 方案与项目文档建档。
  - v1.1 (CHAOSCNT-2): 持续注入参数解析、校验与核心调度落地。
  - v2.0 (CHAOSCNT-3): run_config/summary 证据与统计字段扩展。
  - v2.1 (CHAOSCNT-4): S9 手册接线与使用口径更新。
  - v2.2 (CHAOSCNT-5): 完成短窗实跑验证与证据归档。
  - v2.3 (CHAOSCNT-6): 按 strict schema 人工迁移并统一 `.prd` 命名。
- Technical Risks:
  - 风险-1: 注入频率过高导致测试结果偏离真实退化模型。
  - 风险-2: 随机注入造成问题复现困难。
  - 风险-3: 混合模式叠加使失败定位复杂度上升。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-CHAOS-001 | CHAOSCNT-1/2 | `test_tier_required` | continuous CLI 解析/调度回归 + 混合模式执行验证 | P2P chaos 执行链路 |
| PRD-TESTING-LONGRUN-CHAOS-002 | CHAOSCNT-2/3/4 | `test_tier_required` | run_config/summary/markdown 字段核验 + S9 手册口径检查 | 门禁证据可审计性 |
| PRD-TESTING-LONGRUN-CHAOS-003 | CHAOSCNT-3/5/6 | `test_tier_required` | seed 复跑一致性与短窗实跑计数校验 | 复现能力与发布风险评估 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-CHAOS-001 | 在固定计划之外新增持续注入并支持混合模式 | 仅保留固定计划注入 | 提升长期扰动覆盖，且不丢回归可控性。 |
| DEC-CHAOS-002 | 使用 seed 控制伪随机并记录到产物 | 完全随机且不记录种子 | 复现能力是问题定位与门禁审计前提。 |
| DEC-CHAOS-003 | 事件串行执行并复用既有失败处理路径 | 并发注入多个事件 | 降低干扰维度与定位复杂度。 |

## 原文约束点映射（内容保真）
- 原“目标（持续注入 + 固定计划兼容 + 可审计证据）” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“新增 CLI、调度规则、证据扩展” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M0~M4” -> 第 5 章 Phased Rollout（CHAOSCNT-1~6）。
- 原“风险（过度注入、不可复现、叠加过密）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
- 原“当前状态（M0~M4 已完成与实跑计数）” -> 第 6 章 Test Plan 与验证证据语义。
