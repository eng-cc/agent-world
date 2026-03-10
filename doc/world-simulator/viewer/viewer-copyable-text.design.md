# Viewer 文本可选中与复制能力设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-copyable-text.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-copyable-text.project.md`

## 1. 设计定位
定义 Viewer 中可选中文本与复制面板能力：通过引入 `bevy_egui` 承接可选中文本展示，使现有 UI 文本具备复制能力且不破坏 3D 交互边界。

## 2. 设计结构
- UI 接入层：引入 `bevy_egui` 并注册 UI 渲染调度。
- 面板层：提供可复制文本面板，读取现有 Viewer 文本信息。
- 开关层：Top Controls 新增复制面板显示/隐藏按钮。
- i18n 层：按钮文案统一进入 `i18n.rs`，避免单语回归。

## 3. 关键接口 / 入口
- `bevy_egui`
- `copyable_text.rs`
- Top Controls 面板开关
- `panel_layout.rs`
- `i18n.rs`

## 4. 约束与边界
- 复制面板是补充能力，不重写现有信息展示主路径。
- `bevy_egui` 接入后需保持 3D 视口交互边界稳定。
- 本阶段不扩展复制内容筛选/导出能力。
- 文案和按钮状态需有测试覆盖，避免后续漂移。

## 5. 设计演进计划
- 先接入 `bevy_egui` 和复制面板基础能力。
- 再加入开关按钮与语言刷新。
- 最后通过测试和文档回写收口复制能力。
