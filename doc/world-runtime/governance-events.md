# Agent World Runtime：治理事件与 Shadow 报告（设计分册）

本分册为 `doc/world-runtime.md` 的详细展开。

## 模块失败事件的审计关联（草案）

**关联字段（建议）**
- `proposal_id`：若发生在治理 apply/shadow 流程中
- `trace_id`：运行时调用链路标识
- `module_id` / `wasm_hash` / `version`
- `cause_event_id`：触发该失败的事件/动作

**审计导出建议**
- 对 `ModuleValidationFailed` 输出 `proposal_id` 与 `ShadowReport` 引用（若存在）。
- 对 `ModuleCallFailed` 输出 `trace_id` 与导致的 `EffectIntent`/`WorldEvent` 关联。

## 模块失败事件负载结构（草案）

**ModuleLoadFailed**
```
{
  "module_id": "...",
  "wasm_hash": "...",
  "code": "ARTIFACT_MISSING|HASH_MISMATCH|IO_ERROR",
  "detail": "..."
}
```

**ModuleValidationFailed**
```
{
  "module_id": "...",
  "proposal_id": "...",
  "code": "ABI_INCOMPATIBLE|CAPS_DENIED|LIMITS_EXCEEDED|VERSION_CONFLICT",
  "detail": "..."
}
```

**ModuleCallFailed**
```
{
  "module_id": "...",
  "trace_id": "...",
  "code": "TRAP|TIMEOUT|OUTPUT_TOO_LARGE|EFFECT_LIMIT_EXCEEDED",
  "detail": "..."
}
```

## 模块事件与校验（草案）

**事件结构（示意）**
```
RegisterModule {
  module_id,
  wasm_hash,
  manifest,         // ModuleManifest 快照
  registered_by,
}

ActivateModule {
  module_id,
  version,
  activated_by,
}

DeactivateModule {
  module_id,
  reason,
  deactivated_by,
}

UpgradeModule {
  module_id,
  from_version,
  to_version,
  new_wasm_hash,
  manifest,         // 新 ModuleManifest 快照
  upgraded_by,
}
```

**校验规则（示意）**
- `wasm_hash` 必须与工件内容哈希一致；不存在则拒绝并记录失败事件。
- `manifest` 与 `module_id/wasm_hash/interface_version` 必须一致且合法。
- `required_caps` 必须有对应 CapabilityGrant，且通过 Policy 校验。
- `limits` 必须在系统允许范围内（内存/gas/频率/输出大小）。
- `RegisterModule` 不允许覆盖已有 `module_id` + `version`。
- `UpgradeModule` 需满足版本单调递增，且 `from_version` 与当前激活版本一致。
- 任何模块事件必须来自治理闭环 `apply` 结果，不允许绕过治理直接写入。

## ShadowReport 结构（草案）

> 目标：在 shadow 阶段输出可审计的诊断结果，阻断不可用模块变更。

**ShadowReport（示意）**
```
{
  "proposal_id": "...",
  "status": "passed|failed|warning",
  "checked_at": i64,
  "errors": [
    { "code": "HASH_MISMATCH", "module_id": "m.weather", "detail": "..." }
  ],
  "warnings": [
    { "code": "LIMITS_HIGH", "module_id": "m.weather", "detail": "..." }
  ],
  "modules": [
    { "module_id": "m.weather", "result": "ok|error|warning", "notes": [ "..." ] }
  ]
}
```

**常见错误码（示意）**
- `HASH_MISMATCH`：工件哈希不一致
- `ARTIFACT_MISSING`：工件缺失
- `ABI_INCOMPATIBLE`：接口版本不兼容
- `CAPS_DENIED`：能力/政策拒绝
- `LIMITS_EXCEEDED`：资源上限超出
- `VERSION_CONFLICT`：版本冲突或非单调升级

## ShadowPolicy 配置与传播（草案）

**WorldConfig 扩展**
```rust
struct WorldConfig {
    // ...
    shadow_policy: ShadowPolicy,
}

enum ShadowPolicy {
    AlwaysPass,
    AlwaysFail,
    ByModuleId(HashSet<String>),
}
```

**ShadowReport 关联**
- 报告中可附加 `shadow_policy` 字段用于审计（测试环境可选）。
- 当 `AlwaysFail` 或命中 `ByModuleId` 时，`status=failed` 且错误码为 `SHADOW_FORCED_FAIL`。

## ShadowReport 事件与审计输出（草案）

**GovernanceEvent::ShadowReport（示意）**
```
ShadowReport {
  proposal_id,
  manifest_hash,
  report,         // ShadowReport 结构
}
```

**审计导出字段（建议）**
- `proposal_id`
- `status`
- `checked_at`
- `errors[]` / `warnings[]`
- `module_id` 维度摘要（ok/error/warning 计数）

## GovernanceEvent 负载结构（草案）

**Proposed**
```
{ "proposal_id": "...", "author": "...", "base_manifest_hash": "..." }
```

**ShadowReport**
```
{ "proposal_id": "...", "manifest_hash": "...", "report": { ... } }
```

**Approved**
```
{ "proposal_id": "...", "approver": "...", "decision": "approve|reject", "reason": "..." }
```

**Applied**
```
{
  "proposal_id": "...",
  "manifest_hash": "...",
  "module_changes": { ... },  // ModuleChangeSet（若存在）
  "module_events": [ "RegisterModule", "UpgradeModule", "ActivateModule", "DeactivateModule" ]
}
```
