# Agent World Runtime：WASM SDK Wire 类型收敛设计

## 目标
- 将 builtin wasm 模块中重复定义的 `ModuleCallInput/ModuleContext/ModuleOutput` 等协议结构收敛到 `agent_world_wasm_sdk`。
- 让各模块仅保留领域业务结构，减少样板代码与协议漂移风险。
- 在不改变运行时 ABI（`alloc/reduce/call`）与模块行为的前提下完成迁移。

## 范围

### In Scope
- 在 `crates/agent_world_wasm_sdk` 增加可复用 wire 类型与编解码 helper。
- `agent_world_builtin_wasm_modules/*` 改为复用 SDK wire 类型，移除本地重复定义。
- 保持 `on_reduce/on_call` 与生命周期 trait 现有语义不变。

### Out of Scope
- 不改 runtime 的 `ModuleKind::{Reducer,Pure}` 分发语义。
- 不改模块业务规则与效果产出语义。
- 不引入新的 runtime 依赖或改动 `third_party/*`。

## 接口 / 数据
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

## 里程碑
- WIRESDK-1：设计文档 + 项目管理文档落地。
- WIRESDK-2：SDK wire 抽象实现 + 23 模块迁移 + 构建/回归验证。

## 风险
- 风险：抽象后个别模块存在字段兼容差异。
  - 缓解：先确认同构结构，再批量替换并跑 m1/m4 sync check。
- 风险：SDK 新增 serde 依赖影响 no_std 路径。
  - 缓解：将 wire 类型放在 feature gate 下，默认不影响核心生命周期 trait。
- 风险：批量替换引入行为回归。
  - 缓解：执行 required-tier 编译与 wasm 构建清单校验，确保 hash 与产物闭环。
