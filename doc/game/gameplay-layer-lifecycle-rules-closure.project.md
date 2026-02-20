# Gameplay Layer Lifecycle Rules Closure（项目管理文档）

## 任务拆解

### T0 设计建档
- [x] 新建设计文档：`doc/game/gameplay-layer-lifecycle-rules-closure.md`
- [x] 新建项目管理文档：`doc/game/gameplay-layer-lifecycle-rules-closure.project.md`

### T1 协议与状态扩展
- [x] 扩展 `events.rs`：治理提案开启/结算、危机生成/超时、战争结算事件
- [x] 扩展 `gameplay_state.rs` 与 `state.rs`：生命周期状态持久化与事件应用
- [x] 扩展 `event_processing.rs`：治理提案开启与投票约束

### T2 Tick 生命周期推进
- [x] 新增 `world/gameplay_loop.rs` 并接入 `step.rs`
- [x] 在 tick 周期推进治理结算、危机生成/超时、战争自动结算
- [x] 扩展 `module_runtime_labels.rs` 标签

### T3 测试与收口
- [x] 新增/扩展 `runtime/tests/gameplay_protocol.rs` 生命周期测试（`test_tier_required`）
- [x] 跑 `cargo check` 与 gameplay 目标测试
- [x] 回写项目文档状态与 `doc/devlog/2026-02-20.md`

## 依赖
- `README.md`
- `doc/game/gameplay-top-level-design.md`
- `doc/game/gameplay-engineering-architecture.md`
- `doc/game/gameplay-layer-war-governance-crisis-meta-closure.md`
- `testing-manual.md`

## 状态
- 当前状态：`已完成`
- 已完成：T0、T1、T2、T3
- 进行中：无
- 阻塞项：无
