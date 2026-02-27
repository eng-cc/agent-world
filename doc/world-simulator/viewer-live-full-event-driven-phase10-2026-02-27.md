# Viewer Live 完全事件驱动改造 Phase 10（2026-02-27）

## 目标
- 清理 `agent_world::viewer` 活跃运行链路中残留的 tick 轮询逻辑，统一为事件驱动推进。
- 删除离线 viewer server 的定时回放推进（`tick_interval`），避免播放过程空 tick 空跑。
- 删除 web bridge 的可配置轮询间隔（`poll_interval`）及其轮询 sleep 链路，收敛为事件触发转发模型。

## 范围
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `crates/agent_world/tests/viewer_offline_integration.rs`（如需同步）
- 与以上接口变更相关的调用点与测试
- 活跃手册/入口示例中的 viewer 旧 tick 参数残留（仅活跃文档，不改历史 devlog）

不在范围内：
- `agent_world_node` 共识/执行主循环中的 `tick_interval`（基础 runtime 机制，需单独阶段重构）
- 历史归档文档与历史 devlog

## 接口/数据
- `ViewerServerConfig`：移除 `tick_interval` 字段及 `with_tick_interval`。
- `ViewerServer` 回放控制：`Play` 改为事件触发推进（一次请求驱动连续事件发出），不再依赖定时 tick。
- `ViewerWebBridgeConfig`：移除 `poll_interval` 字段及 `with_poll_interval`，清理轮询 sleep。

## 里程碑
1. M0：建档（设计文档+项目管理文档）。
2. M1：offline server 去 tick 化并通过 required 测试。
3. M2：web bridge 去轮询化并通过 required 测试。
4. M3：文档与示例收口、阶段结项。

## 风险
- 离线 `Play` 从“定速推送”变为“事件驱动批量推送”后，前端若依赖慢速动画可能表现变化。
- web bridge 去轮询后需确保断连退出、上游重连行为不退化。
- 若误触 node/runtime 基础 tick 机制，可能引入共识回归，需要严格边界控制。
