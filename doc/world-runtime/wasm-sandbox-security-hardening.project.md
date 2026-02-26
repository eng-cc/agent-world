# Agent World Runtime：WASM 沙箱安全补强（项目管理文档）

## 任务拆解
- [x] T0 建立设计文档与任务拆解
- [x] T1 执行器硬化：fuel 兜底、epoch 抢占超时、store 内存限制器
- [ ] T2 模块仓库加载完整性校验（磁盘工件哈希复验）
- [ ] T3 测试补强、文档回写与收口

## 依赖
- `doc/world-runtime/wasm-executor.md`
- `doc/world-runtime/module-storage.md`
- `crates/agent_world_wasm_executor/src/lib.rs`
- `crates/agent_world/src/runtime/world/persistence.rs`

## 状态
- 当前阶段：T2（模块仓库完整性校验进行中）
