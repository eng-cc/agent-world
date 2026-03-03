# Agent World Runtime：WASM 模块设计对齐增强（agent-os 借鉴）

## 目标
- 在保持 `agent_world` 现有 wasm-1 运行时兼容性的前提下，补齐一批可直接借鉴 `third_party/agent-os` 的能力。
- 本轮聚焦五项落地：
  - 1) ABI/Schema 合约约束增强。
  - 2) effect `cap_slot -> cap_ref` 绑定。
  - 3) pure 模块策略插件链路（作为模块效果前置判定器）。
  - 5) `ModuleContext` 元信息增强。
  - 6) `WasmExecutor` 磁盘编译缓存。

## 范围
- 涉及 crate：
  - `crates/agent_world_wasm_abi`
  - `crates/agent_world_wasm_sdk`
  - `crates/agent_world`
  - `crates/agent_world_wasm_executor`
- 不修改 `third_party/` 下任何代码（仅参考）。
- 不改变现有 manifest 主版本与核心治理流程。

## 接口/数据
- `ModuleManifest` 新增 `abi_contract: ModuleAbiContract`（默认兼容）：
  - `abi_version: Option<u32>`
  - `input_schema: Option<String>`
  - `output_schema: Option<String>`
  - `cap_slots: BTreeMap<String, String>`（slot 到 cap_ref 映射）
  - `policy_hooks: Vec<String>`（pure 策略模块链，按顺序执行）
- `ModuleEffectIntent` 新增可选 `cap_slot`，与 `cap_ref` 并行支持。
- `ModuleContext` 新增元信息字段（默认可缺省）：
  - `stage`, `manifest_hash`, `journal_height`, `module_version`, `module_kind`, `module_role`。
- `WasmExecutorConfig` 新增可选 `compiled_cache_dir`，用于 wasmtime serialized module 磁盘缓存。

## 里程碑
- M1（任务1）：manifest ABI/schema 校验落地 + 测试。
- M2（任务2）：cap slot 解析与约束落地 + 测试。
- M3（任务3）：pure 策略插件调用链路落地 + 测试。
- M4（任务5）：`ModuleContext` 元信息贯通 + 测试。
- M5（任务6）：执行器磁盘编译缓存落地 + 测试。

## 风险
- 结构体新增字段会影响大量构造点；必须通过 `serde(default)` 与可选字段保证兼容。
- pure 策略插件若设计不当会引入递归调用或副作用泄漏；本轮仅允许纯判定输出。
- wasmtime 序列化缓存与引擎配置强耦合；需引入 engine fingerprint 并对损坏缓存容错。
