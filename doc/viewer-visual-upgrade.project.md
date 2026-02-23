# Viewer 视觉升级（Phase 10 后增量强化）项目管理文档

## 任务拆解
- [x] VVU-P0：修订设计文档，完成 Phase 10 后增量定位与数据语义对齐。
- [x] VVU-P1：在模拟层新增 Agent 运动学字段与默认序列化兼容（`serde(default)`）。
- [x] VVU-P2：将 Agent 移动改造为耗时推进（按 tick 更新），补齐内核边界测试。
- [x] VVU-P3：打通快照到 Viewer 的速度/方向数据链路，并落地方向与速度视觉反馈。
- [x] VVU-P4：按 `fragment_budget` 落地 Location 破损映射，移除旧口径分支。
- [x] VVU-P5：完成材料差异化增强（重点收敛 Carbon/Composite 专属效果）。
- [x] VVU-P6：补齐新增视觉配置项解析与运行时兼容逻辑（含非法值回退测试）。
- [ ] VVU-P7：执行 required/full 回归、Playwright Web 闭环与性能收口。

## 依赖
- `doc/viewer-visual-upgrade.md`
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/kernel/*`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/scene_helpers_entities.rs`
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/theme_runtime.rs`
- `testing-manual.md`

## 状态
- 当前阶段：VVU-P6 完成。
- 下一阶段：VVU-P7（回归、Web 闭环与性能收口）进行中。
- 阻塞项：无。
