# world-runtime PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_RUNTIME-001 (PRD-WORLD_RUNTIME-001) [test_tier_required]: 完成 world-runtime PRD 改写，建立运行时设计主入口。
- [ ] TASK-WORLD_RUNTIME-002 (PRD-WORLD_RUNTIME-001/002) [test_tier_required]: 补齐 runtime 核心边界（确定性、WASM、治理）验收清单。
- [ ] TASK-WORLD_RUNTIME-003 (PRD-WORLD_RUNTIME-002/003) [test_tier_required]: 建立运行时安全与数值语义回归跟踪模板。
- [ ] TASK-WORLD_RUNTIME-004 (PRD-WORLD_RUNTIME-003) [test_tier_required]: 对接跨模块发布门禁中的 runtime 质量指标。
- [x] TASK-WORLD_RUNTIME-005 (PRD-WORLD_RUNTIME-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-WORLD_RUNTIME-006 (PRD-WORLD_RUNTIME-002) [test_tier_required]: 同步 m1/m5 builtin wasm 工件 `sha256` 与 identity manifest，修复 CI hash token 不一致导致的运行时加载失败；回归 `env -u RUSTC_WRAPPER cargo test -p agent_world --tests --features test_tier_required`。
- [x] TASK-WORLD_RUNTIME-016 (PRD-WORLD_RUNTIME-016/017/018) [test_tier_required]: 新增“线上模块发布合法性闭环补齐”专题 PRD/项目管理文档并纳入主索引。
- [x] TASK-WORLD_RUNTIME-017 (PRD-WORLD_RUNTIME-016) [test_tier_required]: 引入线上 builtin 发布清单入口与生产禁 fallback 策略（`ReleaseSecurityPolicy` + online manifest API）。
- [x] TASK-WORLD_RUNTIME-018 (PRD-WORLD_RUNTIME-016) [test_tier_required]: `m1/m4/m5` bootstrap 加载迁移到治理清单解析路径，保留受控 fallback。
- [x] TASK-WORLD_RUNTIME-019 (PRD-WORLD_RUNTIME-016) [test_tier_full]: 补齐线上 manifest 不可达/回滚/版本漂移场景回归与故障签名。
- [x] TASK-WORLD_RUNTIME-020 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 生产策略下禁用 `identity_hash_v1` 回退并补齐回归。
- [x] TASK-WORLD_RUNTIME-021 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: `apply_proposal` 去本地自签路径，改为外部 finality 证书必需并补齐 epoch 快照验证者签名集阈值与轮换回归。
- [x] TASK-WORLD_RUNTIME-022 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 新增去中心化发布提案与复构建证明收集流程（`proposal -> attestation`）并形成可审计证据结构。
- [x] TASK-WORLD_RUNTIME-023 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 落地“epoch 快照验证者签名集”阈值签名聚合与 release manifest 激活路径（不依赖 CI 服务）并补齐拒绝路径测试。
- [x] TASK-WORLD_RUNTIME-024 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 更新发布运行手册与告警策略（证明冲突、阈值不足、manifest 不可达），并明确 CI 仅用于开发回归且不承担生产发布写入。
- [ ] TASK-WORLD_RUNTIME-025 (PRD-WORLD_RUNTIME-017) [test_tier_required + test_tier_full]: 扩展 finality 证书/信任根数据模型，落地 `epoch_id + validator_set_hash + stake_root + threshold_bps + min_unique_signers` 校验与回归。
- [ ] TASK-WORLD_RUNTIME-026 (PRD-WORLD_RUNTIME-017) [test_tier_required]: 梳理安装/升级/回滚/发布应用调用点，生产路径禁止本地自签 `apply_proposal()`，统一切换外部证书 apply。
- [x] TASK-WORLD_RUNTIME-027 (PRD-WORLD_RUNTIME-016) [test_tier_required]: `ModuleRelease* -> release manifest` 映射状态落盘并补齐回放断言。
- [ ] TASK-WORLD_RUNTIME-028 (PRD-WORLD_RUNTIME-018) [test_tier_required]: 从主 CI 移除生产发布写入/激活职责，仅保留 `--check` 类回归；补齐节点侧发布运行手册与验收脚本。
- [ ] TASK-WORLD_RUNTIME-029 (PRD-WORLD_RUNTIME-018) [test_tier_required + test_tier_full]: 增加 `stake/epoch` 验签耗时与“2 epoch 收敛”固定基准入口，产出可归档性能与收敛报告。

## 依赖
- doc/world-runtime/prd.index.md
- `doc/world-runtime/runtime/runtime-integration.md`
- `doc/world-runtime/wasm/wasm-interface.md`
- `doc/world-runtime/governance/governance-events.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-08
- 当前状态: active
- 下一任务: TASK-WORLD_RUNTIME-025
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 world-runtime 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md` 与后续当日日志。
