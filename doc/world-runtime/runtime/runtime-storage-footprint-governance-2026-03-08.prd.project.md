# Agent World Runtime：执行桥接与运行态存储体积治理（项目管理文档）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
### T0 建档与文档树接线
- [x] T0 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 新建专题 PRD / project，并回写 `doc/world-runtime/prd.md`、`doc/world-runtime/prd.project.md`、`doc/world-runtime/prd.index.md` 的映射关系。
- [x] T0.1 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 输出详细技术设计文档 `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.design.md`，明确 canonical replay log / checkpoint / GC / metrics / migration 方案。

### T1 Canonical replay contract
- [x] T1.1 (PRD-WORLD_RUNTIME-014) [test_tier_required]: 定义 `ExecutionBridgeRecordV2` 持久化字段与兼容读取策略，明确 `commit_log_ref` / `checkpoint_ref` / `latest_state_ref` / `external_effect_ref` 的角色边界。
- [ ] T1.2 (PRD-WORLD_RUNTIME-014) [test_tier_required]: 定义 `ExecutionCheckpointManifest` 目录布局、pin 语义、hash/height 校验字段与 reader / writer 原子切换规则。
- [ ] T1.3 (PRD-WORLD_RUNTIME-014) [test_tier_required]: 在 replay planner 中实现“最近 checkpoint + commit log”重建路径，并显式处理无 checkpoint 的全日志回放降级。
- [ ] T1.4 (PRD-WORLD_RUNTIME-014) [test_tier_required]: 明确外部非确定性 effect 的 materialization contract，保证 replay 输入可闭包且 mismatch 时 fail-closed。
- [ ] T1.5 (PRD-WORLD_RUNTIME-014) [test_tier_required]: 补齐 retained-height replay / no-checkpoint fallback / replay mismatch / checkpoint corruption 定向测试。

### T2 Execution bridge retention
- [ ] T2.1 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 在 `execution_bridge.rs` 实现 latest head + hot window pin set 计算，不再为每个 committed height 默认固定完整 `snapshot_ref`。
- [ ] T2.2 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 落地 sparse checkpoint cadence 与 pin set 计算，保证稀疏高度可直接跳转恢复。
- [ ] T2.3 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 基于显式 pin set sweep 历史 `snapshot_ref` / `journal_ref`，删除后不得留下 dangling refs。
- [ ] T2.4 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 完成 `ExecutionBridgeRecordV1 -> V2` 向后兼容读取与渐进迁移，保证旧样本可读且不强制一次性重写。
- [ ] T2.5 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 补齐 head-window retention、稀疏 checkpoint、restart recovery、dangling-ref 拒绝回归测试。

### T3 Sidecar generation GC
- [ ] T3.1 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 定义 `SidecarGenerationIndex` 目录布局、manifest 字段与 generation pin 集，区分 staging / latest / rollback-safe generation。
- [ ] T3.2 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 在 `save_to_dir` 落地两阶段 generation 切换，确保 latest generation 原子更新且最少保留 `keep=2`。
- [ ] T3.3 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 实现 manifest-aware sweep，successful save 后孤儿 blob 数量为 `0`，失败时不得删除仍被 latest/rollback generation 引用的数据。
- [ ] T3.4 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 补齐 save 中断、manifest 部分写入、rollback 恢复与 orphan cleanup 故障注入测试。

### T4 Tick consensus 热冷分层
- [ ] T4.1 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 将 `tick_consensus_records` 从热快照拆分为热摘要 + 冷归档，控制 `snapshot.json` 默认体积。
- [ ] T4.2 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 定义 `TickConsensusArchiveIndex` 的 anchor / hash chain / range metadata，保证审计与校验可顺序读取。
- [ ] T4.3 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 落地 archive read / verify 路径与旧快照迁移逻辑，保证冷热分层后查询与验证语义不变。
- [ ] T4.4 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 补齐 snapshot size regression、archive read、hash verify、旧样本迁移回归测试。

### T5 冷数据索引语义收敛
- [ ] T5.1 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 收敛 `execution_records` 与 `replication_commit_messages` 的热/冷窗口口径，统一“热窗口 + 稀疏冷索引 + 归档读回”语义。
- [ ] T5.2 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 定义共享 cold index 命名、目录布局、元数据字段与 range anchor 规则，避免不同子系统各自发明目录协议。
- [ ] T5.3 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 完成旧目录布局兼容读取 / 别名迁移，保证已有样本与工具脚本不立即失效。
- [ ] T5.4 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 补齐 cold-index scan、archive seek、跨模块读回一致性回归测试。

