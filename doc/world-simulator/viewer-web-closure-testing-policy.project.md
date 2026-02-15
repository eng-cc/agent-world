# Viewer Web 端闭环测试策略（项目管理文档）

## 任务拆解

### WCT1 文档建模
- [x] WCT1.1 输出设计文档（`doc/world-simulator/viewer-web-closure-testing-policy.md`）
- [x] WCT1.2 输出项目管理文档（本文件）
- [x] WCT1.3 在总项目文档挂载任务入口

### WCT2 规范迁移
- [ ] WCT2.1 更新 `AGENTS.md` 闭环流程为 Web 默认
- [ ] WCT2.2 更新 `doc/viewer-manual.md` 闭环章节为 Web 默认
- [ ] WCT2.3 更新 `doc/scripts/capture-viewer-frame*.md` 为 fallback 口径
- [ ] WCT2.4 更新 `doc/world-simulator/viewer-bevy-web-runtime*.md` 增补策略约束

### WCT3 回归与收口
- [ ] WCT3.1 执行最小回归（viewer wasm check + Web 脚本 help）
- [ ] WCT3.2 更新项目管理文档状态、总项目文档与开发日志

## 依赖
- `AGENTS.md`
- `doc/viewer-manual.md`
- `doc/scripts/capture-viewer-frame.md`
- `doc/scripts/capture-viewer-frame.project.md`
- `doc/world-simulator/viewer-bevy-web-runtime.md`
- `doc/world-simulator/viewer-bevy-web-runtime.project.md`
- `doc/world-simulator.project.md`

## 状态
- 当前阶段：WCT1 已完成，WCT2~WCT3 进行中。
- 下一步：完成文档口径迁移并回归收口。
- 最近更新：2026-02-15（完成策略文档建模与任务拆解）。
