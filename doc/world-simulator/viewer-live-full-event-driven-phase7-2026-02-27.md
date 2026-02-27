# Viewer Live 完全事件驱动改造 Phase 7（2026-02-27）

## 目标
- 明确 script 路径的节拍策略（保留定时脉冲用于节奏控制，或切换为显式驱动节拍事件）。
- 为 live 主循环补充信号级可观测指标（吞吐、merge/drop、处理延迟），支撑达成度评估。
- 形成“完全事件驱动”验收清单并完成闭环验证。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

不在范围内：
- 不修改对外 viewer 协议。
- 不改动 node/runtime 共识协议。

## 接口/数据
- 新增 live 信号统计快照结构（内部），至少包含：
  - 每类信号入队数、合并数、丢弃数。
  - 主循环处理耗时分位（或最小集合：平均/最大）。
- script 节拍策略配置：
  - `timer_pulse`（兼容）
  - `event_drive`（实验）

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：script 节拍策略可切换实现（默认兼容）。
3. M2：主循环信号级可观测指标接线。
4. M3：回归测试 + Web 闭环验证。
5. M4：文档收口，输出完全事件驱动达成度结论。

## 风险
- script 路径若切换为纯事件驱动，可能出现推进速度与 UI 预期节奏不一致。
- 指标采样若引入额外锁竞争，可能反向放大主循环延迟。
