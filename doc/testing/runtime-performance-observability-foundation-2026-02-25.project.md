# Agent World Runtime：代码执行性能采集/统计/分析基础（项目管理）

## 任务拆解
- [x] `RPOF-1`：输出设计文档 `runtime-performance-observability-foundation-2026-02-25.md`
- [x] `RPOF-2`：实现 runtime perf 模块（采集 + 统计 + 分析）
  - [x] 新增 simulator runtime perf 数据结构与统计逻辑
  - [x] 增加 health/bottleneck 规则与单测
- [x] `RPOF-3`：接入 `AgentRunner` 与 `RunnerMetrics`
  - [x] `tick/tick_decide_only/notify_action_result` 采集
  - [x] 外部 action 执行补录接口
  - [x] `RunnerMetrics` 扩展与兼容默认值
- [x] `RPOF-4`：输出链路打通
  - [x] `world_llm_agent_demo` 报告与 stdout 接线
  - [x] `world_viewer_live` metrics 链路接线（LLM driver）
- [x] `RPOF-5`：长跑脚本汇总扩展
  - [x] `scripts/llm-longrun-stress.sh` 读取 runtime perf 字段
  - [x] 单场景 summary 与多场景聚合输出补充
- [x] `RPOF-6`：测试与回归
  - [x] 单测：simulator runtime perf 与 runner 采样路径
  - [x] 脚本/命令回归与 `cargo check`
- [x] `RPOF-7`：文档收口
  - [x] 回写项目状态
  - [x] 更新当日 devlog

## 依赖
- 设计文档：`doc/testing/runtime-performance-observability-foundation-2026-02-25.md`
- 测试手册：`testing-manual.md`
- 关键代码：
  - `crates/agent_world/src/simulator/runner.rs`
  - `crates/agent_world/src/bin/world_llm_agent_demo/*`
  - `crates/agent_world/src/viewer/live_*`
  - `scripts/llm-longrun-stress.sh`

## 状态
- 当前阶段：已完成（RPOF-1~RPOF-7 全部收口）
- 阻塞：无
- 风险跟踪：
  - `tick_decide_only` 的 action 执行在 runner 外部，需保证补录路径完整。
  - 新字段接入 `RunnerMetrics` 后需回归 viewer/server 结构字面量初始化。
- 最近更新：2026-02-25（RPOF-7）
