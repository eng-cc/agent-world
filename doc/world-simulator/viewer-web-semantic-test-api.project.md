# Viewer Web 语义化测试 API（项目管理）

## 任务拆解
- [x] WTA-0 文档建档：设计文档 + 项目管理文档
- [x] WTA-1 `viewer_automation` 支持运行时步骤入队
- [x] WTA-2 `window.__AW_TEST__` wasm 桥接与命令队列
- [x] WTA-3 `app_bootstrap` 接入命令消费/状态发布系统
- [x] WTA-4 回归测试与编译验证（`agent_world_viewer`）
- [x] WTA-5 状态回写与 devlog 收口

## 依赖
- `crates/agent_world_viewer/src/viewer_automation.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `testing-manual.md`

## 状态
- 当前阶段：WTA 全部完成
- 下一步：在 Playwright 闭环脚本中切换到 `window.__AW_TEST__` 语义调用，替代坐标点击
- 最近更新：2026-02-21（WTA-5 完成，功能与测试收口）
