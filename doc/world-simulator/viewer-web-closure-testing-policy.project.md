# Viewer Web 端闭环测试策略（项目管理文档）

## 任务拆解

### WCT1 文档建模
- [x] WCT1.1 输出设计文档（`doc/world-simulator/viewer-web-closure-testing-policy.md`）
- [x] WCT1.2 输出项目管理文档（本文件）
- [x] WCT1.3 在总项目文档挂载任务入口

### WCT2 规范迁移
- [x] WCT2.1 更新 `AGENTS.md` 闭环流程为 Web 默认
- [x] WCT2.2 更新 `doc/viewer-manual.md` 闭环章节为 Web 默认
- [x] WCT2.3 更新 `doc/scripts/capture-viewer-frame*.md` 为 fallback 口径
- [x] WCT2.4 更新 `doc/world-simulator/viewer-bevy-web-runtime*.md` 增补策略约束

### WCT3 回归与收口
- [x] WCT3.1 执行最小回归（viewer wasm check + Web 脚本 help + Playwright 闭环）
- [x] WCT3.2 更新项目管理文档状态、总项目文档与开发日志

## 依赖
- `AGENTS.md`
- `doc/viewer-manual.md`
- `doc/scripts/capture-viewer-frame.md`
- `doc/scripts/capture-viewer-frame.project.md`
- `doc/world-simulator/viewer-bevy-web-runtime.md`
- `doc/world-simulator/viewer-bevy-web-runtime.project.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WCT1~WCT3 全部完成。
- 下一步：按该策略执行后续 Viewer 闭环任务（Web 默认，native fallback）。
- 最近更新：2026-02-15（完成最小回归与 Playwright 闭环收口）。
