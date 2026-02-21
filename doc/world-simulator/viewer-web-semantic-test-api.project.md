# Viewer Web 语义化测试 API（项目管理）

## 任务拆解
- [x] WTA-0 文档建档：设计文档 + 项目管理文档
- [x] WTA-1 `viewer_automation` 支持运行时步骤入队
- [x] WTA-2 `window.__AW_TEST__` wasm 桥接与命令队列
- [x] WTA-3 `app_bootstrap` 接入命令消费/状态发布系统
- [x] WTA-4 回归测试与编译验证（`agent_world_viewer`）
- [x] WTA-5 状态回写与 devlog 收口
- [x] WTA-6 `testing-manual.md` S6 Playwright 示例切换到 `window.__AW_TEST__`
- [x] WTA-7 `getState()` 增补相机语义字段（`cameraMode/cameraRadius/cameraOrthoScale`）

## 依赖
- `crates/agent_world_viewer/src/viewer_automation.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `testing-manual.md`

## 状态
- 当前阶段：WTA 全部完成
- 下一步：可选将 `window.__AW_TEST__` 常用序列封装成脚本级 helper，进一步减少重复命令
- 最近更新：2026-02-21（WTA-7 完成，`getState()` 相机语义状态发布）
