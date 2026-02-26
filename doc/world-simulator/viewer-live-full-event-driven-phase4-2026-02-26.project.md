# Viewer Live 完全事件驱动改造 Phase 4（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 控制事件化：`StepRequested` / `SeekRequested` 接线
- [x] T2 回归测试：Step/Seek 控制语义 + live 基线语义
- [x] T3 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：已完成（T0~T3）
- 备注：Phase 4 已完成；进入 Phase 5 推进共识提交事件化与有界背压治理。
