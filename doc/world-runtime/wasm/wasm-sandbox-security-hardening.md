# Agent World Runtime：WASM 沙箱安全补强（设计文档）

## 目标
- 修补 WASM 沙箱在执行可抢占性与资源强约束上的关键缺口。
- 提高模块工件加载路径的完整性校验强度，降低磁盘工件被篡改后静默加载风险。
- 保持现有模块 ABI 与治理流程不破坏，优先做兼容加固。

## 范围

### In Scope
- 执行器加固：
  - `max_gas=0` 不再形成 fuel 无上限执行路径。
  - 启用基于 epoch 的可抢占超时（watchdog + epoch deadline）。
  - 为 store 注入内存增长限制器，限制 `memory.grow` 上界。
- 模块存储加固：
  - 从磁盘加载模块工件时重算哈希，并校验与 `wasm_hash` 一致。
- 回归测试：
  - 执行器 fuel/超时/内存硬限制行为测试。
  - 模块存储加载路径篡改检测测试。

### Out of Scope
- 引入完整的工件签名信任链（公私钥、证书轮换、签名验证服务）。
- 新增 host functions 能力面（当前 linker 仍保持空暴露）。
- 改造治理审批策略本身。

## 接口 / 数据

### 执行器（`agent_world_wasm_executor`）
- `ModuleLimits.max_gas` 解释收敛：
  - 当请求值为 `0` 时，执行器按 `WasmExecutorConfig.max_fuel` 注入 fuel，避免无限执行。
- epoch 超时抢占：
  - 每次调用设置 `Store::set_epoch_deadline(1)`。
  - 启动 watchdog，在 `max_call_ms` 超时后调用 `Engine::increment_epoch()`，触发 trap 中断。
  - `Trap::Interrupt` 与 `Trap::OutOfFuel` 统一映射到 `ModuleCallErrorCode::Timeout`。
- 内存增长限制：
  - 使用 `StoreLimitsBuilder::memory_size(max_mem_bytes)` + `Store::limiter`。
  - `trap_on_grow_failure(true)` 使越界增长直接失败而非返回模糊状态。

### 模块存储（`agent_world::runtime::world::persistence`）
- 加载 `module_registry.json` 与 `*.wasm` 工件后，重算 `sha256`。
- 若重算值与记录的 `wasm_hash` 不一致，立即拒绝加载并返回错误。

## 里程碑
- `T0`：建档与任务拆解。
- `T1`：执行器安全硬化（fuel/epoch/memory limiter）。
- `T2`：模块仓库加载完整性校验。
- `T3`：测试补强、文档回写与验收。

## 风险
- epoch watchdog 的线程管理处理不当可能引入额外开销；需要确保调用结束后及时回收。
- 旧模块若依赖 `max_gas=0` 语义（历史“无限”行为），将出现行为变化；需通过测试确认当前内置模块不受影响。
- 存储完整性校验会让历史被篡改数据从“可加载”变为“拒绝加载”，属于预期安全收敛。
