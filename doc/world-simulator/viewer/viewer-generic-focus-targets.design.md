# Viewer 通用聚焦目标解析设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-generic-focus-targets.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-generic-focus-targets.project.md`

## 1. 设计定位
定义 Viewer 自动化聚焦/选中的统一 target 语法和 resolver 方案，让更多实体类型共享一套可扩展的 `focus/select/runSteps` 入口。

## 2. 设计结构
- 语法兼容层：保留 `first_agent` / `agent:<id>` 等旧语法，同时引入 `first:<kind>` / `<kind>:<id>`。
- kind 映射层：把 `agent/location/asset/module_visual/power_plant/chunk/fragment` 统一归一到 resolver。
- 索引绑定层：在 `viewer_automation` 单点绑定 scene 索引和 kind 解析。
- 测试收口层：新旧语法、实体 kind 和命中逻辑都通过定向测试保护。

## 3. 关键接口 / 入口
- `viewer_automation.rs`
- `web_test_api.rs`
- `Viewer3dScene` 实体索引
- `first:<kind>` / `<kind>:<id>` target 语法

## 4. 约束与边界
- 不改 Web Test API 对外方法名，只扩展 target 语义。
- resolver 必须集中维护，避免 parser/执行分支多点散落。
- 新增实体类型的默认接入点是 resolver，而不是再扩新语法分支。
- 兼容旧语法是硬约束，不能因通用化破坏既有自动化脚本。

## 5. 设计演进计划
- 先冻结通用 target 语法。
- 再接 kind resolver 与场景索引映射。
- 最后通过解析/命中测试收口自动化聚焦能力。
