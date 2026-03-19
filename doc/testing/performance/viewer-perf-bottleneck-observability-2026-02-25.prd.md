# oasis7：Viewer 性能瓶颈观测能力补齐（2026-02-25）

- 对应设计文档: `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.design.md`
- 对应项目管理文档: `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: 现有 viewer 性能链路主要给出 FPS/帧时/over-budget 结果，能判断是否超标但难以明确“瓶颈在哪”，定位效率低。
- Proposed Solution: 扩展 `RenderPerfSummary` 与 `perf_probe`/`viewer-owr4-stress.sh` 输出，新增 runtime 分阶段指标与容量压力标记，并引入统一 `PerfHotspot` 推断规则。
- Success Criteria:
  - SC-1: `RenderPerfSummary` 输出 runtime 健康、分阶段 p95 与容量 hit 标记。
  - SC-2: `PerfHotspot` 按固定优先级给出单一主瓶颈分类。
  - SC-3: `perf_probe` 与 `viewer-owr4-stress.sh` 能采集并落盘 hotspot/runtime 字段。
  - SC-4: `metrics.csv` 新列尾部追加，保持旧列兼容。
  - SC-5: 单测与脚本最短窗回归可稳定验证新观测字段。

## 2. User Experience & Functionality
- User Personas:
  - 性能维护者：需要快速识别瓶颈类别而非手工拼接多指标。
  - 测试维护者：需要脚本产物直接可读、可比较。
  - 发布负责人：需要 pass/fail 之外的可解释诊断线索。
- User Scenarios & Frequency:
  - viewer 压测后读取 summary，确认主瓶颈类型与峰值阶段。
  - 回归对比时用 CSV 新列观察瓶颈迁移。
  - 故障复盘时区分 runtime、渲染帧、容量命中等路径。
- User Stories:
  - PRD-TESTING-PERF-VPBO-001: As a 性能维护者, I want a deterministic hotspot classification, so that bottleneck triage is fast.
  - PRD-TESTING-PERF-VPBO-002: As a 测试维护者, I want runtime stage p95 and capacity hits exported in probe/stress outputs, so that regressions are measurable in scripts.
  - PRD-TESTING-PERF-VPBO-003: As a 发布负责人, I want summary evidence to include bottleneck diagnostics beyond pass/fail, so that release decisions are defensible.
- Critical User Flows:
  1. Flow-VPBO-001: `viewer 渲染 + runtime 指标聚合 -> RenderPerfSummary 扩展字段输出`
  2. Flow-VPBO-002: `按优先级规则推断 PerfHotspot -> 输出 hotspot_primary`
  3. Flow-VPBO-003: `perf_probe 采集 -> stress 脚本写入 metrics.csv/summary.json`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| summary 扩展 | `runtime_health`、`runtime_bottleneck`、runtime 各阶段 p95、capacity hit | 每轮采样后更新 summary | `collected -> summarized` | 字段缺失时保留 `unknown/none` | viewer 自动输出 |
| hotspot 推断 | `runtime_llm_api/runtime_*/overlay/label/backlog/render_frame/none` | 应用固定优先级得到主瓶颈 | `none -> classified` | 优先级稳定且互斥 | 性能维护者审阅 |
| probe 输出 | hotspot/runtime/capacity 字段 | `perf_probe` 追加键值输出 | `reported` | 保持原有键顺序兼容 | 脚本消费者读取 |
| stress 汇总 | `hotspot_primary` 等峰值字段 | 新列追加到 CSV 尾部并生成 summary | `archived` | 旧列顺序不变 | QA/发布读取 |
- Acceptance Criteria:
  - AC-1: `RenderPerfSummary` 新字段全部可序列化输出。
  - AC-2: hotspot 规则按文档优先级实现并有单测。
  - AC-3: `perf_probe` 输出包含新增键值。
  - AC-4: `viewer-owr4-stress.sh` 输出新增 CSV 列与 summary 字段。
  - AC-5: 文档迁移完成 strict schema 与命名统一。
- Non-Goals:
  - 不引入 GPU timestamp/drawcall/VRAM 真实采样。
  - 不接入外部时序系统（Prometheus/Otel）。
  - 不重写 agent-browser Web 性能框架。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为 viewer 性能观测口径扩展，不涉及 AI 系统改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 viewer 端汇总路径新增 runtime 诊断与容量命中字段，由 `RenderPerfSummary` 提供统一结构；`PerfHotspot` 通过固定优先级映射为可操作分类，再由 `perf_probe` 与 stress 脚本传递到产物层。
- Integration Points:
  - `crates/agent_world_viewer/src/render_perf_summary.rs`
  - `crates/agent_world_viewer/src/perf_probe.rs`
  - `crates/agent_world_viewer/src/egui_right_panel.rs`
  - `scripts/viewer-owr4-stress.sh`
  - `testing-manual.md`
  - `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.project.md`
- Edge Cases & Error Handling:
  - runtime 指标为空：保留 `unknown/none`，避免误判 runtime 瓶颈。
  - hotspot 抖动：固定优先级保证结论稳定可解释。
  - CSV 兼容：新增列仅追加到尾部，不重排历史列。
  - 容量 hit 仅瞬态：summary 同时提供“是否出现过”布尔聚合。
- Non-Functional Requirements:
  - NFR-VPBO-1: 新字段输出对旧脚本消费者向后兼容。
  - NFR-VPBO-2: hotspot 分类在相同输入下稳定确定。
  - NFR-VPBO-3: 诊断字段可在压测后 5 分钟内完成汇总定位。
  - NFR-VPBO-4: 观测扩展对渲染主路径开销保持可控。
- Security & Privacy: 输出仅包含性能统计，不包含敏感用户数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (VPBO-1): 设计与项目文档建档。
  - v1.1 (VPBO-2): summary 字段扩展与 hotspot 规则落地。
  - v2.0 (VPBO-3): probe/stress 输出扩展与脚本回归。
  - v2.1 (VPBO-4): 测试与文档收口。
  - v2.2 (VPBO-5): strict schema 迁移与命名统一。
- Technical Risks:
  - 风险-1: 规则优先级不当导致误导性 hotspot。
  - 风险-2: 新列兼容处理不完整影响历史分析脚本。
  - 风险-3: runtime 数据缺失导致诊断信息不完整。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-PERF-VPBO-001 | VPBO-1/2/3 | `test_tier_required` | hotspot 规则单测 + summary 字段断言 | 瓶颈分类稳定性 |
| PRD-TESTING-PERF-VPBO-002 | VPBO-2/3/4 | `test_tier_required` | perf_probe 输出键值 + stress CSV/summary 字段核验 | 压测产物可用性 |
| PRD-TESTING-PERF-VPBO-003 | VPBO-3/4/5 | `test_tier_required` | 最短窗脚本回归 + 文档治理检查 | 发布性能证据解释力 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-VPBO-001 | 固定优先级 hotspot 推断 | 动态加权综合评分 | 结果可解释性更高、调试成本更低。 |
| DEC-VPBO-002 | CSV 新列尾部追加 | 重新组织列顺序 | 避免破坏历史脚本与对比流程。 |
| DEC-VPBO-003 | runtime 缺失时保留 unknown/none | 直接推断 render_frame | 防止把数据缺失误当性能瓶颈。 |
| DEC-VPBO-004 | 文档人工迁移并保留约束映射 | 批量自动改写 | 保证规则语义与字段口径不丢失。 |

## 原文约束点映射（内容保真）
- 原“目标：从 pass/fail 升级为可定位瓶颈” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope（summary 字段扩展、hotspot 规则、脚本接线）” -> 第 2 章规格矩阵与 Non-Goals。
- 原“RenderPerfSummary 新字段与 PerfHotspot 优先级” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“perf_probe/viewer-owr4-stress 输出扩展、CSV 列追加兼容” -> 第 2 章规格矩阵 + 第 4 章 Edge Cases。
- 原“里程碑 M1~M4” -> 第 5 章 phased rollout（VPBO-1~5）。
- 原“风险（规则抖动、CSV 兼容、runtime 空值）” -> 第 4 章边界处理 + 第 5 章技术风险。
