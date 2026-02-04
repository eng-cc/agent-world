# Agent World Runtime：AOS 风格 world+agent 运行时（设计文档）

## 目标
- 在现有 `agent-world` 中实现一套 **world+agent 运行时**，借鉴 AgentOS 的关键优势：确定性、可审计、可回放、能力/政策边界、显式副作用与收据、受控升级；以**自由沙盒 + WASM 动态模块**作为基础能力。
- 让世界成为第一性：所有状态改变必须经由 **事件 → 规则校验 → 状态演化** 的统一路径，可追溯、可重放。
- 为后续规模化（多 Agent、高并发交互、长期运行）打下可演化的运行时基座。
- 允许 Agent 通过可治理的模块演化引入“新事物”（Rust → WASM），并保证审计/回放与能力边界。

## 技术参考（AgentOS）
- **AIR 风格控制面**：以结构化数据描述模块/计划/能力/政策；本项目以 manifest/patch 方式做简化对齐。
- **WASM reducers/pure modules**：确定性计算在沙箱模块内完成，避免内核承载复杂逻辑。
- **Effects/Receipts**：所有外部 I/O 必须显式声明并生成收据，纳入事件流审计。
- **Capability/Policy**：无环境授权，最小权限授权与策略审计。
- **Shadow → Approve → Apply**：受控升级流程作为系统演化的基本机制。
- **Minimal trusted base**：内核保持最小可信边界，复杂性外置到模块/适配器。

## 范围

### In Scope（V1）
- **确定性内核**：单线程 stepper，固定顺序处理事件，避免不可控并发。
- **事件溯源**：事件日志 + 快照；世界状态由事件重放导出。
- **显式副作用**：外部 I/O 只能以 Effect 意图表达，不可在 reducer 内直接执行。
- **Receipt 机制**：每个 effect 产生收据并写回事件流。
- **收据签名与校验**：收据需包含签名/校验信息，支持审计与回放一致性（V1 采用 HMAC‑SHA256）。
- **能力与政策**：capability grants + policy gate，限制 effect 的类型、范围与预算。
- **控制面（Manifest）**：以数据结构声明 reducers、effects、caps、policies、routing。
- **受控升级**：支持 propose → shadow → approve → apply 的最小治理流程（用于 manifest 级别变更）。
- **回滚与审计**：支持基于快照的回滚，并记录 RollbackApplied 审计事件。
- **Patch 与 Diff/Merge**：支持 manifest patch（set/remove）与 diff/merge 辅助函数。
- **冲突检测与快照清理**：merge 时报告冲突；快照保留策略可驱动文件清理。
- **可观测性**：事件尾随、收据查询、per-agent timeline。
- **文件级持久化**：journal/snapshot 的落盘与恢复接口。
- **WASM 模块沙箱（接口预留）**：模块以内容哈希登记，支持动态装载/调用；模块仅通过事件/Effect 与外部交互。

### Agent 机制（已选）
- **Agent = Cell（keyed reducer）**：每个 agent 由同一 reducer 的 keyed 实例表示（key = `agent_id`），拥有独立状态与 mailbox；事件路由按 key 分发，调度器以确定性顺序轮询。

### Out of Scope（V1 不做）
- 完整 AIR 规范与生产级 WASM 运行时实现（V1 仅保留简化版 manifest 与模块接口占位）。
- 跨 world 协议与一致性（后续阶段考虑）。
- 复杂并行执行（保持单线程确定性）。
- 完整 UI/可视化工具链（仅保留 CLI/日志接口）。

## 接口 / 数据

