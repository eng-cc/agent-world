# oasis7：P2P 长跑反馈事件注入（2026-03-02）

- 对应设计文档: `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.design.md`
- 对应项目管理文档: `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: S9 长跑当前主要覆盖共识/存储/奖励指标，缺少真实业务事件流量注入，导致反馈提交流程无法在长跑阶段持续验证。
- Proposed Solution: 在 `scripts/p2p-longrun-soak.sh` 增加 feedback submit 事件注入能力，支持周期注入、成功/失败分统计、独立日志与 summary 汇总，并接入 S9 手册验收口径。
- Success Criteria:
  - SC-1: 支持 feedback 注入 CLI 参数（开关、起始、间隔、上限）。
  - SC-2: 注入逻辑可与现有 chaos plan/continuous 并行运行。
  - SC-3: 产出 `feedback_events.log` 并在 `summary.json/summary.md` 输出分项与 totals 计数。
  - SC-4: `testing-manual.md` S9 提供注入示例与通过标准。
  - SC-5: 注入失败不中断主长跑流程，可用于稳定性对比与诊断。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：希望长跑覆盖真实业务写入链路。
  - 发布负责人：希望看到业务注入成功率与失败分布证据。
  - 脚本维护者：希望新增能力不破坏现有 gate 与脚本稳定性。
- User Scenarios & Frequency:
  - 常规 S9 回归：开启低频 feedback 注入观察链路健壮性。
  - 发布前长跑：开启注入并审阅成功/失败统计与 gate 结果。
  - 故障复盘：对比失败日志与节点状态定位瓶颈。
- User Stories:
  - PRD-TESTING-LONGRUN-FEEDINJ-001: As a 测试维护者, I want periodic feedback submissions in longrun, so that business event paths are validated continuously.
  - PRD-TESTING-LONGRUN-FEEDINJ-002: As a 发布负责人, I want auditable success/failure counters and logs, so that release evidence covers business traffic health.
  - PRD-TESTING-LONGRUN-FEEDINJ-003: As a 脚本维护者, I want feedback injection isolated from core gate pipeline, so that transient submission errors do not terminate the entire run.
- Critical User Flows:
  1. Flow-FEEDINJ-001: `解析 feedback CLI 参数 -> 初始化注入计划 -> 周期触发注入`
  2. Flow-FEEDINJ-002: `按节点轮询 + 类别交替（bug/suggestion）提交 /v1/chain/feedback/submit`
  3. Flow-FEEDINJ-003: `记录 feedback_events.log -> 汇总 success/failed 计数到 summary`
  4. Flow-FEEDINJ-004: `手册口径核验 -> 回归收口`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| CLI 配置 | `--feedback-events-enable`、`--feedback-events-interval-secs`、`--feedback-events-start-sec`、`--feedback-events-max-events` | 解析并校验注入参数 | `disabled -> configured -> active` | 默认 `start=30s`、`interval=60s`、`max=0`（直到结束） | 测试维护者可配置 |
| 注入执行 | `/v1/chain/feedback/submit`、`category=bug/suggestion` | 周期注入事件并记录响应 | `scheduled -> submitted -> success/failed` | 节点轮询、类别交替，失败不中断主循环 | 运行脚本自动执行 |
| 并行协同 | feedback + chaos(plan/continuous) | 并行执行不同事件源 | `co-running` | 共享 run 生命周期，统计独立 | QA/发布可审阅 |
| 证据汇总 | `feedback_events.log`、`summary.json` 字段、`summary.md` 展示 | 输出 per-topology 与 totals 统计 | `collected -> reported` | totals=success+failed，字段名稳定 | 门禁消费方读取 |
- Acceptance Criteria:
  - AC-1: S9 主脚本支持 feedback 注入参数与执行器。
  - AC-2: `run_config/summary/markdown` 包含 feedback 配置与统计字段。
  - AC-3: `feedback_events.log` 可追踪每次注入结果。
  - AC-4: S9 手册命令与通过标准更新完成。
  - AC-5: 项目状态与文档收口完成。
- Non-Goals:
  - 不改造 `s10-five-node-game-soak.sh`。
  - 不新增 feedback append/tombstone 注入。
  - 不修改共识或 DistFS 语义。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为脚本事件注入与证据扩展，不涉及 AI 推理系统改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 S9 长跑脚本中新增 feedback 注入子流程，按时间计划触发 HTTP 提交，独立记录日志与计数，并在 summary 汇总，不侵入原有 gate 判定主链。
- Integration Points:
  - `scripts/p2p-longrun-soak.sh`
  - `testing-manual.md`（S9）
  - `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md`
  - `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.project.md`
- Edge Cases & Error Handling:
  - 接口短时抖动：失败记账并写日志，不中断主循环。
  - 注入频率过高：默认保守参数并允许显式上限。
  - 与 chaos 并行时噪声增加：保持日志分流与统计分项，便于拆解。
  - 参数异常：启动阶段校验失败并返回明确错误。
- Non-Functional Requirements:
  - NFR-FEEDINJ-1: 新能力不破坏现有 gate 计算与脚本主流程稳定性。
  - NFR-FEEDINJ-2: 反馈事件统计具备可审计性与可比性。
  - NFR-FEEDINJ-3: 日志字段满足回归对比和故障定位需求。
  - NFR-FEEDINJ-4: 手册与脚本文档口径保持一致。
- Security & Privacy: 注入内容应避免敏感信息；日志仅记录必要诊断字段。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (FEEDINJ-1): 设计文档与项目文档建档。
  - v1.1 (FEEDINJ-2): 脚本参数与注入执行器落地。
  - v2.0 (FEEDINJ-3): run_config/summary/markdown/日志证据扩展。
  - v2.1 (FEEDINJ-4): S9 手册接线与回归验证。
  - v2.2 (FEEDINJ-5): 按 strict schema 人工迁移并统一 `.prd` 命名。
- Technical Risks:
  - 风险-1: 接口抖动拉高失败计数，干扰结果解读。
  - 风险-2: 注入频率设置不当影响主指标稳定性。
  - 风险-3: 脚本复杂度上升引入维护负担。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-FEEDINJ-001 | FEEDINJ-1/2 | `test_tier_required` | 参数解析与注入执行路径验证 | 业务事件注入链路 |
| PRD-TESTING-LONGRUN-FEEDINJ-002 | FEEDINJ-2/3/4 | `test_tier_required` | run_config/summary/markdown/日志字段核验 | 发布证据完整性 |
| PRD-TESTING-LONGRUN-FEEDINJ-003 | FEEDINJ-3/4/5 | `test_tier_required` | 并行场景稳定性与文档治理检查 | 脚本兼容性与追溯一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-FEEDINJ-001 | 反馈提交失败不打断主长跑流程 | 任一失败即终止整场运行 | 降低瞬时抖动对长跑结论的破坏。 |
| DEC-FEEDINJ-002 | 轮询节点 + bug/suggestion 交替注入 | 固定节点或单一类别注入 | 覆盖更广链路并保持实现简单。 |
| DEC-FEEDINJ-003 | 独立 `feedback_events.log` 与分项计数 | 混入通用日志无独立统计 | 审计与复盘效率更高。 |

## 原文约束点映射（内容保真）
- 原“目标（反馈事件注入、业务流量覆盖、可审计证据）” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“CLI/注入策略/证据统计字段” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M1~M4” -> 第 5 章 Phased Rollout（FEEDINJ-1~5）。
- 原“风险（抖动、频率、复杂度）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
- 原“完成态（脚本接入、手册接线、项目收口）” -> 第 6 章验证与追踪。
