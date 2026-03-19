# oasis7 Runtime：WASM SDK Wire 类型收敛设计

- 对应设计文档: `doc/world-runtime/wasm/wasm-sdk-wire-types-dedup.design.md`
- 对应项目管理文档: `doc/world-runtime/wasm/wasm-sdk-wire-types-dedup.project.md`

审计轮次: 4


## 1. Executive Summary
- 将 builtin wasm 模块中重复定义的 `ModuleCallInput/ModuleContext/ModuleOutput` 等协议结构收敛到 `agent_world_wasm_sdk`。
- 让各模块仅保留领域业务结构，减少样板代码与协议漂移风险。
- 在不改变运行时 ABI（`alloc/reduce/call`）与模块行为的前提下完成迁移。

## 2. User Experience & Functionality
### In Scope
- 在 `crates/agent_world_wasm_sdk` 增加可复用 wire 类型与编解码 helper。
- `agent_world_builtin_wasm_modules/*` 改为复用 SDK wire 类型，移除本地重复定义。
- 保持 `on_reduce/on_call` 与生命周期 trait 现有语义不变。

### Out of Scope
- 不改 runtime 的 `ModuleKind::{Reducer,Pure}` 分发语义。
- 不改模块业务规则与效果产出语义。
- 不引入新的 runtime 依赖或改动 `third_party/*`。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- SDK 新增（建议以 feature gate 控制）：
  - `ModuleCallInput`
  - `ModuleContext`
  - `ModuleEffectIntent`
  - `ModuleEmit`
  - `ModuleOutput`
  - `empty_output()`
  - `encode_output(output)`
  - `decode_input(input_bytes)`
  - `decode_action(input)`
- 约束：
  - 结构字段保持与当前模块 CBOR 协议兼容；
  - `ModuleOutput` 默认值与 `output_bytes` 计算口径不变；
  - helper 失败语义保持“返回空输出/None”现有策略。

## 5. Risks & Roadmap
- WIRESDK-1：设计文档 + 项目管理文档落地。
- WIRESDK-2：SDK wire 抽象实现 + 23 模块迁移 + 构建/回归验证。

### Technical Risks
- 风险：抽象后个别模块存在字段兼容差异。
  - 缓解：先确认同构结构，再批量替换并跑 m1/m4 sync check。
- 风险：SDK 新增 serde 依赖影响 no_std 路径。
  - 缓解：将 wire 类型放在 feature gate 下，默认不影响核心生命周期 trait。
- 风险：批量替换引入行为回归。
  - 缓解：执行 required-tier 编译与 wasm 构建清单校验，确保 hash 与产物闭环。

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
