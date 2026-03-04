# Agent World Runtime：审计导出规范（设计分册）

本分册为 `doc/world-runtime/prd.md` 的详细展开。

## 当前实现口径（2026-03-04）
- 导出入口：`World::save_audit_log(path, filter)`。
- 导出内容：`filter` 命中的 `WorldEvent` JSON 数组。
- 当前实现不支持 `limit/cursor` 分页导出，也不支持多文件 chunk 导出。

## 导出接口（当前）

```rust
pub fn save_audit_log(
    &self,
    path: impl AsRef<Path>,
    filter: &AuditFilter,
) -> Result<(), WorldError>
```

- `path`：输出 JSON 文件路径。
- `filter`：审计过滤条件。

## AuditFilter 字段（当前）
- `kinds`: 可选事件类型列表（`AuditEventKind`）。
- `from_time` / `to_time`: 时间区间。
- `from_event_id` / `to_event_id`: 事件 ID 区间。
- `caused_by`: `action` / `effect`。

## AuditEventKind（当前）
- `domain`
- `effect_queued`
- `receipt_appended`
- `policy_decision`
- `rule_decision`
- `action_overridden`
- `governance`
- `module_event`
- `module_call_failed`
- `module_emitted`
- `module_state_updated`
- `module_runtime_charged`
- `snapshot_created`
- `manifest_updated`
- `rollback_applied`

## 导出示例（当前语义）
```json
[
  {
    "id": 1,
    "time": 100,
    "caused_by": null,
    "body": {
      "kind": "Governance",
      "payload": {
        "type": "Proposed",
        "data": {
          "proposal_id": "p-001",
          "author": "agent:alpha",
          "base_manifest_hash": "h0"
        }
      }
    }
  }
]
```

## 待补齐能力（未实现）
- 分页导出（`limit/cursor`）。
- 导出元信息包装（`export_id`、`start/end_event_id`）。
- 多文件分片导出。

以上能力若落地，应先更新 `runtime/world/audit.rs` 与测试，再回写本分册。
