# Agent World Simulator：Viewer Chat 中文输入兼容修复（项目管理文档）

## 任务拆解
- [x] CIM1 输出设计文档（`doc/world-simulator/viewer-chat-ime-cn-input.md`）
- [x] CIM2 输出项目管理文档（本文件）
- [ ] CIM3 调整 Web Viewer 输入事件处理配置，修复 Chat 输入框中文输入
- [ ] CIM4 回归验证（`test_tier_required` + wasm 目标检查 + Web 闭环截图）
- [ ] CIM5 文档回写、devlog、提交收口

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `scripts/run-viewer-web.sh`

## 状态
- 当前阶段：进行中（CIM3）
- 下一步：完成代码修复并执行回归验证
- 最近更新：CIM1-CIM2 完成（2026-02-16）
