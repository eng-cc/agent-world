# Agent World：M5 可视化与调试（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/visualization.md`）
- [x] 输出项目管理文档（本文件）
- [x] 定义 viewer 协议与消息结构（hello/subscribe/snapshot/event/control）
- [x] 实现 viewer server（离线回放：snapshot/journal → 事件流）
- [x] 新建 Bevy viewer crate（窗口/输入/网络连接）
- [x] UI：世界状态面板（地点/Agent/资源摘要）
- [x] UI：事件浏览器（列表/筛选）
- [x] UI：回放控制（暂停/单步/跳转 tick）
- [x] 基础指标展示（RunnerMetrics）
- [x] UI 自动化测试（Bevy 自带 App/ECS）
- [x] 补充协议/回放单元测试
- [x] viewer headless 模式（无显示环境运行）
- [x] 更新任务日志
- [x] 运行测试 `env -u RUSTC_WRAPPER cargo test -p agent_world`
- [ ] 提交到 git

## 依赖
- `WorldSnapshot` / `WorldEvent` / `RunnerMetrics`（`crates/agent_world`）
- 持久化文件格式（`snapshot.json` / `journal.json`）
- Bevy（viewer 客户端）

## 状态
- 当前阶段：设计与任务拆解完成，进入实现