### 核心概念
- **World**：事件日志 + 快照 + manifest + reducer 状态集合。
- **Agent Cell**：同一 reducer 的 keyed 实例（`agent_id` 为 key）。
- **Reducer**：纯函数式状态机（输入事件 + 旧状态 → 新状态 + Effect 意图）。
- **WASM Module**：以 Rust 等语言编写并编译为 WASM 的可装载模块（reducer 或纯计算组件），运行在沙箱内。
- **Sandbox**：模块的受控执行环境，能力/政策约束在此生效，禁止直接 I/O。
- **Effect / Receipt**：显式副作用与其回执；重放时只读取 receipt，不重新执行 I/O。
- **Capability / Policy**：运行时授权与治理规则。

### WASM 扩展接口（草案）

> 目标：允许 Agent 自行设计“新事物”模块（Rust → WASM），由世界内核以事件/接口动态调用；模块只产生确定性计算与显式 Effect 意图。

**ModuleManifest（控制面条目）**
```rust
struct ModuleManifest {
    module_id: String,         // 内容地址或哈希
    name: String,
    version: String,           // 语义化版本
    kind: ModuleKind,          // Reducer / Pure（后续可扩展）
    wasm_hash: String,         // 模块工件哈希
    interface_version: String, // 例如 "wasm-1"
    exports: Vec<String>,      // 导出函数名
    subscriptions: Vec<ModuleSubscription>,
    required_caps: Vec<CapabilityRef>,
    limits: ModuleLimits,      // 沙箱资源上限
}
```

**ModuleKind**
- `Reducer`：有状态的确定性 reducer（输入事件 → 新状态 + Effect 意图）
- `Pure`：无状态纯函数组件（输入 → 输出）

**ModuleSubscription**
- `event_kinds`: Vec<String>（订阅的事件类型）
- `action_kinds`: Vec<String>（可选，订阅的动作类型）
- `filters`: 可选过滤条件（例如仅关注某类 owner/地点）

**ModuleLimits（示意字段）**
```rust
struct ModuleLimits {
    max_mem_bytes: u64,        // 线性内存上限
    max_gas: u64,              // 指令燃料
    max_call_rate: u32,        // 每 tick 最大调用次数
    max_output_bytes: u64,     // 输出上限（ModuleOutput 编码后大小）
    max_effects: u32,          // 单次调用最大 effect 数量
    max_emits: u32,            // 单次调用最大 event 数量
}
```

**Reducer 调用签名（示意）**
```rust
fn reduce(event: WorldEvent, state: Bytes, ctx: ModuleContext) -> ModuleOutput
```

**Pure 调用签名（示意）**
```rust
fn call(input: Bytes, ctx: ModuleContext) -> Bytes
```

**ModuleContext / ModuleOutput（示意）**
- `ModuleContext`：`{ time, origin, world_config, module_id, trace_id }`
- `ModuleOutput`：`{ new_state, effects: Vec<EffectIntent>, emits: Vec<WorldEvent> }`

**模块生命周期事件（占位）**
- `RegisterModule / ActivateModule / DeactivateModule / UpgradeModule`
- 以事件写入日志，支持审计与回放

**模块失败事件（占位）**
- `ModuleLoadFailed { module_id, wasm_hash, reason }`
- `ModuleValidationFailed { module_id, reason }`
- `ModuleCallFailed { module_id, trace_id, reason }`

### 模块失败事件的审计关联（草案）

**关联字段（建议）**
- `proposal_id`：若发生在治理 apply/shadow 流程中
- `trace_id`：运行时调用链路标识
- `module_id` / `wasm_hash` / `version`
- `cause_event_id`：触发该失败的事件/动作

**审计导出建议**
- 对 `ModuleValidationFailed` 输出 `proposal_id` 与 `ShadowReport` 引用（若存在）。
- 对 `ModuleCallFailed` 输出 `trace_id` 与导致的 `EffectIntent`/`WorldEvent` 关联。

### 模块失败事件负载结构（草案）

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

### 模块事件与校验（草案）

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

### ShadowReport 结构（草案）

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

### ShadowPolicy 配置与传播（草案）

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

### ShadowReport 事件与审计输出（草案）

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

### GovernanceEvent 负载结构（草案）

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

