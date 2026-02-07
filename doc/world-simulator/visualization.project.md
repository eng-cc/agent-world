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
- [x] UI 测试覆盖：世界面板（headless 断言）
- [x] UI 测试覆盖：事件浏览（headless 断言）
- [x] UI 测试覆盖：回放控制（headless 断言）
- [x] UI 测试覆盖：指标展示（headless 断言）
- [x] UI 测试覆盖：订阅筛选（headless 断言）
- [x] UI 测试覆盖：控制按钮（Play/Pause/Step/Seek）
- [x] UI 测试覆盖：headless 状态输出
- [x] 补充协议/回放单元测试
- [x] viewer headless 模式（无显示环境运行）
- [x] viewer offline 模式（headless 无网络权限运行）
- [x] headless 默认离线并支持强制联网开关（AGENT_WORLD_VIEWER_FORCE_ONLINE）
- [x] 更新任务日志
- [x] 运行测试 `env -u RUSTC_WRAPPER cargo test -p agent_world`
- [x] 在线模式：live viewer server（WorldKernel + demo script）
- [x] 在线模式：CLI（world_viewer_live）与运行说明
- [x] 在线模式：基础单元测试（script/step/reset）
- [x] 在线模式：live viewer server 支持 LLM 决策驱动（--llm）
- [x] 在线模式：前后端联合测试（独立 integration test + feature gate）
- [x] CI：显式执行联测目标（viewer_live_integration + feature）
- [x] 离线回放：前后端联合测试（viewer_offline_integration）
- [x] 修复 live viewer 断连导致联测失败（忽略连接重置）
- [x] UI：新增 Agent 活动面板（位置/电力/最近动作）
- [x] 3D：新增世界背景参照（边界盒 + 地板网格）
- [x] UI 测试覆盖：Agent 活动面板文本（headless 断言）
- [x] 修复多相机 order 冲突（3D/UI）并恢复视角交互
- [x] 修复 3D 轨道相机拖拽输入（基于 cursor delta，支持 Shift+左键平移）
- [x] UI 测试覆盖：选中详情面板（Location + Agent LLM I/O）
- [x] 在线模式：新增 LLM 决策 trace 下发（DecisionTrace）
- [x] UI：新增选中对象详情面板（Agent/Location）
- [x] 提交到 git

## 依赖
- `WorldSnapshot` / `WorldEvent` / `RunnerMetrics`（`crates/agent_world`）
- 持久化文件格式（`snapshot.json` / `journal.json`）
- Bevy（viewer 客户端）

## 状态
- 当前阶段：在线模式增强完成（支持 script/llm 双驱动）
- 最近更新：新增选中对象详情面板并接入 LLM 决策 I/O trace（2026-02-07）
