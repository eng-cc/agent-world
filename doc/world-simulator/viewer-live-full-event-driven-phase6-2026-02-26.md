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

## Phase 6 完成态（2026-02-27）
- 已完成：非共识 LLM 路径接入 `NonConsensusDriveRequested`，推进由事件驱动替代固定 `PlaybackPulse`。
- 已完成：共识/非共识驱动路径在“无事件且无决策痕迹”时跳过 metrics 推送，减少空推送。
- 已完成：补齐驱动模式分流与 idle metrics 节流测试，`viewer::live::tests` 全组通过。

## Phase 7 入口（完全事件驱动达成度评估）
1. 评估并收敛 script 路径是否保留定时脉冲（用于节奏控制）或切换为显式驱动节拍事件。
2. 为 live 主循环补充分级可观测指标（各信号吞吐、merge/drop、平均处理延迟）。
3. 形成“完全事件驱动”验收清单并闭环 Web 端手工/自动化验证。
