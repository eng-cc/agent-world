# Agent World Runtime：审计导出规范（设计分册）

本分册为 `doc/world-runtime.md` 的详细展开。

## 审计导出统一记录格式（草案）

> 目标：统一治理事件、ShadowReport、模块失败事件的审计导出格式，便于检索与归档。

**AuditRecord（示意）**
```
{
  "time": i64,
  "kind": "GovernanceEvent|ShadowReport|ModuleLoadFailed|ModuleValidationFailed|ModuleCallFailed|ManifestMigrated",
  "proposal_id": "...",
  "module_id": "...",
  "trace_id": "...",
  "payload": { ... }   // 对应事件的原始负载
}
```

**索引建议**
- `proposal_id` 维度：追踪一次治理链路的全量记录。
- `module_id` 维度：追踪模块生命周期与失败原因。
- `trace_id` 维度：追踪运行时调用链路。
- `manifest_hash` 维度：追踪迁移前后版本变更。

**ManifestMigrated 审计记录示意**
```
{
  "time": 300,
  "kind": "ManifestMigrated",
  "payload": {
    "from_version": "v1",
    "to_version": "v2",
    "from_hash": "h1",
    "to_hash": "h2",
    "reason": "load_restore"
  }
}
```

**审计过滤建议**
- `AuditFilter.kinds` 应支持 `ManifestMigrated`，便于按迁移事件筛选导出。

**AuditFilter 示例（模块 + 迁移）**
```
{
  "kinds": ["GovernanceEvent", "ShadowReport", "ModuleValidationFailed", "ManifestMigrated"],
  "from_time": 100,
  "to_time": 500,
  "caused_by": "proposal:p-002"
}
```

## 审计导出分页/分片（草案）

- **分页字段**：`limit`（最大记录数）、`cursor`（上一页末尾事件 id）。
- **稳定顺序**：按 `event_id` 或 `time+event_id` 排序，保证翻页一致性。
- **导出接口**：`save_audit_log(path, filter, limit?, cursor?) -> next_cursor?`
- **chunk 导出**：支持按固定数量分片写出多个文件（如 `audit_0001.json`）。

**cursor 约定（示意）**
- 使用 `event_id` 作为 cursor（单调递增，字符串或 u64）。
- 若 cursor 不存在或已过期（日志裁剪后），返回可恢复错误 `CURSOR_INVALID`。

**cursor 编码建议**
- 可用不透明字符串（如 base64 编码的 event_id + 校验和）以避免直接猜测事件编号。
- 解析失败视为 `CURSOR_INVALID`，不泄露内部事件结构。

**与 AuditFilter 的关系**
- 当使用不透明 cursor 时，客户端不应假设可直接比较/递增 event_id。
- 服务端负责将 cursor 解码为内部起点，再结合 `AuditFilter` 进行筛选与分页。

**cursor 编码示例（草案）**
```
raw: "event_id=123|checksum=abcd"
encoded: "ZXZlbnRfaWQ9MTIzfGNoZWNrc3VtPWFiY2Q="
```

**与快照保留/裁剪的关系**
- 当旧事件被裁剪时，低于最早保留 event_id 的 cursor 视为失效。
- 建议返回 `min_valid_cursor`，提示客户端从最新可用位置继续导出。

**cursor 失效恢复示例**
```
request: { cursor: "42" }
response: { error: "CURSOR_INVALID", min_valid_cursor: "120" }
next_request: { cursor: "120" }
```

## 并发导出一致性（草案）

- **快照导出**：导出开始时锁定事件边界（`end_event_id`），导出范围固定为 `[start, end]`。
- **并发写入**：导出过程中新增事件不进入当前导出；需下一次导出获取。
- **审计记录**：导出元信息中记录 `start_event_id/end_event_id`，便于重放与对齐。

**导出元信息建议**
- `export_id`：导出批次标识
- `start_event_id` / `end_event_id`
- `total_records`
- `generated_at`

**导出包装格式（草案）**
```
{
  "meta": {
    "export_id": "exp-001",
    "start_event_id": 100,
    "end_event_id": 200,
    "total_records": 42,
    "generated_at": 1700000000
  },
  "records": [ ...AuditRecord... ]
}
```

## 审计导出示例（草案）

```
[
  {
    "time": 100,
    "kind": "GovernanceEvent",
    "proposal_id": "p-001",
    "payload": { "proposal_id": "p-001", "author": "agent:alpha", "base_manifest_hash": "h0" }
  },
  {
    "time": 120,
    "kind": "ShadowReport",
    "proposal_id": "p-001",
    "payload": {
      "proposal_id": "p-001",
      "status": "passed",
      "checked_at": 120,
      "errors": [],
      "warnings": [],
      "modules": [ { "module_id": "m.weather", "result": "ok", "notes": [] } ]
    }
  },
  {
    "time": 130,
    "kind": "GovernanceEvent",
    "proposal_id": "p-001",
    "payload": { "proposal_id": "p-001", "approver": "agent:beta", "decision": "approve", "reason": "" }
  },
  {
    "time": 140,
    "kind": "GovernanceEvent",
    "proposal_id": "p-001",
    "payload": {
      "proposal_id": "p-001",
      "manifest_hash": "h1",
      "module_changes": {
        "register": [ { "module_id": "m.weather", "version": "0.1.0" } ],
        "activate": [ { "module_id": "m.weather", "version": "0.1.0" } ],
        "deactivate": [],
        "upgrade": []
      },
      "module_events": [ "RegisterModule", "ActivateModule" ]
    }
  }
]
```

**升级失败示例（含 ShadowReport 警告与失败记录）**
```
[
  {
    "time": 200,
    "kind": "GovernanceEvent",
    "proposal_id": "p-002",
    "payload": { "proposal_id": "p-002", "author": "agent:gamma", "base_manifest_hash": "h1" }
  },
  {
    "time": 220,
    "kind": "ShadowReport",
    "proposal_id": "p-002",
    "payload": {
      "proposal_id": "p-002",
      "status": "warning",
      "checked_at": 220,
      "errors": [],
      "warnings": [
        { "code": "LIMITS_HIGH", "module_id": "m.weather", "detail": "max_gas too high" }
      ],
      "modules": [ { "module_id": "m.weather", "result": "warning", "notes": [ "limits high" ] } ]
    }
  },
  {
    "time": 230,
    "kind": "GovernanceEvent",
    "proposal_id": "p-002",
    "payload": { "proposal_id": "p-002", "approver": "agent:delta", "decision": "approve", "reason": "accept warning" }
  },
  {
    "time": 240,
    "kind": "ModuleValidationFailed",
    "proposal_id": "p-002",
    "module_id": "m.weather",
    "payload": {
      "module_id": "m.weather",
      "proposal_id": "p-002",
      "code": "ABI_INCOMPATIBLE",
      "detail": "interface_version mismatch"
    }
  }
]
```
