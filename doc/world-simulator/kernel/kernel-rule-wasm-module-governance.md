# Agent World Simulator：规则 Wasm 模块装载治理（第五阶段）设计文档

## 目标
- 让 `WorldKernel` 的 wasm pre-action 规则调用不再依赖每次手工传入 `wasm_bytes`，改为“先注册 artifact，再按 hash 激活模块”。
- 为后续接入真实模块分发/升级治理做最小内核准备，先解决本地装载与一致性校验问题。
- 保持默认行为不变：未激活 wasm 规则模块时，内核仍沿用现有 pre-hook 路径。

## 范围
- In Scope：
  - 在 `WorldKernel` 新增规则 wasm artifact 注册表（runtime-only）。
  - 提供注册/查询/按 hash 激活 pre-action wasm 规则模块的 API。
  - 对 hash 与 bytes 做最小一致性约束（同 hash 重复注册内容必须一致）。
  - 补充装载治理测试（missing hash、重复注册冲突、激活成功后行为）。
- Out of Scope：
  - 分布式 artifact 拉取与版本协商。
  - 模块签名验证与权限治理。
  - `KernelRuleCost` 扣费落账。

## 接口 / 数据
- 新增 runtime-only 注册表（不持久化）：
  - `wasm_rule_artifacts: BTreeMap<String, Vec<u8>>`
- 新增 API（草案）：
  - `register_pre_action_wasm_rule_artifact(wasm_hash, wasm_bytes) -> Result<(), String>`
  - `set_pre_action_wasm_rule_module_from_registry(...) -> Result<(), String>`
  - `remove_pre_action_wasm_rule_artifact(wasm_hash) -> bool`
- 行为约束：
  - `set_*_from_registry` 在 hash 不存在时返回错误，不静默降级。
  - 同 hash 重复注册不同 bytes 返回错误，防止误覆盖。

## 里程碑
- M1：完成 KWM-1（artifact 注册表与装载 API）。
- M2：完成 KWM-2（装载治理测试补齐）。
- M3：完成 KWM-3（回归验证与文档收口）。

## 风险
- 注册表仅内存态，重启后需重载：本阶段接受，后续结合模块治理系统解决。
- hash 校验策略过弱可能引入漂移：先保证“同 hash 内容一致”，后续补签名链路。
- API 增加可能导致调用路径混乱：保留旧 API 兼容，逐步迁移调用方。