### 审计导出统一记录格式（草案）

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

### 审计导出分页/分片（草案）

- **分页字段**：`limit`（最大记录数）、`cursor`（上一页末尾事件 id）。
- **稳定顺序**：按 `event_id` 或 `time+event_id` 排序，保证翻页一致性。
- **导出接口**：`save_audit_log(path, filter, limit?, cursor?) -> next_cursor?`
- **chunk 导出**：支持按固定数量分片写出多个文件（如 `audit_0001.json`）。

### 审计导出示例（草案）

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

### ABI 与序列化（草案）

> 目标：模块与宿主之间的输入/输出采用**确定性**编码，保证回放与跨平台一致性。

**编码格式**
- 使用 **Canonical CBOR**（键排序、确定性编码）。
- 禁止 NaN；浮点仅在明确字段允许时使用（默认使用整数与字节串）。
- `Bytes` 一律使用 CBOR byte string。

**ModuleContext（CBOR Map）**
```
{
  "v": "wasm-1",
  "module_id": "...",
  "trace_id": "...",
  "time": i64,
  "origin": { "kind": "event|action|system", "id": "..." },
  "world_config_hash": "...",
  "limits": { "max_mem_bytes": u64, "max_gas": u64, "max_output_bytes": u64 }
}
```

**Reducer 输入（CBOR Map）**
```
{
  "ctx": ModuleContext,
  "event": Bytes,   // WorldEvent 的 canonical CBOR
  "state": Bytes    // reducer 当前状态（canonical CBOR）
}
```

**Pure 输入（CBOR Map）**
```
{ "ctx": ModuleContext, "input": Bytes }
```

**ModuleOutput（CBOR Map）**
```
{
  "new_state": Bytes | null,
  "effects": [ Bytes ], // EffectIntent 的 canonical CBOR 列表
  "emits": [ Bytes ]    // WorldEvent 的 canonical CBOR 列表
}
```

**错误约定**
- 模块返回非规范 CBOR、输出超限或字段缺失时，宿主记录 `ModuleCallFailed` 事件并拒绝输出。

### 关键数据结构（草案）
- `WorldEvent`：`{ id, time, kind, payload, caused_by }`
- `EffectIntent`：`{ intent_id, kind, params, cap_ref, origin }`
- `EffectReceipt`：`{ intent_id, status, payload, cost?, timestamps, hash }`
- `CapabilityGrant`：`{ name, cap_type, params, expiry? }`
- `PolicyRule`：`{ when, decision }`
- `Manifest`：`{ reducers, modules, module_changes?, effects, caps, policies, routing, defaults }`
- `ManifestPatch`：`{ base_manifest_hash, ops[], new_version? }`，支持 set/remove（merge 要求基于同一 base hash）
- `PatchMergeResult`：`{ patch, conflicts[] }`，冲突包含路径与涉及的 patch 索引
- `PatchConflict`：`{ path, kind, patches[], ops[] }`（kind: same_path/prefix_overlap）
- `Proposal`：`{ id, author, base_manifest_hash, manifest, status }`
- `GovernanceEvent`：`Proposed/ShadowReport/Approved/Applied`
- `RollbackEvent`：`{ snapshot_hash, snapshot_journal_len, prior_journal_len, reason }`
- `SnapshotCatalog`：`{ records[], retention }`
- `SnapshotRetentionPolicy`：`{ max_snapshots }`
- `AuditFilter`：`{ kinds?, from_time?, to_time?, from_event_id?, to_event_id?, caused_by? }`

### 模块治理与兼容性（草案）
- **版本与兼容**：`interface_version` 由内核维护；模块声明兼容范围，若不兼容则拒绝加载。
- **治理闭环**：模块变更走 `propose → shadow → approve → apply`，升级/回滚均形成审计事件。
- **沙箱限制**：内存上限、指令燃料（gas）、调用频率、输出/事件大小上限。
- **能力/政策**：模块不能直接 I/O，只能产出 `EffectIntent`，由 capability/policy 决定是否执行。
- **确定性约束**：禁止读取真实时间/随机数；非确定性来源必须通过 receipt 写回事件流。

