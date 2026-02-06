# Agent World：M6 3D 可视化（项目管理文档）

## 任务拆解
- [x] 输出 3D 可视化设计文档与项目管理文档（`doc/world-simulator/visualization-3d.md`）
- [ ] 3D viewer 最小闭环：Snapshot 初始化 + 事件更新 + 三节点场景可跑通

## 依赖
- Viewer 协议与 live/offline server（`crates/agent_world/src/viewer`）
- `WorldSnapshot` / `WorldEvent` / `GeoPos`（`crates/agent_world`）
- 三节点场景文件（`crates/agent_world/scenarios/triad_p2p_bootstrap.json`）
- Bevy 3D 渲染能力（`crates/agent_world_viewer`）

## 状态
- 当前阶段：设计完成，待实现
- 最近更新：新增 3D 可视化设计文档（2026-02-06）
