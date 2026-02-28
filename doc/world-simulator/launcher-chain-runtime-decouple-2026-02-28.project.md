# 启动器链路重构：链运行时与游戏进程解耦（2026-02-28）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 新增 `world_chain_runtime`：节点启动/停止、状态 API、余额 API。
- [x] T2 重构 `world_game_launcher`：默认托管 `world_chain_runtime`，`world_viewer_live` 切到 `--no-node`。
- [x] T3 更新发行链路：`build-game-launcher-bundle.sh` 纳入 `world_chain_runtime`，更新桌面/CLI 入口参数透传。
- [x] T4 回归与收口：required 测试、项目状态与文档更新。

## 依赖
- `agent_world_node`（NodeRuntime/PoS/P2P）
- `agent_world::runtime::World`（execution world 读取余额）
- 现有 `world_game_launcher` 与 `agent_world_client_launcher`

## 状态
- 当前阶段：已完成（T0~T4）。
- 当前任务：无（项目结项）。
