# Viewer 右侧面板模块开关与本地缓存（项目管理文档）

## 任务拆解
- [x] VRPM1：输出设计文档（`doc/world-simulator/viewer-right-panel-module-visibility.md`）
- [x] VRPM2：输出项目管理文档（本文件）
- [x] VRPM3：新增模块显隐状态资源与默认值
- [x] VRPM4：实现本地缓存路径解析、加载与持久化
- [x] VRPM5：右侧面板接入“模块开关”与各模块显隐联动
- [x] VRPM6：补充/更新测试（状态默认、缓存读写、UI 开关行为）
- [x] VRPM7：执行格式化与测试校验
- [x] VRPM8：回顾并更新本项目文档状态与任务日志

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/i18n.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`

## 状态
- 当前阶段：VRPM8 完成（模块开关与本地缓存能力已落地）
- 下一阶段：按需补充“模块开关预设模板/一键恢复默认”能力
- 最近更新：完成模块显隐、本地 JSON 缓存、测试与校验（2026-02-09）