### T6 Metrics / profile / launcher 透传
- [ ] T6.1 (PRD-WORLD_RUNTIME-015) [test_tier_required]: 落地 `StorageProfileConfig` 解析、默认值与 runtime / launcher / script 的统一透传入口。
- [ ] T6.2 (PRD-WORLD_RUNTIME-015) [test_tier_required]: 在 runtime status / state file 中输出 `StorageMetricsSnapshot`，覆盖 bytes、ref_count、pin_count、checkpoint_count、orphan_count、gc_last_result 等最低字段。
- [ ] T6.3 (PRD-WORLD_RUNTIME-015) [test_tier_required]: 补齐 GC 最近结果、失败原因、profile、生效预算与回放能力摘要字段，保证脚本/launcher 无需读取内部目录即可判断状态。
- [ ] T6.4 (PRD-WORLD_RUNTIME-015) [test_tier_required]: 对齐 launcher、`run-web-launcher.sh`、chain runtime 启动脚本的 profile 参数透传与默认口径，避免环境间语义漂移。
- [ ] T6.5 (PRD-WORLD_RUNTIME-015) [test_tier_required]: 补齐 profile 参数、status 输出、错误字段、launcher 透传的定向测试。

### T7 Footprint gate / 回归 / 收口
- [ ] T7.1 (PRD-WORLD_RUNTIME-014/015) [test_tier_full]: 构造 `>= 2500` heights 的可复现实验样本，作为 footprint gate 与 replay regression 的统一输入基线。
- [ ] T7.2 (PRD-WORLD_RUNTIME-014/015) [test_tier_required]: 建立默认 profile 的体积预算、restart recovery、retained-height replay gate，并输出失败时的目录/指标差异。
- [ ] T7.3 (PRD-WORLD_RUNTIME-014/015) [test_tier_full]: 建立 GC fail-safe、profile 切换、archive read、checkpoint corruption、replay mismatch 的全量回归套件。
- [ ] T7.4 (PRD-WORLD_RUNTIME-014/015) [test_tier_full]: 对接 launcher / chain runtime / soak 场景，验证 `dev_local`、`release_default`、`soak_forensics` 三档 profile 口径一致。
- [ ] T7.5 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 回写专题 PRD / project、模块项目文档、`testing-manual.md`（如测试入口变化）与 `doc/devlog/2026-03-08.md`，归档体积对比与回放验证结论。

## 执行顺序与依赖
- M1（契约冻结）: 先完成 T1.1 ~ T1.4，冻结 replay truth-source、checkpoint manifest 与外部 effect contract；T2 / T3 / T6 以此为前置。
- M2（写路径与 GC）: 再完成 T2.1 ~ T2.4 与 T3.1 ~ T3.3，优先解决 execution bridge 历史 refs 与 sidecar blob 无界增长。
- M3（冷热分层与观测）: 在 T4.1 ~ T5.3 与 T6.1 ~ T6.4 中统一 cold index / archive / metrics 语义，避免各子系统重复定义目录协议。
- M4（测试与收口）: T1.5、T2.5、T3.4、T4.4、T5.4、T6.5 与 T7.1 ~ T7.5 作为统一验证和回写阶段；未经 gate 通过不得切换默认 profile。
- 并行边界: T2 与 T3 可在 T1 完成后并行；T4 可与 T2/T3 并行推进，但 T5 / T6 需等待冷热目录语义稳定后再收口。

## 依赖
- `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.prd.md`
- `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.design.md`
- `doc/world-runtime/prd.md`
- `doc/world-runtime/prd.project.md`
- `doc/world-runtime/prd.index.md`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part2.rs`
- `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/runtime/world/persistence.rs`
- `crates/agent_world/src/runtime/snapshot.rs`
- `crates/agent_world_node/src/replication.rs`
- `crates/agent_world_distfs/src/lib.rs`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-08
- 当前状态: planning
- 已完成: T0、T0.1、T1.1
- 已拆解待执行: T1.1 ~ T7.5
- 进行中: T1.2
- 阻塞项: 无；但 T2 / T3 / T6 / T7 的实现必须以前置 T1 契约冻结为准。
- 下一任务: T1.2（定义 `ExecutionCheckpointManifest` 目录布局、pin 语义与原子切换规则）
