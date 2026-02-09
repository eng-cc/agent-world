# Viewer 文本可选中与复制能力（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/viewer-copyable-text.md`）
- [x] 输出项目管理文档（本文件）
- [x] CT1：引入 `bevy_egui` 并注册 UI 渲染调度
- [x] CT2：实现可选中复制文本面板（读取现有 UI 文本）
- [x] CT3：Top Controls 新增复制面板显示/隐藏按钮
- [x] CT4：补充/更新测试（按钮切换、语言刷新）
- [x] CT5：运行格式化与测试校验
- [x] CT6：更新项目状态与任务日志

## 依赖
- `crates/agent_world_viewer/Cargo.toml`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/panel_layout.rs`
- `crates/agent_world_viewer/src/i18n.rs`
- `crates/agent_world_viewer/src/copyable_text.rs`

## 状态
- 当前阶段：CT6 完成（复制面板与开关能力已收口）
- 下一阶段：按需补充“复制内容筛选/导出”能力
- 最近更新：完成 `bevy_egui` 接入、按钮联动与测试（2026-02-09）
