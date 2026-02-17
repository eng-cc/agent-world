# Agent World Runtime：Builtin WASM 业务逻辑下沉到模块工程设计

## 目标
- 将 builtin wasm 的业务逻辑从 `crates/agent_world_builtin_wasm_runtime` 迁移到 `crates/agent_world_builtin_wasm_modules/*` 各模块 crate。
- 保持 wasm ABI（`alloc/reduce/call`）与模块 ID/版本常量不变，确保 runtime 集成层无感升级。
- 让每个模块 crate 成为独立闭环工程：入口、生命周期、业务逻辑在同一 crate 内。

## 范围

### In Scope
- 改造 23 个模块 crate：移除对 `agent_world_builtin_wasm_runtime::{reduce_for_module, call_for_module}` 的依赖方式，改为本地实现业务逻辑。
- `agent_world_builtin_wasm_runtime` 仅保留 runtime 侧需要的常量与通用基础能力（非业务决策逻辑）。
- 更新模块构建与回归校验链路，确保 m1/m4 构建产物与 hash/DistFS 同步闭环。

### Out of Scope
- 不变更模块 ID、版本号、治理安装流程。
- 不变更 `agent_world` runtime 的模块治理接口。
- 不引入新的领域行为规则，仅做逻辑承载位置迁移。

## 接口 / 数据
- 模块入口仍为 `alloc/reduce/call`（由 `agent_world_wasm_sdk` 生命周期 trait + 宏导出）。
- runtime 常量来源仍为 `agent_world_builtin_wasm_runtime`（供 world/bootstrap 与测试复用）。
- 业务输出协议（module input/output CBOR + emits/effects/new_state）保持兼容。

## 里程碑
- MBM-1：设计文档 + 项目管理文档落地。
- MBM-2：代码迁移（23 模块业务逻辑下沉 + runtime 去业务分发）与回归收口。

## 风险
- 风险：迁移后模块行为出现细微漂移（决策 notes/cost、状态编码边界）。
  - 缓解：复用原逻辑实现，执行 m1/m4 同步 check 与 required-tier 编译回归。
- 风险：模块依赖扩展导致 wasm 构建不稳定。
  - 缓解：统一依赖版本并跑 `build-builtin-wasm-modules` + sync 校验。
- 风险：runtime 侧遗留旧分发函数被误引用。
  - 缓解：迁移完成后全仓 `rg` 清理 `reduce_for_module/call_for_module` 引用。
