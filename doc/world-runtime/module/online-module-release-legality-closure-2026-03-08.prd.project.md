# Agent World Runtime：线上模块发布合法性闭环补齐（项目管理文档）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_RUNTIME-016 (PRD-WORLD_RUNTIME-016/017/018) [test_tier_required]: 建立专题 PRD/项目管理文档，冻结目标态边界与验收口径。
- [x] TASK-WORLD_RUNTIME-017 (PRD-WORLD_RUNTIME-016) [test_tier_required]: 引入线上发布清单主真源与节点加载策略切换（生产禁用本地未授权 fallback）。
- [x] TASK-WORLD_RUNTIME-018 (PRD-WORLD_RUNTIME-016) [test_tier_required]: 将 `m1/m4/m5` builtin artifact 加载从仓库内置清单迁移到可治理清单入口，保留受控 bootstrap 兜底。
- [x] TASK-WORLD_RUNTIME-019 (PRD-WORLD_RUNTIME-016) [test_tier_full]: 补齐线上 manifest 不可达/回滚/版本漂移场景回归与故障签名。
- [x] TASK-WORLD_RUNTIME-020 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 生产策略下禁用 `identity_hash_v1` 回退，强制 artifact ed25519 签名校验。
- [x] TASK-WORLD_RUNTIME-021 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: `apply_proposal` 去本地自签路径，改为外部 finality 证书必需并补齐“epoch 快照验证者签名集阈值最终性”与轮换回归。
- [ ] TASK-WORLD_RUNTIME-022 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 新增去中心化发布提案与复构建证明收集流程（`proposal -> attestation`）并形成可审计证据结构。
- [ ] TASK-WORLD_RUNTIME-023 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 落地“epoch 快照验证者签名集”阈值签名聚合与 release manifest 激活路径（不依赖 CI 服务）并补齐拒绝路径测试。
- [ ] TASK-WORLD_RUNTIME-024 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 更新发布运行手册与告警策略（证明冲突、阈值不足、manifest 不可达），并明确 CI 仅用于开发回归且不承担生产发布写入。
- [ ] TASK-WORLD_RUNTIME-025 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: 扩展 finality 证书/信任根数据模型，落地 `epoch_id + validator_set_hash + stake_root + threshold_bps + min_unique_signers` 校验与回归。
- [ ] TASK-WORLD_RUNTIME-026 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 梳理安装/升级/回滚/发布应用调用点，生产路径禁止本地自签 `apply_proposal()`，统一切换外部证书 apply。
- [x] TASK-WORLD_RUNTIME-027 (PRD-WORLD_RUNTIME-016) [test_tier_required]: 为现有 `ModuleRelease*` 状态机补齐 `release manifest` 映射关系与回放一致性测试，保障迁移期兼容。
- [ ] TASK-WORLD_RUNTIME-028 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 从主 CI 移除生产发布写入/激活职责，仅保留 `--check` 类回归；补齐节点侧发布运行手册与验收脚本。
- [ ] TASK-WORLD_RUNTIME-029 (PRD-WORLD_RUNTIME-018) [test_tier_required + test_tier_full]: 增加 `stake/epoch` 验签耗时与“2 epoch 收敛”固定基准入口，产出可归档性能与收敛报告。

## 依赖
- `doc/world-runtime/module/online-module-release-legality-closure-2026-03-08.prd.md`
- `doc/world-runtime/module/player-published-entities-2026-03-05.prd.md`
- `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md`
- `doc/p2p/distributed/distributed-pos-consensus.prd.md`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-08
- 当前状态: active
- 下一任务: TASK-WORLD_RUNTIME-022
- 实施备注:
  - `TASK-WORLD_RUNTIME-019` 已完成：新增故障签名 `builtin_release_manifest_unreachable`、`builtin_release_manifest_missing_or_rolled_back`、`builtin_release_manifest_identity_drift`，并补齐 `test_tier_full` 回归。
  - `TASK-WORLD_RUNTIME-021` 已完成：finality 校验新增按 epoch 快照阈值与 signer 集校验，拒绝快照外 signer，并补齐轮换回归（旧 signer 拒绝、新 signer 通过）。
