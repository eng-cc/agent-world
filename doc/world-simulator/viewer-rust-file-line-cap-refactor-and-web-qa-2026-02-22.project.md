# Viewer Rust 文件行数上限重构与 Web 闭环对比（项目管理文档）

## 任务拆解
- [x] VFL0：建立设计文档与项目管理文档。
- [x] VFL1：完成低风险拆分（测试外移与小模块下沉），覆盖 `camera_controls`、`viewer_3d_config`、`egui_right_panel`、`tests`、`selection_linking`。
- [x] VFL2：完成高体量拆分，覆盖 `egui_right_panel_chat`、`main`。
- [x] VFL3：执行 viewer 相关回归（viewer crate 单测 + wasm check + 必要定向测试）。
- [x] VFL4：执行 Web Playwright 闭环（S6）并沉淀证据。
- [x] VFL5：回写文档状态、更新 devlog、给出可操作性对比结论并收口。

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VFL0~VFL5 全部完成（已收口）。
- 阻塞项：无。
- 最近更新：2026-02-22 22:47 CST。
