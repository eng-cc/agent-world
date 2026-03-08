# p2p PRD Project

审计轮次: 10

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-001 (PRD-P2P-001) [test_tier_required]: 完成 p2p PRD 改写，建立分布式系统设计入口。
- [x] TASK-P2P-002 (PRD-P2P-001/002) [test_tier_required]: 补齐网络/共识/DistFS 三线联合验收清单。
- [x] TASK-P2P-003 (PRD-P2P-002/003) [test_tier_required]: 建立 S9/S10 长跑结果与缺陷闭环模板。
- [x] TASK-P2P-004 (PRD-P2P-003) [test_tier_required]: 对接发行门禁中的分布式质量指标。
- [x] TASK-P2P-005 (PRD-P2P-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-P2P-006 (PRD-P2P-004) [test_tier_required]: 输出“手机轻客户端权威状态”专题 PRD 与项目管理文档，并回写模块主索引链路。
- [x] TASK-P2P-007 (PRD-P2P-004) [test_tier_required + test_tier_full]: 实现 intent-only 接入、finality UI、challenge/reconnect 闭环并补齐回归证据。
- [x] TASK-P2P-008 (PRD-P2P-005) [test_tier_required + test_tier_full]: 实现 PoS 固定时间槽（slot/epoch）真实时钟驱动、漏槽计数与时间窗口校验，并补齐回归证据。
- [x] TASK-P2P-009 (PRD-P2P-006) [test_tier_required + test_tier_full]: 实现 PoS 槽内 tick 相位门控（`ticks_per_slot`）与动态节拍调度，并补齐回归证据。
- [x] TASK-P2P-010 (PRD-P2P-007) [test_tier_required + test_tier_full]: 对齐 PoS 时间锚定控制面参数与可观测口径（runtime/viewer/launcher/scripts）。
- [x] TASK-P2P-011 (PRD-P2P-008) [test_tier_required]: 收敛 PoS 时间锚定残留语义偏差（状态字段命名、launcher 校验文案、viewer/manual/site 与 `world_viewer_live` 实际能力对齐）。
- [x] TASK-P2P-012 (PRD-P2P-009) [test_tier_required]: 修正默认 PoS 时间参数与校验文案残留偏差，确保默认启动即符合“tick 锚定出块时间”口径。
- [x] TASK-P2P-013 (PRD-P2P-009) [test_tier_required]: 将默认 PoS 时间参数对齐到“12s 出块、每块 10 tick”基线（`12000/10/9`），并收敛 CLI/脚本/文档一致性。
- [ ] TASK-P2P-014 (PRD-P2P-010) [test_tier_required]: 移除 `world_viewer_live --release-config/--node-*` 控制面参数，收敛为纯观察服务并完成文档/示例/测试闭环。

### TASK-P2P-002 执行拆解（PRD-P2P-001/002）
- [x] TASK-P2P-002-A [test_tier_required]: 在 `doc/p2p/prd.md` 补齐网络/共识/DistFS 三线联合验收清单（基线命令、门禁命令、阻断条件、证据产物）。
- [x] TASK-P2P-002-B [test_tier_required]: 在主 PRD 的 Acceptance Criteria 与 Decision Log 增加联合验收清单的验收口径与决策依据。
- [x] TASK-P2P-002-C [test_tier_required]: 执行文档门禁并回写项目状态，确保任务与测试层级可追溯。

### TASK-P2P-003 执行拆解（PRD-P2P-002/003）
- [x] TASK-P2P-003-A [test_tier_required]: 在 `doc/p2p/prd.md` 新增 S9/S10 长跑结果模板，统一运行记录字段与证据来源。
- [x] TASK-P2P-003-B [test_tier_required]: 在 `doc/p2p/prd.md` 新增缺陷闭环模板，定义 `incident_id -> fix_task -> fix_commit -> regression_command` 链路。
- [x] TASK-P2P-003-C [test_tier_required]: 在主 PRD 增补 AC/NFR 并回写项目状态，确保长跑失败不可静默。

### TASK-P2P-004 执行拆解（PRD-P2P-003）
- [x] TASK-P2P-004-A [test_tier_required]: 在 `doc/p2p/prd.md` 新增发行门禁分布式质量指标映射表（指标、数据源、阈值、阻断策略、责任归属）。
- [x] TASK-P2P-004-B [test_tier_required]: 在主 PRD 增补 `AC-10`、`NFR-P2P-10` 与 `DEC-P2P-008`，固化“指标硬阻断”口径。
- [x] TASK-P2P-004-C [test_tier_required]: 执行 `release-gate` 干跑与文档门禁检查，回写项目状态与追踪链路。

### TASK-P2P-010 执行拆解（PRD-P2P-007）
- [x] TASK-P2P-010-T0 [test_tier_required]: 新增专题 PRD / project 文档并回写 `doc/p2p/prd.md`、`doc/p2p/prd.project.md`、`doc/p2p/prd.index.md` 映射。
- [x] TASK-P2P-010-T1 [test_tier_required]: `world_chain_runtime/world_viewer_live` 暴露并校验 `slot_duration_ms/ticks_per_slot/proposal_tick_phase/adaptive_tick_scheduler_enabled/slot_clock_genesis_unix_ms/max_past_slot_lag`，并明确 `node_tick_ms` 为轮询间隔。
- [x] TASK-P2P-010-T2 [test_tier_required]: launcher UI/配置字段与参数透传对齐新口径，补齐校验与错误提示。
- [x] TASK-P2P-010-T3 [test_tier_required]: p2p longrun/s10 脚本、release lock 示例、专题文档口径更新，避免将 `node_tick_ms` 作为出块时间。
- [x] TASK-P2P-010-T4 [test_tier_required + test_tier_full]: 补齐 CLI/launcher/脚本/状态接口回归测试并完成证据收口。

### TASK-P2P-011 执行拆解（PRD-P2P-008）
- [x] TASK-P2P-011-T0 [test_tier_required]: 更新模块主 PRD/project 任务映射，冻结“worker poll vs consensus tick”语义边界与验收口径。
- [x] TASK-P2P-011-T1 [test_tier_required]: 调整 `world_chain_runtime` 状态字段命名（新增 `worker_poll_count`）与 launcher `chain_node_tick_ms` 校验/错误文案，避免误读为出块时间。
- [x] TASK-P2P-011-T2 [test_tier_required]: 修正文档与手册残留（`world-rule`、p2p/node PRD、viewer/manual/site、launcher/longrun 专题）与当前实现能力一致。
- [x] TASK-P2P-011-T3 [test_tier_required]: 运行定向 required 回归并完成项目状态与 devlog 收口。

### TASK-P2P-012 执行拆解（PRD-P2P-009）
- [x] TASK-P2P-012-T0 [test_tier_required]: 在 `doc/p2p/prd.md` 与 `doc/p2p/prd.project.md` 建立“默认参数口径收敛”任务链并冻结验收口径。
- [x] TASK-P2P-012-T1 [test_tier_required]: 调整 `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher` 默认 `slot_duration_ms` 为统一基线值，并收敛 `world_web_launcher` 校验文案为 poll interval 语义，补齐定向回归。
- [x] TASK-P2P-012-T2 [test_tier_required]: 回写 launcher/testing 相关文档默认值与语义说明，执行文档门禁并完成任务收口。

### TASK-P2P-013 执行拆解（PRD-P2P-009）
- [x] TASK-P2P-013-T0 [test_tier_required]: 在主 PRD/project 建立“12s/10/9 默认参数”任务链并冻结验收口径。
- [x] TASK-P2P-013-T1 [test_tier_required]: 调整 `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher/world_viewer_live` 默认 `slot_duration_ms/ticks_per_slot/proposal_tick_phase` 到 `12000/10/9`，并补齐定向回归。
- [x] TASK-P2P-013-T2 [test_tier_required]: 调整 `scripts/p2p-longrun-soak.sh` 与 `scripts/s10-five-node-game-soak.sh` 默认 PoS 参数到 `12000/10/9`，并更新帮助文案。
- [x] TASK-P2P-013-T3 [test_tier_required]: 回写 launcher/testing/longrun 相关文档默认值与语义说明，执行文档门禁并完成任务收口。

### TASK-P2P-014 执行拆解（PRD-P2P-010）
- [x] TASK-P2P-014-T0 [test_tier_required]: 在主 PRD/project 建立“viewer 移除 release/node 控制面参数”任务链并冻结验收口径。
- [x] TASK-P2P-014-T1 [test_tier_required]: 在 `world_viewer_live` CLI 移除 `--release-config` 与 `--node-*` 参数解析、帮助文案与 release-mode 分支，误传时输出迁移提示。
- [x] TASK-P2P-014-T2 [test_tier_required]: 更新 `world_viewer_live.release.example.toml`、viewer manual/site 镜像及相关专题文档，删除与当前能力冲突的表述。
- [ ] TASK-P2P-014-T3 [test_tier_required]: 更新/替换 `world_viewer_live` 定向测试覆盖，验证 legacy 参数拒绝行为与观察服务参数仍可用。
- [ ] TASK-P2P-014-T4 [test_tier_required]: 执行 required 回归并完成项目状态与 devlog 收口。

## 依赖
- doc/p2p/prd.index.md
- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
- `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
- `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.prd.md`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
- `world_viewer_live.release.example.toml`
- `crates/agent_world_client_launcher/src/launcher_core.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `scripts/p2p-longrun-soak.sh`
- `scripts/s10-five-node-game-soak.sh`
- `world-rule.md`
- `doc/world-simulator/viewer/viewer-manual.md`
- `site/doc/cn/viewer-manual.html`
- `site/doc/en/viewer-manual.html`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-08
- 当前状态: in_progress（ROUND-010）
- 下一任务: TASK-P2P-014-T3
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 本轮新增: `TASK-P2P-006` 已完成，专题文档 `p2p-mobile-light-client-authoritative-state-2026-03-06` 已纳入索引和模块追踪映射。
- 本轮新增: `TASK-P2P-008` 已建档，专题文档 `node-pos-slot-clock-real-time-2026-03-07` 已纳入模块追踪映射。
- 本轮新增: `TASK-P2P-009` 已建档，专题文档 `node-pos-subslot-tick-pacing-2026-03-07` 已纳入模块追踪映射。
- TASK-P2P-007 进展（2026-03-07）: 已完成子任务 `TASK-P2P-MLC-002`（intent `tick/seq/sig` 字段、`runtime_live` 幂等 ACK、相关回归测试）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-003` 已完成代码落地与定向 required 回归（权威批次 `state_root/data_root`、`pending/confirmed/final` 状态机、final-only 结算/排行闸门）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-004` 已完成代码落地与 `test_tier_full` 定向回归（watcher challenge 入口、resolve/slash 仲裁、challenge 阻断 final）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-005` 已完成代码落地与定向 required 回归（稳定点回滚、重连追平元数据、会话吊销换钥与鉴权拦截）。
- TASK-P2P-007 进展（2026-03-07）: `TASK-P2P-MLC-006` 已完成 required/full 联合回归与门禁证据沉淀，MLC 专题任务全部收口。
- TASK-P2P-007 收口（2026-03-07）: 主任务状态已完成，子任务 `TASK-P2P-MLC-001~006` 全部闭环并具备 required/full 回归证据。
- TASK-P2P-008 进展（2026-03-07）: `TASK-P2P-008-T0~T3` 全部完成（`slot_duration_ms`/`slot_clock_genesis_unix_ms`、wall-clock 提案门控、`last_observed_slot`/`missed_slot_count` 持久化、proposal/attestation 未来槽与过旧槽拒绝、attestation epoch 映射校验、拒绝原因快照可观测、跨节点 gossip/network 定向回归）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T0` 已完成（专题 PRD/项目管理建档，明确 `10 tick/slot` 相位门控与动态调度方案）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T1` 已完成代码落地与 required 回归（`ticks_per_slot/proposal_tick_phase` 配置校验、logical tick 观测、提案相位门控、tick 级快照持久化）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T2` 已完成代码落地与 required 回归（runtime 动态等待下一 logical tick 边界、异常回退固定间隔、调度频率对比单测）。
- TASK-P2P-009 进展（2026-03-07）: `TASK-P2P-009-T3` 已完成定向 required/full 回归与文档收口，`TASK-P2P-009` 全部完成。
- TASK-P2P-002 进展（2026-03-07）: 已完成三线联合验收清单收口（网络/共识/DistFS 的基线命令、S9/S10 门禁阈值、阻断条件与证据产物定义）。
- TASK-P2P-003 进展（2026-03-07）: 已完成 S9/S10 结果与缺陷闭环模板收口（运行结果字段、缺陷闭环字段、AC/NFR 口径）。
- TASK-P2P-004 进展（2026-03-07）: 已完成发行门禁分布式质量指标映射收口（S9/S10 指标阈值、阻断策略、责任归属与脚本参数对齐）。
- TASK-P2P-010 进展（2026-03-07）: `TASK-P2P-010-T0` 已完成（专题文档建档并回写模块主 PRD / project / index 映射）。
- TASK-P2P-010 进展（2026-03-07）: `TASK-P2P-010-T1` 已完成代码落地与 required 定向回归（runtime/viewer 新增 `--pos-*` 参数、`node_tick_ms` 轮询语义澄清、`NodePosConfig` 映射与相位校验）。
- TASK-P2P-010 进展（2026-03-07）: `TASK-P2P-010-T2` 已完成代码落地与 required 定向回归（game/web/client launcher 新增 `chain-pos-*` 字段、校验与参数透传，UI schema/设置面板同步扩展）。
- TASK-P2P-010 进展（2026-03-07）: `TASK-P2P-010-T3` 已完成脚本与示例口径对齐（`p2p-longrun/s10` 支持 `--pos-*`、`node_tick_ms` 文案改为轮询语义、release 示例补齐 PoS 参数）。
- TASK-P2P-010 进展（2026-03-07）: `TASK-P2P-010-T4` 已完成 required/full 回归与文档收口（runtime/game/web/client launcher + scripts 回归通过，主任务 `TASK-P2P-010` 完成）。
- TASK-P2P-011 启动（2026-03-08）: 新增残留语义收敛任务链，覆盖状态字段命名、launcher 校验文案与跨文档能力对齐。
- TASK-P2P-011 进展（2026-03-08）: `TASK-P2P-011-T1` 已完成代码落地（`/v1/chain/status` 新增 `worker_poll_count` 并保留 `tick_count` 兼容别名；launcher `chain_node_tick_ms` 校验文案改为“poll interval”语义）。
- TASK-P2P-011 进展（2026-03-08）: `TASK-P2P-011-T2` 已完成文档残留收敛（world-rule、p2p node 专题、launcher/longrun 专题、viewer 手册与 site 镜像全部对齐当前 CLI 能力）。
- TASK-P2P-011 收口（2026-03-08）: `TASK-P2P-011-T3` 定向 required 回归通过（`world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher`）并完成任务闭环。
- TASK-P2P-012 启动（2026-03-08）: 新增默认参数口径收敛任务链，覆盖 runtime/game/web/client launcher 默认 `slot_duration_ms` 基线与校验文案统一。
- TASK-P2P-012 进展（2026-03-08）: `TASK-P2P-012-T0` 已完成（主 PRD / project 建档并冻结验收口径）。
- TASK-P2P-012 进展（2026-03-08）: `TASK-P2P-012-T1` 已完成代码落地（统一 runtime/game/web/client launcher 默认 `slot_duration_ms=200`，并将 `world_web_launcher` 的 `chain_node_tick_ms` 校验文案收敛为 poll interval 语义）。
- TASK-P2P-012 收口（2026-03-08）: `TASK-P2P-012-T2` 已完成文档回写（launcher/testing/viewer 专题默认值与语义说明对齐）并通过文档门禁。
- TASK-P2P-013 启动（2026-03-08）: 基于“12s 出块、每块 10 tick”设计口径新增默认参数对齐任务链，覆盖 runtime/game/web/client launcher/world_viewer_live 与 longrun 脚本默认值收敛。
- TASK-P2P-013 进展（2026-03-08）: `TASK-P2P-013-T0` 已完成（主 PRD/project 建档并冻结 `12000/10/9` 验收口径）。
- TASK-P2P-013 进展（2026-03-08）: `TASK-P2P-013-T1` 已完成代码落地与 required 定向回归（runtime/game/web/client/viewer 默认值统一为 `12000/10/9`，并修正 `world_viewer_live` help 默认值文案）。
- TASK-P2P-013 进展（2026-03-08）: `TASK-P2P-013-T2` 已完成脚本默认值收敛（`p2p-longrun/s10` 帮助文案与默认参数统一为 `12000/10/9`）并通过 dry-run 校验。
- TASK-P2P-013 收口（2026-03-08）: `TASK-P2P-013-T3` 已完成文档回写与门禁检查，主任务 `TASK-P2P-013` 全部闭环。
- TASK-P2P-014 启动（2026-03-08）: 基于“viewer 仅保留观察服务 CLI”新增任务链，目标为移除 `world_viewer_live --release-config/--node-*` 控制面参数并收敛文档/测试口径。
- TASK-P2P-014 进展（2026-03-08）: `TASK-P2P-014-T0` 已完成（主 PRD/project 建档并冻结验收口径）。
- TASK-P2P-014 进展（2026-03-08）: `TASK-P2P-014-T1` 已完成代码落地（`parse_launch_options` 显式拒绝 `--release-config/--node-*`，并收敛 help 文案到观察服务入口）。
- TASK-P2P-014 进展（2026-03-08）: `TASK-P2P-014-T2` 已完成文档/示例回写（release 示例改为弃用说明，viewer-manual 与 viewer-live 历史专题补充归档状态）并通过文档门禁。
- 说明: 本文档仅维护 p2p 设计执行状态；过程记录在 `doc/devlog/2026-03-07.md` 与 `doc/devlog/2026-03-08.md`。
- ROUND-002 进展（2026-03-05）: 已并行完成 `B3-C2-009-S2/C2-010/C2-011`（observer sync-mode、node-contribution、distfs-self-healing）主从化回写。
- ROUND-002 进展（2026-03-05）: 已并行完成 `B3-C2-003/C2-008-S1/C2-008-S2`（node-redeemable-power-asset、distfs-production-hardening phase1~9）主从化回写。
