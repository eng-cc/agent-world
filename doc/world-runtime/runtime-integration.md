# Agent World Runtime：运行时集成要点（设计分册）

本分册为 `doc/world-runtime.md` 的详细展开。

## 模块加载与缓存（草案）

- **加载键**：仅允许按 `wasm_hash` 加载，拒绝同名覆盖。
- **缓存策略**：LRU + `max_cached_modules` 上限；超限时淘汰最久未使用模块。
- **编译缓存**：可选缓存已编译实例（WASM → 本地可执行表示）。
- **失败事件**：加载失败写入 `ModuleLoadFailed`（含 hash/原因）。

## 沙箱执行器与资源限制（草案）

- **资源限额**：`max_mem_bytes`、`max_gas`、`max_call_rate`、`max_output_bytes`。
- **隔离**：模块不可直接访问 I/O，仅能产生 `EffectIntent`。
- **超限处理**：超限触发 `ModuleCallFailed`（code=TIMEOUT/OUTPUT_TOO_LARGE/EFFECT_LIMIT_EXCEEDED）。
- **执行入口**：`World::execute_module_call` 通过沙箱接口执行模块并返回 `ModuleOutput`。

## Capability/Policy 绑定（草案）

- **绑定点**：模块输出的每个 `EffectIntent` 必须关联 `cap_ref`。
- **校验**：`required_caps` 与系统 grants 匹配；policy 允许才执行。
- **审计**：策略拒绝记录为 `PolicyDecisionRecorded`。

## 事件订阅与路由（草案）

- **订阅来源**：`ModuleManifest.subscriptions` 指定 event/action kinds。
- **路由顺序**：按 `module_id` 字典序调用，保证确定性。
- **隔离性**：模块之间不共享状态，状态由 reducer 自身维护。
- **事件 kind 命名**：`domain.agent_registered`、`domain.agent_moved`、`domain.action_rejected` 等；其它系统事件使用 `effect.*`/`module.*`/`snapshot.*`/`manifest.*` 前缀。

## 模块输出校验（草案）

- **数量限制**：`effects`/`emits` 数量不得超过 `ModuleLimits`。
- **大小限制**：`ModuleOutput` 编码后大小不得超过 `max_output_bytes`。
- **拒绝策略**：违反规则写入 `ModuleCallFailed` 并丢弃输出。

## 运行时接口（草案）
- `World::step(n_ticks)`：推进时间并处理事件队列
- `World::step_with_modules(sandbox)`：推进时间并处理事件队列，同时路由事件到模块
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
- `World::load_module(wasm_hash)`：按哈希加载模块（命中缓存或从工件库读取）
- `World::set_module_cache_max(max_cached_modules)`：调整模块缓存容量
- `World::set_module_limits_max(limits)`：调整模块资源上限
- `World::execute_module_call(module_id, trace_id, input, sandbox)`：执行模块调用并写入 ModuleCallFailed/ModuleEmitted
- `World::route_event_to_modules(event, sandbox)`：按订阅路由事件并触发模块调用
- `World::propose_module_changes(changes)`：提交模块变更提案（治理闭环）
- `World::module_registry()`：读取模块索引

## 代码结构调整（草案）

- `Manifest` 结构体新增 `module_changes: Option<ModuleChangeSet>`
- `ManifestPatch` 允许 `"/module_changes"` set/remove
- `GovernanceEvent::Applied` 增加 `module_changes` 字段（可选）
- 审计导出支持 `Module*Failed` 与 `ShadowReport` 记录
