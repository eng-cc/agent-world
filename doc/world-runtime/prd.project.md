# world-runtime PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_RUNTIME-001 (PRD-WORLD_RUNTIME-001) [test_tier_required]: 完成 world-runtime PRD 改写，建立运行时设计主入口。
- [ ] TASK-WORLD_RUNTIME-002 (PRD-WORLD_RUNTIME-001/002) [test_tier_required]: 补齐 runtime 核心边界（确定性、WASM、治理）验收清单。
- [ ] TASK-WORLD_RUNTIME-003 (PRD-WORLD_RUNTIME-002/003) [test_tier_required]: 建立运行时安全与数值语义回归跟踪模板。
- [ ] TASK-WORLD_RUNTIME-004 (PRD-WORLD_RUNTIME-003) [test_tier_required]: 对接跨模块发布门禁中的 runtime 质量指标。
- [x] TASK-WORLD_RUNTIME-005 (PRD-WORLD_RUNTIME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-WORLD_RUNTIME-006 (PRD-WORLD_RUNTIME-002) [test_tier_required]: 同步 m1/m5 builtin wasm 工件 `sha256` 与 identity manifest，修复 CI hash token 不一致导致的运行时加载失败；回归 `env -u RUSTC_WRAPPER cargo test -p agent_world --tests --features test_tier_required`。
<<<<<<< HEAD
- [x] TASK-WORLD_RUNTIME-016 (PRD-WORLD_RUNTIME-016/017/018) [test_tier_required]: 新增“线上模块发布合法性闭环补齐”专题 PRD/项目管理文档并纳入主索引。
- [x] TASK-WORLD_RUNTIME-017 (PRD-WORLD_RUNTIME-016) [test_tier_required]: 引入线上 builtin 发布清单入口与生产禁 fallback 策略（`ReleaseSecurityPolicy` + online manifest API）。
- [x] TASK-WORLD_RUNTIME-018 (PRD-WORLD_RUNTIME-016) [test_tier_required]: `m1/m4/m5` bootstrap 加载迁移到治理清单解析路径，保留受控 fallback。
- [x] TASK-WORLD_RUNTIME-019 (PRD-WORLD_RUNTIME-016) [test_tier_full]: 补齐线上 manifest 不可达/回滚/版本漂移场景回归与故障签名。
- [x] TASK-WORLD_RUNTIME-020 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 生产策略下禁用 `identity_hash_v1` 回退并补齐回归。
- [x] TASK-WORLD_RUNTIME-021 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: `apply_proposal` 去本地自签路径，改为外部 finality 证书必需并补齐 epoch 快照验证者签名集阈值与轮换回归。
- [x] TASK-WORLD_RUNTIME-022 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 新增去中心化发布提案与复构建证明收集流程（`proposal -> attestation`）并形成可审计证据结构。
- [x] TASK-WORLD_RUNTIME-023 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 落地“epoch 快照验证者签名集”阈值签名聚合与 release manifest 激活路径（不依赖 CI 服务）并补齐拒绝路径测试。
- [x] TASK-WORLD_RUNTIME-024 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 更新发布运行手册与告警策略（证明冲突、阈值不足、manifest 不可达），并明确 CI 仅用于开发回归且不承担生产发布写入。
- [x] TASK-WORLD_RUNTIME-025 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: 扩展 finality 证书/信任根数据模型，落地 `epoch_id + validator_set_hash + stake_root + threshold_bps + min_unique_signers` 校验与回归。
- [x] TASK-WORLD_RUNTIME-026 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 梳理安装/升级/回滚/发布应用调用点，生产路径禁止本地自签 `apply_proposal()`，统一切换外部证书 apply。
- [x] TASK-WORLD_RUNTIME-027 (PRD-WORLD_RUNTIME-016) [test_tier_required]: `ModuleRelease* -> release manifest` 映射状态落盘并补齐回放断言。
- [x] TASK-WORLD_RUNTIME-028 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 从主 CI 移除生产发布写入/激活职责，仅保留 `--check` 类回归；补齐节点侧发布运行手册与验收脚本。
- [x] TASK-WORLD_RUNTIME-029 (PRD-WORLD_RUNTIME-018) [test_tier_required + test_tier_full]: 增加 `stake/epoch` 验签耗时与“2 epoch 收敛”固定基准入口，产出可归档性能与收敛报告。
- [x] TASK-WORLD_RUNTIME-030 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 建立运行态存储体积治理专题 PRD / project，并回写模块主 PRD、项目文档与索引。
- [x] TASK-WORLD_RUNTIME-031 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 落地 execution bridge / execution world retention policy（head window、稀疏 checkpoint、manifest-aware GC）并验证 latest-state 恢复不回退。
- [ ] TASK-WORLD_RUNTIME-032 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 实现 `tick_consensus_records` 热冷分层与 storage metrics/status 输出，建立 snapshot size regression 与 archive read 回归。
- [ ] TASK-WORLD_RUNTIME-033 (PRD-WORLD_RUNTIME-014/015) [test_tier_required + test_tier_full]: 建立 launcher / chain runtime / soak profile 的 footprint gate、GC fail-safe 与重启恢复联合验证。
- [x] TASK-WORLD_RUNTIME-034 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 输出详细技术设计文档，明确 canonical replay log / checkpoint / GC / metrics / migration 方案。
- [x] TASK-WORLD_RUNTIME-035 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 将专题项目进一步拆解为 T1.1 ~ T7.5 子任务，明确执行顺序、依赖边界与测试闭环。
=======
- [x] TASK-WORLD_RUNTIME-016 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 建立运行态存储体积治理专题 PRD / project，并回写模块主 PRD、项目文档与索引。
- [x] TASK-WORLD_RUNTIME-017 (PRD-WORLD_RUNTIME-013/014) [test_tier_required]: 落地 execution bridge / execution world retention policy（head window、稀疏 checkpoint、manifest-aware GC）并验证 latest-state 恢复不回退。
- [ ] TASK-WORLD_RUNTIME-018 (PRD-WORLD_RUNTIME-013/015) [test_tier_required]: 实现 `tick_consensus_records` 热冷分层与 storage metrics/status 输出，建立 snapshot size regression 与 archive read 回归。
- [ ] TASK-WORLD_RUNTIME-019 (PRD-WORLD_RUNTIME-014/015) [test_tier_required + test_tier_full]: 建立 launcher / chain runtime / soak profile 的 footprint gate、GC fail-safe 与重启恢复联合验证。
- [x] TASK-WORLD_RUNTIME-020 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 输出详细技术设计文档，明确 canonical replay log / checkpoint / GC / metrics / migration 方案。
- [x] TASK-WORLD_RUNTIME-021 (PRD-WORLD_RUNTIME-013/014/015) [test_tier_required]: 将专题项目进一步拆解为 T1.1 ~ T7.5 子任务，明确执行顺序、依赖边界与测试闭环。
>>>>>>> 53758185 (feat(world-runtime): recover interrupted sidecar saves)

## 依赖
- doc/world-runtime/prd.index.md
- `doc/world-runtime/runtime/runtime-integration.md`
- `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.prd.md`
- `doc/world-runtime/wasm/wasm-interface.md`
- `doc/world-runtime/governance/governance-events.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-08
- 当前状态: active
- 下一任务: TASK-WORLD_RUNTIME-032（进入专题子任务 T5.1）
- 实施备注:
  - `TASK-WORLD_RUNTIME-028` 已完成：新增节点侧固定验收入口 `scripts/module-release-node-acceptance.sh` 并将 S11 运行手册切换为“脚本入口 + 等价拆分命令 + 证据目录”；同时收敛 `sync-m1/m4/m5` 非 `--check` 写入授权为“CI 禁止、仅本地显式授权（`AGENT_WORLD_WASM_SYNC_WRITE_ALLOW=local-dev`）”，主 CI 不再具备生产发布写入/激活路径。
  - `TASK-WORLD_RUNTIME-029` 已完成：新增 `scripts/world-runtime-finality-baseline.sh` 固定基准入口，输出 `stake/epoch` 验签耗时聚合指标与 `2 epoch` 收敛状态（`summary.md`/`summary.json` 可归档）；S11 运行手册已补齐命令与产物路径。
  - `TASK-WORLD_RUNTIME-034` 已完成：补齐 `runtime-storage-footprint-governance-2026-03-08.design.md`，明确 replay contract、checkpoint、GC、metrics 与迁移边界。
  - `TASK-WORLD_RUNTIME-035` 已完成：将专题执行拆解到 T1.1 ~ T7.5，明确实现顺序、依赖边界与测试闭环。
  - `TASK-WORLD_RUNTIME-031` 已启动并完成 T1.1：execution bridge record 已升级为 V2 schema，并具备 legacy 兼容读取。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T1.2：checkpoint manifest 的目录布局、latest 指针与 hash/height 校验已落地。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T1.3：replay planner 已支持“最近 checkpoint + 本地 execution records”与无 checkpoint 全日志回放降级。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T1.4：external effect materialization 已通过 `external_effect_ref` 落 CAS，并在 replay plan 构建时执行 fail-closed 校验。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T1.5：retained-height replay / no-checkpoint fallback / replay mismatch / checkpoint corruption 定向测试已补齐。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T2.1：execution bridge 已按 latest head + hot window 重算 CAS pin set，历史 snapshot/journal 不再默认全量固定。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T2.2：sparse checkpoint cadence、latest pointer 与旧 checkpoint record 回写已接入 execution bridge 写路径。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T2.3：archive-only / checkpoint-only heights 的 snapshot/journal refs 会被压缩回写，随后按 pin set sweep orphan blobs。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T2.4：legacy V1 record 现支持按需升 V2 写回，legacy 样本会自动进入 safe-mode 禁 aggressive sweep。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T2.5：head-window retention / sparse checkpoint / restart recovery / dangling-ref 拒绝回归已补齐。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T3.1：sidecar generation index 与 generation pin 集已落到 `.distfs-state/sidecar-generations/` 元数据。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T3.2：`save_to_dir` 已接入 staging -> latest/rollback-safe 的 sidecar generation 两阶段切换，并限制 generation metadata 至少保留 2 代。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T3.3：sidecar sweep 已改为 manifest-aware blob 清扫；成功路径会把 `.distfs-state/blobs` 收敛到 latest/rollback-safe 引用集合，GC 失败则仅记录 `last_gc_result=failed` 并保留恢复数据。
  - `TASK-WORLD_RUNTIME-031` 已继续完成 T3.4：sidecar save 现会在 staging 成功提交后再刷新 root latest manifest/journal，且重试前会清理未提交的 `generation.tmp`；故障注入测试已覆盖中断回滚、部分 staging 写入与 orphan cleanup。
  - `TASK-WORLD_RUNTIME-032` 已启动并完成 T4.1：默认保存链路会把 `tick_consensus_records` 拆成热快照 + `tick-consensus.archive.json` 冷归档，并通过热区摘要字段保证恢复时能校验归档是否齐全。
  - `TASK-WORLD_RUNTIME-032` 已继续完成 T4.2：冷归档已升级为 `tick-consensus.archive.index.json` + `tick-consensus.archive.segments/`，每段记录 `from/to tick`、`content_hash`、`record_count`、`hash_chain_anchor` 与相对路径。
  - `TASK-WORLD_RUNTIME-032` 已继续完成 T4.3：新增显式 archive range read / verify 路径，并能在 index 缺失时回退读取 T4.1 legacy 单文件 archive，保证旧样本迁移可用。
  - `TASK-WORLD_RUNTIME-032` 已继续完成 T4.4：snapshot size regression / archive range read / legacy migration / tampered segment hash verify 回归已补齐，T4 系列任务已闭环。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 world-runtime 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`、`doc/devlog/2026-03-06.md` 与 `doc/devlog/2026-03-08.md`。