### 模块注册表与存储（草案）

> 目标：用**内容寻址**与**审计元数据**管理 WASM 模块，支持可回放、可治理的动态装载。

**存储布局（示意）**
- `module_registry.json`：模块索引（哈希 → 元数据）
- `modules/<wasm_hash>.wasm`：WASM 工件（只读、内容地址）
- `modules/<wasm_hash>.meta.json`：模块元信息（manifest 快照）

**module_registry.json（示意结构）**
```
{
  "version": 1,
  "updated_at": 123,
  "records": [
    {
      "wasm_hash": "...",
      "module_id": "m.weather",
      "name": "Weather",
      "version": "0.1.0",
      "interface_version": "wasm-1",
      "kind": "Reducer",
      "registered_at": 120,
      "registered_by": "agent:alpha",
      "audit_ref": "event:1234"
    }
  ]
}
```

**modules/<wasm_hash>.meta.json（示意结构）**
```
{
  "module_id": "m.weather",
  "name": "Weather",
  "version": "0.1.0",
  "interface_version": "wasm-1",
  "kind": "Reducer",
  "wasm_hash": "...",
  "exports": ["reduce"],
  "subscriptions": ["WorldEvent/WeatherTick"],
  "required_caps": ["cap.weather.query"],
  "limits": { "max_mem_bytes": 1048576, "max_gas": 100000, "max_call_rate": 1 }
}
```

**ModuleRecord（索引条目）**
```rust
struct ModuleRecord {
    wasm_hash: String,
    module_id: String,
    name: String,
    version: String,
    interface_version: String,
    kind: ModuleKind,
    registered_at: i64,
    registered_by: String,   // agent_id / system
    audit_ref: String,       // 对应 RegisterModule 事件 id
}
```

**加载/缓存策略**
- **按哈希装载**：模块加载必须提供 `wasm_hash`，不允许同名替换。
- **LRU 缓存**：内存中缓存已编译模块（带 `max_cached_modules` 上限）。
- **冷启动**：按需从 `modules/` 读取工件；找不到则拒绝加载并记录事件。

### 模块注册 Happy Path（草案）

```
ArtifactWrite(wasm_hash) 
  -> ProposeModuleChangeSet(register+activate)
    -> ShadowReport(pass)
      -> Approve
        -> Apply
          -> RegisterModule event
          -> ActivateModule event
          -> module_registry.json 更新
```

### 模块注册 Failure Path（草案）

```
ArtifactWrite(wasm_hash)
  -> ProposeModuleChangeSet(register+activate)
    -> ShadowReport(failed)
      -> Reject
        -> 无 Apply（不写入模块事件/注册表）
```

```
ArtifactWrite(wasm_hash)
  -> ProposeModuleChangeSet(register+activate)
    -> ShadowReport(pass)
      -> Approve
        -> Apply
          -> ModuleValidationFailed
          -> 无 Register/Activate（不更新注册表）
```

### 集成测试用例（草案）

- **register_happy_path**：artifact 写入 → propose → shadow pass → approve → apply → 注册表更新
- **shadow_fail_blocks_apply**：shadow fail → reject → 不产生模块事件
- **apply_fail_records_validation**：apply 阶段校验失败 → ModuleValidationFailed → 注册表不更新
- **upgrade_flow**：升级成功后版本更新、旧版本不可激活
- **audit_export_contains_module_events**：审计导出包含 GovernanceEvent + Module*Failed/ShadowReport

### 测试基架建议（草案）

- 文件组织：`crates/agent_world/tests/module_lifecycle.rs`
- 共享夹具：`TestWorldBuilder`（构造 world + manifest + registry 初始态）
- 伪造工件：内存内生成 dummy wasm bytes + 计算 hash
- Shadow 注入：允许在测试中强制 shadow 失败/通过
- 断言：事件流顺序、注册表内容、审计导出记录

