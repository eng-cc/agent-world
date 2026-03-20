# oasis7 Runtime：代码执行性能采集/统计/分析基础（2026-02-25）

- 对应设计文档: `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.design.md`
- 对应项目管理文档: `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: runtime/simulator 缺少统一的代码执行性能观测基础，导致卡顿风险只能靠分散日志和体感排查，难以在 longrun/viewer/demo 链路内形成稳定诊断。
- Proposed Solution: 在 simulator 内建 runtime perf 采集与分析层，覆盖 `tick/decision/action_execution/callback` 四阶段，输出窗口统计、预算超限比、health/bottleneck 结论，并接入 RunnerMetrics 与脚本汇总链路。
- Success Criteria:
  - SC-1: 四阶段 wall time 采集接入 runner 核心路径与外部 action 补录路径。
  - SC-2: 统计快照包含 `avg/p50/p95/p99/min/max`、样本计数、over-budget 比例。
  - SC-3: 输出 `health` 与 `bottleneck` 诊断结论且规则稳定可测试。
  - SC-4: `RunnerMetrics`、`world_llm_agent_demo`、`world_viewer_live`、`llm-longrun-stress.sh` 链路可消费 runtime perf。
  - SC-5: 不依赖外部监控基础设施，保持低侵入与序列化兼容。

## 2. User Experience & Functionality
- User Personas:
  - runtime 开发者：需要定位 tick/decision/action 回调中的热点阶段。
  - 测试维护者：需要在 required 路径中快速判断性能健康等级。
  - 发布负责人：需要可追溯性能证据支撑放行判断。
- User Scenarios & Frequency:
  - 每次性能相关改动后执行 required 测试并查看 runtime perf 快照。
  - longrun 回归时按场景汇总 health/bottleneck，比较前后漂移。
  - viewer/live 异常诊断时追踪 runtime 与 LLM 路径的性能角色分离。
- User Stories:
  - PRD-TESTING-PERF-RPOF-001: As a runtime 开发者, I want stage-level runtime wall-time snapshots, so that I can identify the slowest phase quickly.
  - PRD-TESTING-PERF-RPOF-002: As a 测试维护者, I want stable health and bottleneck signals in RunnerMetrics, so that regressions are caught in required tests.
  - PRD-TESTING-PERF-RPOF-003: As a 发布负责人, I want longrun summaries to include runtime perf diagnostics, so that release evidence includes execution-performance risk.
- Critical User Flows:
  1. Flow-RPOF-001: `runner 执行 tick/decision/action/callback -> 采样 wall time -> 写入窗口与累计统计`
  2. Flow-RPOF-002: `生成 RuntimePerfSnapshot -> 判定 health/bottleneck -> 注入 RunnerMetrics`
  3. Flow-RPOF-003: `demo/viewer/脚本读取 runtime_perf -> 输出 report/summary -> 诊断回归`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 四阶段采样 | `tick/decision/action_execution/callback` wall time | runner 生命周期中自动采样；外部 action 通过补录接口写入 | `idle -> sampled` | 固定窗口默认 512 | runtime 自动执行 |
| 统计快照 | `avg/p50/p95/p99/min/max`、`samples_*`、`over_budget_*` | 每次采样后更新快照 | `updated` | 分位数基于窗口样本；比例用 ppm 输出 | 测试/发布可读 |
| 健康分析 | `health=unknown/healthy/warn/critical` | 按阈值规则判定 | `unknown -> healthy/warn/critical` | `critical` 优先于 `warn` | 维护者可审阅 |
| 瓶颈分析 | `bottleneck=none/tick/decision/action_execution/callback` | 选择 p95 最大阶段 | `none -> stage` | 全无样本返回 `none` | 维护者可审阅 |
| 输出接线 | `RunnerMetrics.runtime_perf`、report/summary 字段 | 序列化输出并被脚本汇总 | `collected -> reported` | `serde(default)` 保持兼容 | 消费方只读 |
- Acceptance Criteria:
  - AC-1: runtime perf 模块在 simulator 侧实现，四阶段采样完整。
  - AC-2: `RuntimePerfSnapshot` 与系列快照字段完整且测试覆盖核心统计与阈值。
  - AC-3: `RunnerMetrics` 扩展后保持历史反序列化兼容。
  - AC-4: demo/viewer/longrun 汇总链路均能消费 runtime perf 字段。
  - AC-5: 文档迁移为 strict schema，并提供原文约束点映射。
- Non-Goals:
  - 不引入 Prometheus/OTel exporter。
  - 不引入外部时序数据库或 flamegraph/profiler 集成。
  - 不在本阶段实现自动调参或自动治理闭环。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为运行时性能观测基础设施，不涉及 AI 模型推理系统改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 simulator runner 增加轻量 wall-time 采集器，分阶段写入窗口统计与累计计数，派生 `RuntimePerfSnapshot` 后通过 `RunnerMetrics` 对外传播，并被 demo/viewer/longrun 脚本统一消费。
- Integration Points:
  - `crates/oasis7/src/simulator/runner.rs`
  - `crates/oasis7/src/bin/world_llm_agent_demo/*`
  - `crates/oasis7/src/viewer/live_*`
  - `scripts/llm-longrun-stress.sh`
  - `testing-manual.md`
  - `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.project.md`
- Edge Cases & Error Handling:
  - 环境抖动噪声：通过固定窗口 + ppm 比例降低偶发峰值干扰。
  - 采集开销回流：保持常量级采样与轻量统计，避免反向拖慢执行路径。
  - 历史字段兼容：`RunnerMetrics.runtime_perf` 使用默认值，避免旧快照反序列化失败。
  - runner 外部 action 路径：要求显式补录接口接线，避免 `action_execution` 低估。
  - 无样本场景：health 回落 `unknown`，bottleneck 返回 `none`。
- Non-Functional Requirements:
  - NFR-RPOF-1: 统计计算稳定并可在 required 测试中重复。
  - NFR-RPOF-2: 输出字段对 longrun 脚本和 report 消费方保持兼容。
  - NFR-RPOF-3: 采样机制对执行路径额外开销保持低侵入。
  - NFR-RPOF-4: 诊断结论在 30 分钟内可完成跨场景对比定位。
- Security & Privacy: 指标仅包含时延统计与计数，不包含敏感业务负载或密钥信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (RPOF-1): 设计文档与项目文档建档。
  - v1.1 (RPOF-2): runtime perf 模块与统计分析落地。
  - v2.0 (RPOF-3/4): RunnerMetrics + demo/viewer 输出链路接通。
  - v2.1 (RPOF-5/6): longrun 脚本汇总扩展与测试回归。
  - v2.2 (RPOF-7/8): 文档收口与 strict schema 迁移。
- Technical Risks:
  - 风险-1: wall time 指标受运行环境波动影响。
  - 风险-2: 采集漏接导致分阶段指标失真。
  - 风险-3: 新字段扩展引发旧数据兼容隐患。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-PERF-RPOF-001 | RPOF-1/2/3 | `test_tier_required` | 统计正确性单测 + runner 采样路径覆盖 | simulator runner 性能画像 |
| PRD-TESTING-PERF-RPOF-002 | RPOF-2/3/4/6 | `test_tier_required` | health/bottleneck 阈值测试 + metrics 输出核验 | required 回归诊断能力 |
| PRD-TESTING-PERF-RPOF-003 | RPOF-4/5/6/8 | `test_tier_required` | demo/report + longrun summary 字段核验 + 文档治理检查 | 发布证据链与性能追溯 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-RPOF-001 | 先做内建本地采集与统计 | 先接外部监控平台 | 低侵入、落地快、便于测试闭环。 |
| DEC-RPOF-002 | 采用窗口统计 + 累计 ppm 双轨 | 仅输出瞬时值 | 兼顾短期波动诊断与长期趋势判断。 |
| DEC-RPOF-003 | RunnerMetrics 统一承载 runtime perf | 分散到各脚本私有字段 | 减少口径分裂并便于复用。 |
| DEC-RPOF-004 | 文档迁移采用人工重写 + 约束映射 | 自动批量改写 | 保证语义完整与审阅可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标：建立 runtime 性能采集/统计/分析闭环并接入 RunnerMetrics” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope（四阶段采样、统计指标、外部监控不在内）” -> 第 2 章规格矩阵与 Non-Goals。
- 原“接口结构 RuntimePerf*、采样时机、窗口模型、预算阈值” -> 第 2 章规格矩阵 + 第 4 章架构与边界处理。
- 原“health/bottleneck 判定策略” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 RPOF-1~4 与测试计划” -> 第 5 章 phased rollout + 第 6 章验证追踪。
- 原“风险（噪声、开销、兼容、补录遗漏）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
