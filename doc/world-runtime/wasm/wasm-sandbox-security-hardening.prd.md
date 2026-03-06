# Agent World Runtime：WASM 沙箱安全补强（设计文档）

审计轮次: 3

- 对应项目管理文档: doc/world-runtime/wasm/wasm-sandbox-security-hardening.prd.project.md

## 1. Executive Summary
- 修补 WASM 沙箱在执行可抢占性与资源强约束上的关键缺口。
- 提高模块工件加载路径的完整性校验强度，降低磁盘工件被篡改后静默加载风险。
- 保持现有模块 ABI 与治理流程不破坏，优先做兼容加固。

## 2. User Experience & Functionality
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


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
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

## 5. Risks & Roadmap
- `T0`：建档与任务拆解。
- `T1`：执行器安全硬化（fuel/epoch/memory limiter）。
- `T2`：模块仓库加载完整性校验。
- `T3`：测试补强、文档回写与验收。

### Technical Risks
- epoch watchdog 的线程管理处理不当可能引入额外开销；需要确保调用结束后及时回收。
- 旧模块若依赖 `max_gas=0` 语义（历史“无限”行为），将出现行为变化；需通过测试确认当前内置模块不受影响。
- 存储完整性校验会让历史被篡改数据从“可加载”变为“拒绝加载”，属于预期安全收敛。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