**TestWorldBuilder（示意 API）**
```rust
struct TestWorldBuilder {
    with_registry: bool,
    with_manifest: bool,
    caps: Vec<CapabilityGrant>,
}

impl TestWorldBuilder {
    fn new() -> Self;
    fn with_registry(mut self) -> Self;
    fn with_manifest(mut self) -> Self;
    fn with_caps(mut self, caps: Vec<CapabilityGrant>) -> Self;
    fn build(self) -> World;
}
```

**Dummy WASM 工件工具（示意）**
```rust
fn dummy_wasm_bytes(label: &str) -> Vec<u8>;  // 生成确定性字节
fn wasm_hash(bytes: &[u8]) -> String;         // 计算内容哈希（sha256/hex）
```

**Shadow 注入建议（示意）**
- 在 `WorldConfig` 或测试构造器中注入 `ShadowPolicy`（`AlwaysPass` / `AlwaysFail` / `ByModuleId(HashSet)`）。
- 测试中将 `ShadowPolicy` 设置为 `AlwaysFail` 以触发 shadow reject 分支。

**审计与可回放**
- 注册/激活/升级事件进入日志，`module_registry.json` 可由事件重建。
- 任意运行时模块版本都可由 `wasm_hash` 唯一定位。

### 模块治理流程接入（草案）

**流程（概要）**
1. Agent 编译模块 → 计算 `wasm_hash` → 将工件写入 `modules/<wasm_hash>.wasm`。
2. 生成 `ModuleManifest` 与变更计划（Register/Activate/Upgrade）。
3. `propose → shadow → approve → apply`：治理闭环审查模块变更。
4. `apply` 成功后写入 `RegisterModule/ActivateModule/UpgradeModule` 事件，并更新模块注册表。

**Shadow 校验（示意）**
- 工件存在性与哈希一致性校验（`wasm_hash`）。
- 接口版本与 ABI 兼容性校验（`interface_version`）。
- `required_caps` 与 `Policy` 规则校验。
- `limits` 合法性校验（不超过系统上限）。

**Apply 行为（示意）**
- 生成模块生命周期事件并追加到事件流。
- 更新 `module_registry.json` 与内存缓存索引。
- 若任何校验失败，拒绝 apply 并记录 `ModuleValidationFailed`。

**ModuleChangeSet（示意）**
```rust
struct ModuleChangeSet {
    register: Vec<ModuleManifest>,
    activate: Vec<ModuleActivation>,
    deactivate: Vec<ModuleDeactivation>,
    upgrade: Vec<ModuleUpgrade>,
}

struct ModuleActivation { module_id: String, version: String }
struct ModuleDeactivation { module_id: String, reason: String }
struct ModuleUpgrade { module_id: String, from_version: String, to_version: String, wasm_hash: String }
```

**Apply 事件序列（示意）**
1. `RegisterModule`（若有 register）
2. `UpgradeModule`（若有 upgrade）
3. `ActivateModule`（若有 activate）
4. `DeactivateModule`（若有 deactivate）
> 顺序固定以保证回放确定性；同类事件按 `module_id` 字典序处理。

**ModuleChangeSet 应用算法（草案）**
```
fn apply_module_changes(changes: ModuleChangeSet) -> Result<()> {
  validate_changes(changes)?;
  shadow_check(changes)?;

  // 1) register
  for m in sort_by_module_id(changes.register) {
    write_event(RegisterModule { ..m });
    registry.insert(m);
  }

  // 2) upgrade
  for u in sort_by_module_id(changes.upgrade) {
    write_event(UpgradeModule { ..u });
    registry.update(u);
  }

  // 3) activate
  for a in sort_by_module_id(changes.activate) {
    write_event(ActivateModule { ..a });
    registry.activate(a);
  }

  // 4) deactivate
  for d in sort_by_module_id(changes.deactivate) {
    write_event(DeactivateModule { ..d });
    registry.deactivate(d);
  }

  Ok(())
}
```

