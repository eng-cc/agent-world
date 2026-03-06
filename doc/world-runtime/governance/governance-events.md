# Agent World Runtime：治理事件与 Shadow 报告（设计分册）

审计轮次: 4

本分册为 `doc/world-runtime/prd.md` 的详细展开。

## 当前实现口径（2026-03-04）
- 治理事件使用 `GovernanceEvent` 枚举统一承载：`Proposed` / `ShadowReport` / `Approved` / `Applied`。
- world 事件体中治理相关入口为 `WorldEventBody::Governance(GovernanceEvent)`。
- 模块运行失败当前统一进入 `WorldEventBody::ModuleCallFailed`，未拆分 `ModuleLoadFailed` / `ModuleValidationFailed` 事件体。
- `ShadowReport` 事件当前仅记录提案标识与影子 hash，不内嵌富报告结构。

## GovernanceEvent 负载结构（当前）

### Proposed
```json
{
  "proposal_id": "...",
  "author": "...",
  "base_manifest_hash": "...",
  "manifest": { "...": "..." },
  "patch": { "...": "..." }
}
```

### ShadowReport
```json
{
  "proposal_id": "...",
  "manifest_hash": "..."
}
```

### Approved
```json
{
  "proposal_id": "...",
  "approver": "...",
  "decision": "approve | reject"
}
```

### Applied
```json
{
  "proposal_id": "...",
  "manifest_hash": "... (optional)",
  "consensus_height": 123,
  "threshold": 5,
  "signer_node_ids": ["node-a", "node-b"]
}
```

## 模块失败事件口径（当前）
- 当前失败事件类型：`ModuleCallFailed`。
- 典型失败来源：
  - 沙箱执行 Trap/OutOfFuel/Interrupted。
  - 输出超限、效果上限超限、无效输出。
  - 模块加载失败（例如工件缺失）在调用阶段映射为 `ModuleCallFailed(code=SandboxUnavailable)`。
- 若后续需要独立 `ModuleLoadFailed` / `ModuleValidationFailed` 事件，应在 `WorldEventBody` 与 `AuditEventKind` 同步扩展后再回写本分册。

## 审计关联（当前）
- 审计维度通过 `AuditEventKind` 过滤：
  - 治理链路：`governance`
  - 模块失败：`module_call_failed`
  - 模块生命周期事件：`module_event`
- 因果链通过 `caused_by` 保留 action/effect 关联。

## 实现锚点
- `crates/agent_world/src/runtime/governance.rs`
- `crates/agent_world/src/runtime/world_event.rs`
- `crates/agent_world/src/runtime/audit.rs`
