# Agent World Runtime：无限时长运行的序列号滚动与数值防溢出（项目管理文档）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-infinite-sequence-rollover.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-infinite-sequence-rollover.project.md`

### T1 序列滚动与持久化
- [ ] 为 runtime 四类 `next_*_id` 增加 era 状态与滚动分配逻辑
- [ ] snapshot 增加 era 字段并保持旧快照兼容
- [ ] 补充序列滚动与快照 roundtrip 测试

### T2 数值防溢出加固
- [ ] 修复关键未保护加法（资源与规则聚合）
- [ ] 修复 `len as u32` 与 `u64 as i64` 窄化转换风险
- [ ] 补充对应拒绝路径/边界测试

### T3 回归与收口
- [ ] 运行 runtime 与相关 crate 定向测试
- [ ] 更新设计/项目文档状态
- [ ] 更新 `doc/devlog/2026-02-22.md`

## 依赖
- Runtime world 核心：
  - `crates/agent_world/src/runtime/world/mod.rs`
  - `crates/agent_world/src/runtime/world/actions.rs`
  - `crates/agent_world/src/runtime/world/effects.rs`
  - `crates/agent_world/src/runtime/world/governance.rs`
  - `crates/agent_world/src/runtime/world/event_processing.rs`
  - `crates/agent_world/src/runtime/world/persistence.rs`
- Snapshot 结构：
  - `crates/agent_world/src/runtime/snapshot.rs`
- 相关防溢出路径：
  - `crates/agent_world/src/runtime/world/resources.rs`
  - `crates/agent_world/src/runtime/rules.rs`
  - `crates/agent_world/src/simulator/types.rs`
  - `crates/agent_world/src/runtime/world/module_runtime.rs`
  - `crates/agent_world_wasm_executor/src/lib.rs`
  - `crates/agent_world_net/src/execution_storage.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
