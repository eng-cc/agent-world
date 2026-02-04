# Agent World Runtime：AOS 风格 world+agent 运行时（设计文档）

## 目标
- 在现有 `agent-world` 中实现一套 **world+agent 运行时**，借鉴 AgentOS 的关键优势：确定性、可审计、可回放、能力/政策边界、显式副作用与收据、受控升级。
- 让世界成为第一性：所有状态改变必须经由 **事件 → 规则校验 → 状态演化** 的统一路径，可追溯、可重放。
- 为后续规模化（多 Agent、高并发交互、长期运行）打下可演化的运行时基座。

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

### Agent 机制（已选）
- **Agent = Cell（keyed reducer）**：每个 agent 由同一 reducer 的 keyed 实例表示（key = `agent_id`），拥有独立状态与 mailbox；事件路由按 key 分发，调度器以确定性顺序轮询。

### Out of Scope（V1 不做）
- WASM 运行时与 AIR 完整规格实现（仅保留简化版 manifest）。
- 跨 world 协议与一致性（后续阶段考虑）。
- 复杂并行执行（保持单线程确定性）。
- 完整 UI/可视化工具链（仅保留 CLI/日志接口）。

## 接口 / 数据

### 核心概念
- **World**：事件日志 + 快照 + manifest + reducer 状态集合。
- **Agent Cell**：同一 reducer 的 keyed 实例（`agent_id` 为 key）。
- **Reducer**：纯函数式状态机（输入事件 + 旧状态 → 新状态 + Effect 意图）。
- **Effect / Receipt**：显式副作用与其回执；重放时只读取 receipt，不重新执行 I/O。
- **Capability / Policy**：运行时授权与治理规则。

### 关键数据结构（草案）
- `WorldEvent`：`{ id, time, kind, payload, caused_by }`
- `EffectIntent`：`{ intent_id, kind, params, cap_ref, origin }`
- `EffectReceipt`：`{ intent_id, status, payload, cost?, timestamps, hash }`
- `CapabilityGrant`：`{ name, cap_type, params, expiry? }`
- `PolicyRule`：`{ when, decision }`
- `Manifest`：`{ reducers, effects, caps, policies, routing, defaults }`
- `ManifestPatch`：`{ base_manifest_hash, ops[], new_version? }`，支持 set/remove（merge 要求基于同一 base hash）
- `PatchMergeResult`：`{ patch, conflicts[] }`，冲突包含路径与涉及的 patch 索引
- `PatchConflict`：`{ path, kind, patches[], ops[] }`（kind: same_path/prefix_overlap）
- `Proposal`：`{ id, author, base_manifest_hash, manifest, status }`
- `GovernanceEvent`：`Proposed/ShadowReport/Approved/Applied`
- `RollbackEvent`：`{ snapshot_hash, snapshot_journal_len, prior_journal_len, reason }`
- `SnapshotCatalog`：`{ records[], retention }`
- `SnapshotRetentionPolicy`：`{ max_snapshots }`
- `AuditFilter`：`{ kinds?, from_time?, to_time?, from_event_id?, to_event_id?, caused_by? }`

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
