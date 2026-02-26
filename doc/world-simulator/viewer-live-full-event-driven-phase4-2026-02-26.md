# Viewer Live 完全事件驱动改造 Phase 4（2026-02-26）

## 目标
- 将 `ViewerControl::Step/Seek` 从请求解析路径中剥离，改为主循环显式控制事件执行。
- 保持 `Play/Pause/Step/Seek` 对外协议和响应语义兼容。
- 继续减少“请求分支直接驱动 world”的耦合，为后续统一事件总线和背压治理铺路。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`

不在范围内：
- 不在本阶段引入跨 crate 事件总线。
- 不在本阶段改造共识回放为提交即推送事件（后续阶段处理）。
- 不修改 viewer 对外协议字段。

## 接口/数据
- 新增 live 内部控制信号：`StepRequested`、`SeekRequested`。
- `ViewerRequest` 处理输出扩展为“deferred control effect”：
  - `Step { count }`
  - `Seek { tick }`
- 主循环收到 deferred control effect 后投递控制信号并执行：
  - `StepRequested`：执行 step 循环并回写事件/快照/metrics。
  - `SeekRequested`：执行 seek 并回写快照/metrics/错误响应。

## 里程碑
1. M0：设计文档/项目管理文档建档。
2. M1：`Step/Seek` 事件化接线（请求处理只产出效果）。
3. M2：回归测试（控制语义 + live 基线语义）。
4. M3：文档收口并明确下一阶段（共识事件化 + 背压）。

## 风险
- 控制信号入队后与 `PlaybackPulse` 竞态，若顺序处理不当可能造成额外空脉冲或响应时序漂移。
- `Step` 多步循环搬迁后若遗漏 metrics/快照发射逻辑，可能造成 UI 观测回退。
