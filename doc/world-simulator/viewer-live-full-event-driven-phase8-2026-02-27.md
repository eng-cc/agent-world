# Viewer Live 完全事件驱动改造 Phase 8（2026-02-27）

## 目标
- 将 script 路径默认且唯一节拍收敛为 `event_drive`，不再保留 `timer_pulse` 回退模式。
- 清理 script 回退链路代码（配置开关、分支判断、定时脉冲信号与线程）。
- 保持对外 viewer 协议不变，在不引入空跑 tick 的前提下维持 play/step/seek 行为。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world/src/viewer/mod.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`

不在范围内：
- 不改动 viewer 对外协议字段。
- 不改动 node/runtime 共识协议与 reward runtime 机制。

## 接口/数据
- 删除 script 节拍策略开关：
  - 移除 `ViewerLiveScriptPacingMode`。
  - 移除 `ViewerLiveServerConfig.script_pacing_mode` 与 `with_script_pacing_mode`。
- script 非共识推进统一走 `NonConsensusDriveRequested` 事件链路。
- 清理 playback pulse 相关内部信号与统计项，保留事件驱动信号统计。

## 里程碑
1. M0：建档（设计文档 + 项目管理文档）。
2. M1：代码收敛到 script-only event-drive（删除回退开关与脉冲链路）。
3. M2：测试改造与 required 回归通过。
4. M3：文档收口与阶段结项。

## 风险
- 移除定时脉冲后，若 `Play` 初始触发链路遗漏，可能表现为不推进。
- 统计项调整后，已有日志解析脚本若依赖旧字段可能失配。
- 若仍有外部调用依赖已删除配置项，可能触发编译错误，需要同步收敛调用侧。
