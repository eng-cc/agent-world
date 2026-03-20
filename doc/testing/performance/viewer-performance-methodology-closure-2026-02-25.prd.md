# oasis7：Viewer 性能测试方法论能力补齐（2026-02-25）

- 对应设计文档: `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.design.md`
- 对应项目管理文档: `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: viewer 性能回归长期依赖人工观察，缺少自动采集、门禁判定与跨版本对比的统一方法，发布评审成本高且一致性不足。
- Proposed Solution: 以 `scripts/viewer-owr4-stress.sh` 为核心落地性能方法论闭环，统一采集 `perf_probe` 指标、执行 profile 阈值门禁、支持可选 baseline 对比，并输出结构化 summary。
- Success Criteria:
  - SC-1: 脚本可自动采集 `perf_probe` 并生成 `metrics.csv/summary.md/summary.json`。
  - SC-2: 支持 profile 与阈值覆盖参数，输出明确 gate 结果与失败原因。
  - SC-3: 支持可选 baseline 对比并识别回退比例。
  - SC-4: 短窗 smoke + baseline smoke + help/语法验证可稳定通过。
  - SC-5: Web 渲染性能采样口径明确要求 GPU + headed，避免 SwiftShader 失真。

## 2. User Experience & Functionality
- User Personas:
  - 性能维护者：需要一致、可复现的 viewer 性能回归流程。
  - 测试维护者：需要可直接用于 gate 的脚本产物与阈值配置。
  - 发布负责人：需要跨版本可比的机器可读结果。
- User Scenarios & Frequency:
  - 每次 viewer 性能相关改动后执行短窗回归。
  - 发布前执行完整压测并与历史 baseline 对比。
  - 性能故障复盘时读取 summary 与 CSV 定位回退来源。
- User Stories:
  - PRD-TESTING-PERF-VPMC-001: As a 性能维护者, I want automated viewer perf collection and gating, so that manual observation is no longer the primary method.
  - PRD-TESTING-PERF-VPMC-002: As a 测试维护者, I want configurable profile thresholds and baseline regression checks, so that gate decisions match release risk.
  - PRD-TESTING-PERF-VPMC-003: As a 发布负责人, I want machine-readable summary outputs, so that cross-version comparisons are auditable.
- Critical User Flows:
  1. Flow-VPMC-001: `启动 stress 脚本 -> 自动开启 perf_probe -> 解析关键指标`
  2. Flow-VPMC-002: `应用 profile/阈值 -> 生成 gate 结论 -> 输出失败原因`
  3. Flow-VPMC-003: `加载 baseline CSV（可选）-> 计算回退比例 -> 汇总到 summary`
  4. Flow-VPMC-004: `Web 场景前置校验 GPU/AdapterInfo -> 执行 headed 采样`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 参数化门禁 | `--profile`、`--fps-target`、`--fps-min`、`--max-over-budget-pct`、`--perf-budget-ms` | 运行时覆盖默认阈值 | `configured -> gated` | profile 默认阈值 + 显式覆盖优先 | 测试维护者配置 |
| 基线对比 | `--baseline-csv`、`--max-regression-pct` | 读取历史 CSV 并计算回退 | `compare -> pass/fail` | 回退超阈值即失败 | 发布/QA 审阅 |
| 指标采集 | `frame_ms_avg_last`、`frame_ms_p95_peak`、`over_budget_pct_peak`、`auto_degrade_seen`、`perf_samples` | 解析 `perf_probe` 日志并派生 FPS | `sampled -> summarized` | `fps=1000/frame_ms` 派生 | 脚本自动 |
| 产物输出 | `metrics.csv`、`summary.md`、`summary.json` | 结构化输出并保留每场景结果 | `archived` | CSV 一行一场景 | 测试/发布读取 |
| Web 前置约束 | GPU/`--headed`/非 SwiftShader | 执行前校验运行环境 | `checked -> runnable` | 不满足前置时直接标记口径无效 | 维护者执行 |
- Acceptance Criteria:
  - AC-1: stress 脚本自动采集并输出结构化产物。
  - AC-2: 门禁判定支持 profile + 自定义阈值覆盖。
  - AC-3: baseline 对比可输出回退判定且可选启用。
  - AC-4: help/语法/短窗/baseline smoke 验证链路可复现。
  - AC-5: 文档迁移至 strict schema 并统一 `.prd` 命名。
- Non-Goals:
  - 不在 viewer 代码中新增 DrawCall/VRAM/GPU Pass 真实采集。
  - 不接入外部时序系统（Prometheus/Otel）。
  - 不重构 `testing-manual.md` 主结构。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为性能测试方法论与脚本闭环，不涉及 AI 推理能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过 `viewer-owr4-stress.sh` 将 viewer 性能采样、阈值判定、baseline 回归比较整合为单入口流程；`perf_probe` 提供原始指标，脚本负责派生、判定与产物格式化。
- Integration Points:
  - `scripts/viewer-owr4-stress.sh`
  - `crates/oasis7_viewer/src/perf_probe.rs`
  - `testing-manual.md`
  - `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.project.md`
- Edge Cases & Error Handling:
  - 短窗抖动：通过可配置阈值与 baseline 允许范围缓冲误判。
  - 日志格式漂移：解析缺失时显式 `perf_samples=0` 并判失败。
  - 无 LLM key 场景：标注 `script_fallback`，避免与真实 LLM 模式混比。
  - Web 无 GPU 加速：强制 GPU + headed 前置，拒绝 SwiftShader 口径。
- Non-Functional Requirements:
  - NFR-VPMC-1: 产物格式稳定，可被 CI/脚本长期消费。
  - NFR-VPMC-2: 门禁结论可在运行结束后 5 分钟内生成。
  - NFR-VPMC-3: 基线对比支持跨版本一致口径。
  - NFR-VPMC-4: 不侵入 viewer 主业务链路。
- Security & Privacy: 产物仅记录性能统计与配置，不应包含敏感凭据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (VPMC-1): 设计与项目管理建档。
  - v1.1 (VPMC-2): stress 脚本采集/门禁/对比能力落地。
  - v2.0 (VPMC-3): 语法、help、短窗、baseline smoke 验证收口。
  - v2.1 (VPMC-4): 补齐 Web GPU 前置约束与文档状态更新。
  - v2.2 (VPMC-5): strict schema 迁移与命名统一。
- Technical Risks:
  - 风险-1: 环境抖动导致误判回退。
  - 风险-2: `perf_probe` 格式变化导致解析失效。
  - 风险-3: script_fallback 与 LLM 模式混杂影响可比性。
  - 风险-4: SwiftShader 环境导致 Web 渲染口径失真。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-PERF-VPMC-001 | VPMC-1/2/3 | `test_tier_required` | 脚本语法 + help + 短窗 smoke | viewer 性能回归入口 |
| PRD-TESTING-PERF-VPMC-002 | VPMC-2/3/4 | `test_tier_required` | 阈值门禁与 baseline 对比字段校验 | 发布门禁与回退识别 |
| PRD-TESTING-PERF-VPMC-003 | VPMC-3/4/5 | `test_tier_required` | summary/json 结构检查 + 文档治理检查 | 跨版本可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-VPMC-001 | 优先脚本层补齐性能方法论闭环 | 先改 viewer 内核新增重采样指标 | 低侵入、迭代快、可立即用于 gate。 |
| DEC-VPMC-002 | 阈值 profile + 参数覆盖双模式 | 固定单一阈值 | 适配不同场景密度与发布风险等级。 |
| DEC-VPMC-003 | baseline 对比可选启用 | 强制 baseline 才能运行 | 保留无历史数据场景的可用性。 |
| DEC-VPMC-004 | Web 性能采样强制 GPU + headed 口径 | 允许 headless/SwiftShader | 降低渲染性能误导风险。 |

## 原文约束点映射（内容保真）
- 原“目标：从手工观察升级为自动采集+门禁+对比” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope（脚本增强、无新 GPU 指标实现、不改 manual 结构）” -> 第 2 章规格矩阵与 Non-Goals。
- 原“输入参数、采集字段、输出产物结构” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M1~M3 与当前完成态” -> 第 5 章 phased rollout（VPMC-1~5）。
- 原“风险（抖动、解析失败、script fallback、SwiftShader）” -> 第 4 章边界处理 + 第 5 章风险。
