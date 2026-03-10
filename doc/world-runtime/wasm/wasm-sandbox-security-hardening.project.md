# Agent World Runtime：WASM 沙箱安全补强（项目管理文档）

- 对应设计文档: `doc/world-runtime/wasm/wasm-sandbox-security-hardening.design.md`
- 对应需求文档: `doc/world-runtime/wasm/wasm-sandbox-security-hardening.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] T-MIG-20260303 (PRD-ENGINEERING-006): 逐篇阅读旧文档并完成人工重写迁移到 `.prd` 命名。
- [x] T0 建立设计文档与任务拆解
- [x] T1 执行器硬化：fuel 兜底、epoch 抢占超时、store 内存限制器
- [x] T2 模块仓库加载完整性校验（磁盘工件哈希复验）
- [x] T3 测试补强、文档回写与收口

## 依赖
- doc/world-runtime/wasm/wasm-sandbox-security-hardening.prd.md
- `doc/world-runtime/wasm/wasm-executor.prd.md`
- `doc/world-runtime/module/module-storage.prd.md`
- `crates/agent_world_wasm_executor/src/lib.rs`
- `crates/agent_world/src/runtime/world/persistence.rs`

## 状态
- 当前阶段：已完成
