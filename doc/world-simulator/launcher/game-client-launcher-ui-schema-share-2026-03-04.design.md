# 启动器 UI Schema 共享设计（2026-03-04）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.project.md`

## 1. 设计定位
定义 native 与 web 启动器共享的 UI schema 数据模型与消费方式，使字段、分组、文案和可见性策略由单一权威源驱动。

## 2. 设计结构
- schema 定义层：共享 crate `agent_world_launcher_ui` 维护字段元数据与 section 顺序。
- native 渲染层：主配置区按 schema 驱动渲染核心输入，而非重复手写字段列表。
- web 渲染层：通过 `/api/ui/schema` 动态拉取字段并生成表单。
- 兼容诊断层：未知字段映射、空 section 与 schema 拉取失败均保留明确诊断。

## 3. 关键接口 / 入口
- `agent_world_launcher_ui`
- `/api/ui/schema`
- `native_visible` / `web_visible`
- `id/section/kind/label_zh/label_en`

## 4. 约束与边界
- schema 是 native/web 字段定义的唯一权威源。
- `/api/ui/schema` 只暴露 UI 元数据，不返回敏感运行态信息。
- 新增字段必须通过 schema 增量兼容，不得破坏既有渲染。
- 本阶段不统一所有视觉主题，只统一字段定义与可见性。

## 5. 设计演进计划
- 先沉淀共享 schema 结构。
- 再接入 native 与 web 两端渲染。
- 最后补测试与大文件拆分，确保双端一致性长期可维护。
