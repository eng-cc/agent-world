# Agent World Simulator：Viewer Chat Web IME EGUI 输入桥接（项目管理文档）

## 任务拆解
- [x] CIB1 输出设计文档（`doc/world-simulator/viewer-chat-ime-egui-bridge.md`）
- [x] CIB2 输出项目管理文档（本文件）
- [x] CIB3 实现 wasm EGUI IME bridge（DOM 事件桥接 + EGUI 事件注入）
- [ ] CIB4 接入启动流程并完成本地编译/测试回归
- [ ] CIB5 Web 闭环验证（Playwright 快照、console、截图）
- [ ] CIB6 文档回写、devlog、提交收口

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/wasm_egui_input_bridge.rs`（新增）
- `crates/agent_world_viewer/Cargo.toml`

## 状态
- 当前阶段：进行中（CIB4）
- 下一步：完成系统接线并执行编译/测试回归
- 最近更新：CIB3 完成（2026-02-16）
