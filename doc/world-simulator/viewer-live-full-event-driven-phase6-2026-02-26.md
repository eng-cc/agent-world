# Viewer Live 完全事件驱动改造 Phase 6（2026-02-26）

## 目标
- 将非共识路径（script/llm 本地执行）从 `PlaybackPulse` 定时推进进一步收敛为状态变更驱动。
- 减少 `Play` 状态下无效 pulse 和冗余 metrics 推送，继续压缩空跑。
- 保持 viewer 对外协议兼容（不新增/删除对外字段）。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`

不在范围内：
- 不改动共识节点 runtime 协议。
- 不改动 viewer websocket/http 协议。

## 接口/数据
- 新增非共识驱动信号：`NonConsensusDriveRequested`（或等价内部信号）。
- `Play/Pause/Step/Seek` 状态变更触发非共识驱动事件，不再依赖固定间隔 pulse。
- 继续沿用有界队列与 coalesced 投递策略，确保高压场景可控。

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：非共识驱动信号接线，`PlaybackPulse` 降级为兼容兜底或移除。
3. M2：状态变更与 metrics 发射节流（仅在有效推进后推送）。
4. M3：回归测试（非共识推进语义 + 背压语义）。
5. M4：文档收口并评估“完全事件驱动”达成度。

## 风险
- 非共识路径去定时化后，若驱动触发条件不完整，可能出现暂停后无法恢复推进。
- metrics 节流策略若与 UI 期待不一致，可能引起观测延迟误判。
