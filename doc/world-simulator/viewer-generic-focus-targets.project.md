# Viewer 通用聚焦目标（可扩展实体）（项目管理文档）

## 任务拆解
- [x] GFT-0 输出设计文档（`doc/world-simulator/viewer-generic-focus-targets.md`）
- [x] GFT-1 输出项目管理文档（本文件）
- [x] GFT-2 扩展 `viewer_automation` target 语法为通用 kind（兼容旧语法）
- [x] GFT-3 接入 `asset/module_visual/power_plant/power_storage/chunk/fragment` resolver
- [x] GFT-4 新增/更新单元测试（解析 + 目标解析）
- [x] GFT-5 运行定向测试并记录结果
- [x] GFT-6 回写项目状态与 devlog，提交收口

## 依赖
- `crates/agent_world_viewer/src/viewer_automation.rs`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/main.rs`（`Viewer3dScene` 实体索引）

## 状态
- 当前阶段：GFT 完成（GFT-0 ~ GFT-6）
- 下一阶段：后续如新增实体类型，仅需补 target resolver 映射并复跑定向测试
- 最近更新：2026-02-24（通用聚焦 target + 扩展 resolver + 定向测试通过）
