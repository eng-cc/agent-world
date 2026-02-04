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
- `Manifest`：`{ reducers, modules, effects, caps, policies, routing, defaults }`
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