**ModuleChangeSet 校验规则（示意）**
- `module_id` 在 `register/activate/deactivate/upgrade` 内不得重复冲突。
- `register` 与 `upgrade` 不能同时针对同一 `module_id`。
- `activate` 与 `deactivate` 若同时出现，按顺序执行，但必须显式声明（避免隐式切换）。
- `upgrade` 需要 `from_version` == 当前激活版本；`to_version` 必须单调递增。
- `register` 后默认不自动激活，需显式 `activate`（保证可审计变更意图）。
- 所有变更均需通过 shadow 校验（hash/ABI/limits/caps）。

### ModuleChangeSet 在 Manifest/Patch 中的编码（草案）

**Manifest 扩展**
```rust
struct Manifest {
    reducers: Vec<ReducerSpec>,
    modules: Vec<ModuleManifest>,
    module_changes: Option<ModuleChangeSet>,
    effects: Vec<EffectSpec>,
    caps: Vec<CapabilityGrant>,
    policies: Vec<PolicyRule>,
    routing: RoutingSpec,
    defaults: Defaults,
}
```

**Patch 路径约定（示意）**
- `/modules`：替换模块清单（`set`）
- `/module_changes`：提交模块变更计划（`set`）

**Patch 示例（注册 + 激活）**
```
{
  "base_manifest_hash": "...",
  "ops": [
    { "op": "set", "path": "/modules", "value": [ ... ] },
    { "op": "set", "path": "/module_changes", "value": {
        "register": [ { "module_id": "m.weather", "version": "0.1.0", ... } ],
        "activate": [ { "module_id": "m.weather", "version": "0.1.0" } ],
        "deactivate": [],
        "upgrade": []
    } }
  ]
}
```

**Patch 示例（升级）**
```
{
  "ops": [
    { "op": "set", "path": "/modules", "value": [ ... ] },
    { "op": "set", "path": "/module_changes", "value": {
        "register": [],
        "activate": [],
        "deactivate": [],
        "upgrade": [
          { "module_id": "m.weather", "from_version": "0.1.0", "to_version": "0.2.0", "wasm_hash": "..." }
        ]
    } }
  ]
}
```

### ModuleChangeSet 生命周期（草案）

**提案阶段**
- `module_changes` 仅存在于提案的 manifest 中，用于描述预期模块变更。
- Shadow 阶段完成校验后生成 `ShadowReport`（包含错误/警告/通过项）。

**Apply 阶段**
- apply 时按事件序列写入生命周期事件，并更新注册表索引。
- apply 完成后，活动 manifest 中的 `module_changes` 应清空为 `null`（避免重复应用）。

**回放阶段**
- 注册表由事件流重建；`module_changes` 仅用于审计历史，不作为运行时指令。

### 多补丁冲突处理（草案）

> 目标：在 merge/patch 叠加时，确保模块变更的确定性与可审计性。

**冲突判定**
- 同一 `module_id` 在不同 patch 中出现 `register/upgrade/activate/deactivate` 任一重叠，即视为冲突。
- `register` 与 `upgrade` 对同一 `module_id` 视为硬冲突，必须人工选择。
- `activate` 与 `deactivate` 对同一 `module_id` 视为软冲突，允许选择保留其一。
- `upgrade` 的 `from_version` 不一致视为冲突。

**合并策略**
- 默认 **拒绝自动合并**，返回 `PatchMergeResult.conflicts` 供治理人工裁决。
- 若治理提供显式决策（保留 patch A 或 B），则按决策应用并记录审计事件。
- 合并后的 `ModuleChangeSet` 需重新通过 shadow 校验。

