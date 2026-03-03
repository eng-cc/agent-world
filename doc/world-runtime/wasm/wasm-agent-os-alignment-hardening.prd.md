# Agent World Runtime：WASM 模块设计对齐增强（agent-os 借鉴）

## 1. Executive Summary
- 在保持 `agent_world` 现有 wasm-1 运行时兼容性的前提下，补齐一批可直接借鉴 `third_party/agent-os` 的能力。
- 本轮聚焦五项落地：
  - 1) ABI/Schema 合约约束增强。
  - 2) effect `cap_slot -> cap_ref` 绑定。
  - 3) pure 模块策略插件链路（作为模块效果前置判定器）。
  - 5) `ModuleContext` 元信息增强。
  - 6) `WasmExecutor` 磁盘编译缓存。

## 2. User Experience & Functionality
- 涉及 crate：
  - `crates/agent_world_wasm_abi`
  - `crates/agent_world_wasm_sdk`
  - `crates/agent_world`
  - `crates/agent_world_wasm_executor`
- 不修改 `third_party/` 下任何代码（仅参考）。
- 不改变现有 manifest 主版本与核心治理流程。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
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

## 5. Risks & Roadmap
- M1（任务1）：manifest ABI/schema 校验落地 + 测试。
- M2（任务2）：cap slot 解析与约束落地 + 测试。
- M3（任务3）：pure 策略插件调用链路落地 + 测试。
- M4（任务5）：`ModuleContext` 元信息贯通 + 测试。
- M5（任务6）：执行器磁盘编译缓存落地 + 测试。

### Technical Risks
- 结构体新增字段会影响大量构造点；必须通过 `serde(default)` 与可选字段保证兼容。
- pure 策略插件若设计不当会引入递归调用或副作用泄漏；本轮仅允许纯判定输出。
- wasmtime 序列化缓存与引擎配置强耦合；需引入 engine fingerprint 并对损坏缓存容错。

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