> V1 约定：治理“补丁”采用**完整 manifest 替换**语义（shadow 仅计算候选 manifest 哈希）。

### 运行时接口（草案）
- `World::step(n_ticks)`：推进时间并处理事件队列
- `World::apply_action(action)`：校验与入队事件
- `World::emit_effect(intent)`：校验 capability + policy → 入队
- `World::ingest_receipt(receipt)`：写入事件流并唤醒等待
- `World::snapshot()` / `World::restore()`：快照与恢复
- `World::create_snapshot()`：创建并记录快照
- `World::set_snapshot_retention(policy)`：快照保留策略
- `World::save_snapshot_to_dir(dir)` / `World::prune_snapshot_files(dir)`：快照文件落盘与清理
- `World::propose_manifest_update(manifest)`：提交治理提案
- `World::propose_manifest_patch(patch)`：以 patch 形式提交治理提案
- `World::shadow_proposal(id)`：影子运行并生成候选 hash
- `World::approve_proposal(id)`：审批或拒绝
- `World::apply_proposal(id)`：应用并更新 manifest
- `World::rollback_to_snapshot(snapshot, journal)`：回滚并记录审计事件
- `World::audit_events(filter)`：审计筛选（按类型/时间/因果）
- `World::save_audit_log(path, filter)`：导出审计事件到文件
- `diff_manifest(base, target)` / `merge_manifest_patches(base, patches)` / `merge_manifest_patches_with_conflicts(...)`：diff/merge 辅助
- `Scheduler::tick()`：按确定性顺序调度 agent cells
- `World::register_module_artifact(wasm_hash, bytes)`：写入模块工件
- `World::propose_module_changes(changes)`：提交模块变更提案（治理闭环）
- `World::module_registry()`：读取模块索引

### 代码结构调整（草案）

- `Manifest` 结构体新增 `module_changes: Option<ModuleChangeSet>`
- `ManifestPatch` 允许 `"/module_changes"` set/remove
- `GovernanceEvent::Applied` 增加 `module_changes` 字段（可选）
- 审计导出支持 `Module*Failed` 与 `ShadowReport` 记录

### Manifest 版本与迁移策略（草案）

- `manifest_version`：每份 manifest 显式携带版本号（如 `v1` / `v2`）。
- **向后兼容**：新字段默认值由加载器填充（如 `module_changes = None`）。
- **向前拒绝**：内核遇到高于自身支持的版本时拒绝加载并记录审计事件。
- **迁移路径**：提供 `migrate_manifest(from, to)` 辅助函数；迁移需可确定性重放。
- **Patch 约束**：`ManifestPatch` 必须基于同一 `base_manifest_hash`，跨版本 patch 直接拒绝。

**加载/恢复流程（示意）**
```
load_manifest()
  -> if version == supported: use
  -> else if version < supported: migrate_manifest(version, supported) -> new_manifest
  -> else: reject + audit
```

**base_manifest_hash 行为**
- 迁移后生成新的 `manifest_hash`，用于后续 patch 基线。
- 迁移前后的哈希需记录在审计事件中，保证可追溯。

**迁移审计事件（草案）**
```
ManifestMigrated {
  from_version,
  to_version,
  from_hash,
  to_hash,
  reason,
}
```

## 里程碑
- **M0**：方案与接口冻结（本设计 + 项目管理文档）
- **M1**：确定性 world kernel + 事件日志 + 最小快照
- **M2**：Effect/Receipt 路径 + capability + policy gate
- **M3**：Agent cells + 调度器 + 基础可观测性
- **M4**：受控升级（propose/shadow/approve/apply）最小闭环

## 风险
- **“所有优点”带来的复杂度**：治理、收据、能力边界会显著增加实现成本。
- **确定性与性能冲突**：单线程+事件重放可能成为瓶颈。
- **持久化膨胀**：日志与收据增长快，需要快照与归档策略。
- **治理摩擦**：过严的审批/策略可能降低迭代速度。
