# p2p PRD

审计轮次: 6

## 目标
- 建立 p2p 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 p2p 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 p2p 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/p2p/project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/p2p/prd.md`
- 项目管理入口: `doc/p2p/project.md`
- 文件级索引: `doc/p2p/prd.index.md`
- 追踪主键: `PRD-P2P-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 网络、共识、DistFS 与节点激励相关设计迭代频繁，缺少统一 PRD 导致跨子系统改动难以同时满足可用性、安全性与可审计性。
- Proposed Solution: 以 p2p PRD 统一定义分布式系统的目标拓扑、共识约束、存储策略、奖励机制与发布门禁。
- Success Criteria:
  - SC-1: P2P 关键改动 100% 映射到 PRD-P2P-ID。
  - SC-2: 多节点在线长跑套件按计划执行并形成可追溯结果。
  - SC-3: 共识与存储链路关键失败模式具备回归测试覆盖。
  - SC-4: 发行前完成网络/共识/DistFS 三线联合验收。
  - SC-5: 移动端轻客户端路径可在不运行本地权威模拟器前提下稳定接入。
  - SC-6: PoS slot/epoch 在多节点间由统一时间公式驱动，允许漏槽但不出现时间语义倒退。
  - SC-7: PoS 支持槽内 logical tick 相位门控与动态节拍调度，实现可配置 `tick/slot` 语义。
  - SC-8: `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher/scripts` 控制面参数与状态口径与 PoS 时间锚定语义一致，不再将 `node_tick_ms` 误解为出块时间。
  - SC-9: 清理残留时序语义偏差（`tick_count` 观测命名、`world_viewer_live` 旧控制面假设、`world-rule` 时间模型描述），保证规范/实现/运维口径一致。
  - SC-10: runtime/game/web/client launcher 默认 PoS 时间参数与文档一致，默认启动即满足“slot 时钟锚定 + 轮询语义解耦”口径。
  - SC-11: runtime/game/web/client launcher 与 longrun 脚本默认参数统一为 `slot_duration_ms=12000`、`ticks_per_slot=10`、`proposal_tick_phase=9`，满足“12s 出块、每块 10 tick”基线。
  - SC-12: `world_viewer_live` 对外 CLI 收敛为纯观察服务，不再接受 `--release-config` 与 `--node-*` 控制面参数；误传时必须显式拒绝并提示改用 `world_chain_runtime`。
  - SC-13: `world_viewer_live` 移除 legacy 参数兼容层，不再接受 `--runtime-world` 等历史别名；代码库中不再保留未接入生产入口的旧 CLI 解析路径。
  - SC-14: 历史 PRD/project 文档中的 `world_viewer_live` 旧文件路径完成替换，不再指向已删除的 `src/bin/world_viewer_live/` 子目录文件。

## 2. User Experience & Functionality
- User Personas:
  - 协议工程师：需要明确网络与共识边界。
  - 节点运营者：需要稳定部署和可观测运行信号。
  - 安全评审者：需要签名、治理、资产流转的可审计证据。
  - 移动端玩家：需要低算力设备可持续在线并获得正确最终性反馈。
- User Scenarios & Frequency:
  - 协议演进评审：每次共识或网络协议改动前执行。
  - 多节点长跑：按周执行并记录稳定性与恢复结果。
  - 发行前联合验收：每个候选版本执行一次三线联测。
  - 安全审计复核：关键资产链路改动后立即触发。
  - 轻客户端接入验收：每次移动端协议调整后执行输入/最终性/重连验证。
- User Stories:
  - PRD-P2P-001: As a 协议工程师, I want explicit protocol boundaries, so that multi-crate changes remain coherent.
  - PRD-P2P-002: As a 节点运营者, I want reliable longrun validation, so that production confidence increases.
  - PRD-P2P-003: As a 安全评审者, I want auditable cryptographic and governance flows, so that risk is controlled.
  - PRD-P2P-004: As a 移动端玩家, I want intent-only light client access, so that low-end devices can still participate fairly.
  - PRD-P2P-005: As a 协议工程师, I want slot/epoch to be wall-clock driven, so that block time semantics remain stable across restart and lag.
  - PRD-P2P-006: As a 协议工程师, I want slot-internal tick-phase pacing, so that proposal cadence can follow configured `ticks_per_slot`.
  - PRD-P2P-007: As a 节点运营者, I want runtime/launcher/scripts to expose anchored slot-clock parameters explicitly, so that block-time tuning is deterministic and auditable.
  - PRD-P2P-008: As a 协议工程师, I want cross-doc and status field naming to disambiguate worker polling vs consensus ticks, so that observability and operations avoid semantic drift.
  - PRD-P2P-009: As a 节点运营者, I want sane default PoS timing values and uniform validation wording, so that default startup already follows anchored block-time semantics without hidden overrides.
  - PRD-P2P-010: As a 发布维护者, I want `world_viewer_live` to reject legacy release/node control-plane flags, so that chain control is unambiguously hosted by `world_chain_runtime`.
  - PRD-P2P-011: As a 发布维护者, I want legacy compatibility aliases removed from `world_viewer_live`, so that CLI semantics are single-source and there is no dead parser path.
  - PRD-P2P-012: As a 维护者, I want historical docs to reference current source layout, so that reviewers do not chase deleted paths during audit or regression.
- Critical User Flows:
  1. Flow-P2P-001: `网络拓扑变更 -> 共识联调 -> DistFS 同步 -> 节点状态一致性验证`
  2. Flow-P2P-002: `执行 S9/S10 长跑 -> 采集故障与恢复数据 -> 输出收敛报告`
  3. Flow-P2P-003: `资产/签名链路变更 -> 审计检查 -> 安全门禁 -> 发布判定`
  4. Flow-P2P-004: `手机端提交签名 intent -> 权威模拟执行 -> 链上承诺/挑战 -> 客户端 final 确认`
  5. Flow-P2P-005: `节点读取 wall-clock -> 计算 slot/epoch -> 允许漏槽推进 -> 拒绝未来槽/过旧槽提案`
  6. Flow-P2P-006: `节点按 wall-clock 计算 logical tick/phase -> 相位命中才提案 -> runtime 动态等待下一 tick 边界`
  7. Flow-P2P-007: `运维配置 slot_duration/ticks_per_slot/proposal_phase -> runtime/game/web/client launcher 统一生效 -> status/soak 输出可观测并用于门禁`
  8. Flow-P2P-008: 状态接口/手册/PRD 同步更新 -> `tick_count` 明确为 worker poll 指标 -> 采样脚本以共识 slot/tick/height 为主
  9. Flow-P2P-009: `默认启动 runtime/game/web/client launcher -> 使用统一默认 slot_duration/ticks_per_slot -> 文档/帮助/校验文案一致呈现 poll vs slot 语义`
  10. Flow-P2P-010: `用户误传 world_viewer_live --release-config/--node-* -> CLI 显式拒绝并给出替代入口 -> 文档与示例迁移到 world_chain_runtime`
  11. Flow-P2P-011: `用户误传 world_viewer_live 任意 legacy 参数（含 --runtime-world） -> CLI 明确拒绝并输出迁移入口 -> 测试与手册口径一致`
  12. Flow-P2P-012: `执行历史文档巡检 -> 替换已删除源码路径到当前入口路径 -> 文档门禁 + grep 零残留校验`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 网络与共识协同 | 节点ID、轮次、提交高度、延迟 | 启动联测并比对共识结果 | `joining -> syncing -> committed` | 高度/轮次单调递增 | 仅授权节点参与共识 |
| DistFS 复制 | 文件ID、副本状态、同步延迟 | 触发复制并校验完整性 | `queued -> replicating -> verified` | 优先关键数据副本 | 节点需满足存储策略 |
| 长跑与恢复 | 失败类型、恢复动作、恢复时长 | 注入故障并执行恢复流程 | `stable -> degraded -> recovered` | 按故障等级排序处理 | 运维/评审可操作恢复流程 |
| 轻客户端权威状态 | `intent(tick/seq/sig)`、`state_root`、`finality_state` | 手机端只上报 intent，接收 delta/proof 并展示最终性 | `pending -> confirmed -> final` | 按 tick 排序，重复 seq 幂等去重 | 权威状态仅由模拟节点提交，客户端无写权限 |
| PoS 固定时间槽 | `genesis_unix_ms`、`slot_duration_ms`、`epoch_length_slots`、`last_observed_slot`、`missed_slot_count` | 每次 tick 按真实时间换算 slot；仅在 `next_slot <= current_slot` 时允许提案 | `pending -> committed/rejected`（槽位单调） | `current_slot=floor((now-genesis)/slot_duration)`；`epoch=slot/epoch_length_slots` | 仅验证者可提案/投票；未来槽消息拒绝 |
| PoS 槽内 tick 节拍 | `ticks_per_slot`、`tick_phase`、`proposal_tick_phase`、`last_observed_tick`、`missed_tick_count` | 仅在命中提案相位时触发提案；worker 按下一 logical tick 边界动态调度 | `idle -> proposing`（相位门控） | `logical_tick=floor((now-genesis)*ticks_per_slot/slot_duration)`；`phase=tick%ticks_per_slot` | 节拍公式全节点一致；本地调度可回退固定间隔 |
| PoS 控制面参数对齐 | `node_tick_ms`（轮询）+ `slot_duration_ms`、`ticks_per_slot`、`proposal_tick_phase`、`adaptive_tick_scheduler_enabled`、`slot_clock_genesis_unix_ms`、`max_past_slot_lag` | runtime/game/web/client launcher/scripts 显式暴露并校验参数，状态接口回显观测字段 | `configured -> running -> audited` | `node_tick_ms` 不参与出块时间计算，仅作为 worker 轮询/回退间隔 | 运维可配置；非法值必须启动前拒绝 |
| 时序语义残留收敛 | `worker_poll_count`、`consensus.last_observed_tick`、`consensus.committed_height`、`slot_duration_ms` | 更新状态字段命名/文档叙述并修复过时控制面假设 | `legacy -> aligned` | 轮询指标与共识指标分离，不混用同一“tick”语义 | 运维/QA 只读；配置前必须通过校验 |
| Viewer 控制面边界收敛 | `world_viewer_live` CLI（`--bind`/`--web-bind`/`--llm`/`--no-llm`） | 仅保留观察服务参数；误传 `--release-config`、`--runtime-world`、`--node-*` 与其他 legacy 控制面参数直接拒绝 | `legacy_mixed -> observer_only_strict` | CLI 白名单固定；错误信息必须包含迁移目标 `world_chain_runtime` | 运行链控制面仅限受信运维入口 |
- 三线联合验收清单（TASK-P2P-002）:
| 线别 | 必跑命令（基线） | 联合验收门禁 | 阻断条件（任一命中即 fail） | 证据产物 |
| --- | --- | --- | --- | --- |
| 网络线（net） | `env -u RUSTC_WRAPPER cargo test -p agent_world_net --lib`；`env -u RUSTC_WRAPPER cargo test -p agent_world_net --features libp2p --lib` | `./scripts/release-gate.sh --dry-run` + S9 发布档位命令（见 `testing-manual.md`） | `agent_world_net` 单测失败；S9 `metric_gate.status != pass`；`consensus_hash_consistent != true` | `release-gate-summary.md`、S9 `summary.json/timeline.csv` |
| 共识线（consensus） | `env -u RUSTC_WRAPPER cargo test -p agent_world_consensus --lib`；`env -u RUSTC_WRAPPER cargo test -p agent_world_node --lib` | S9 + S10 发布档位命令（见 `testing-manual.md`） | 共识/节点单测失败；S9 或 S10 `overall_status/run.status != ok`；`consensus_hash_mismatch_count > 0` | S9/S10 `summary.json`、`failures.md`（若失败） |
| 存储线（DistFS） | `env -u RUSTC_WRAPPER cargo test -p agent_world_distfs --lib` | S9 发布档位命令（含 `--max-distfs-failure-ratio 0.1`） | DistFS 单测失败；`distfs_failure_ratio` 超阈值；反馈/复制不一致无法闭环 | S9 `summary.json`、`feedback_events.log`、`chaos_events.log` |
- Acceptance Criteria:
  - AC-1: p2p PRD 覆盖网络、共识、存储、激励四条主线。
  - AC-2: p2p project 文档任务项明确映射 PRD-P2P-ID。
  - AC-3: 与 `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md` 等设计文档口径一致。
  - AC-4: S9/S10 相关测试套件在 testing 手册中有对应条目。
  - AC-5: 轻客户端专题需求落盘并映射到独立任务链（`TASK-P2P-MLC-*`）。
  - AC-6: `node-pos-slot-clock-real-time-2026-03-07` 专题文档落盘并映射任务链 `TASK-P2P-008`。
  - AC-7: `node-pos-subslot-tick-pacing-2026-03-07` 专题文档落盘并映射任务链 `TASK-P2P-009`。
  - AC-8: 三线联合验收清单明确给出“基线命令 + 发布门禁阈值 + 阻断条件 + 证据产物”，可直接用于发行前检查。
  - AC-9: S9/S10 长跑结果模板与缺陷闭环模板完成定义，失败运行必须能映射到 `incident_id -> 修复任务 -> 回归证据`。
  - AC-10: 发行门禁分布式质量指标（S9/S10）具备“阈值 + 数据源 + 阻断策略 + 责任归属”映射，并与 `release-gate` 脚本参数一致。
  - AC-11: `node-pos-time-anchor-control-plane-alignment-2026-03-07` 专题文档落盘并映射任务链 `TASK-P2P-010`，覆盖 runtime/game/web/client launcher/scripts 与状态接口口径对齐。
  - AC-12: 残留语义项完成收敛：`world-rule` 时间模型、launcher `chain_node_tick_ms` 校验文案、`/v1/chain/status` 轮询字段命名、viewer/manual/site 与 `world_viewer_live` 实际 CLI 能力保持一致。
  - AC-13: `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher` 默认 `slot_duration_ms` 与文档基线一致；`world_web_launcher` 校验文案明确 `chain_node_tick_ms` 为 poll interval 语义。
  - AC-14: `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher/world_viewer_live/p2p-longrun/s10` 默认 `slot_duration_ms/ticks_per_slot/proposal_tick_phase` 与“12s/10/9”基线一致，相关默认值断言与手册同步更新。
  - AC-15: `world_viewer_live` 解析层移除 `--release-config` 与 `--node-*` 参数能力；定向测试覆盖“误传 legacy 参数 -> 启动失败 + 替代提示”路径。
  - AC-16: `world_viewer_live` 进一步移除 `--runtime-world` 兼容别名与旧 split CLI 路径，定向测试覆盖 `--release-config/--runtime-world/--node-*` 拒绝行为。
  - AC-17: 历史文档中 `world_viewer_live` 子目录旧路径完成迁移（对齐 `world_viewer_live.rs` 与 `world_chain_runtime/*` 现行布局），文档门禁通过。
- Non-Goals:
  - 不在本 PRD 细化 viewer UI 交互。
  - 不替代 runtime 内核的模块执行细节设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 长跑脚本、链路探针、反馈注入、共识日志分析工具。
- Evaluation Strategy: 以在线稳定时长、分叉恢复成功率、反馈链路可用性、错误收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: p2p 模块负责 `agent_world_net`/`agent_world_consensus`/`agent_world_distfs` 与 node 侧分布式运行协同，强调一致性与故障恢复。
- Integration Points:
  - `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
  - `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
  - `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
  - `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
  - `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
  - `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.prd.md`
  - `world-rule.md`
  - `doc/world-simulator/viewer/viewer-manual.md`
  - `doc/world-simulator/launcher/game-client-launcher-chain-runtime-decouple-2026-02-28.prd.md`
  - `world_viewer_live.release.example.toml`
  - `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.md`
  - `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 节点掉线：共识链路需在节点恢复后自动重同步并验证状态。
  - 网络分区：检测分区后阻断不安全提交并等待合并恢复。
  - 轻客户端弱网：启用低频增量+关键帧同步并保持最终性状态不倒退。
  - 空副本：DistFS 副本不足时触发补副本任务并记录告警。
  - 超时：共识轮次超时后执行回退/重试策略。
  - 并发冲突：同高度多提交候选按共识规则拒绝冲突分支。
  - 数据损坏：校验失败副本立即隔离并重建。
  - 时钟回拨/漂移：wall-clock 出现回拨时禁止 slot 倒退；超阈值漂移进入拒绝或告警路径。
  - 大跨度漏槽：节点恢复后按当前 wall-clock 对齐 slot，并累加漏槽计数，不补历史空块。
  - 控制面兼容：保留 `node_tick_ms` 时必须明确其“轮询/回退间隔”语义，避免误用为 slot/block 时间。
  - 旧参数误用：`world_viewer_live` 若收到 `--release-config` 或任意 `--node-*` 参数，必须立即失败并输出“请改用 world_chain_runtime”。
  - 兼容别名误用：`world_viewer_live` 若收到 `--runtime-world`，必须立即失败并输出“请直接使用纯 viewer 参数”。
- Non-Functional Requirements:
  - NFR-P2P-1: 多节点长跑稳定性指标持续达标并可追溯。
  - NFR-P2P-2: 共识提交与复制链路关键失败模式覆盖率 100%。
  - NFR-P2P-3: 节点异常恢复流程具备标准化操作与证据产物。
  - NFR-P2P-4: 资产与签名链路审计记录完整率 100%。
  - NFR-P2P-5: 协议演进不得破坏既有网络兼容性基线。
  - NFR-P2P-6: 手机轻客户端路径必须可验证最终性，且不要求端侧权威模拟。
  - NFR-P2P-7: slot 计算在重启前后保持单调一致；槽位倒退容忍度为 0（仅允许漏槽）。
  - NFR-P2P-8: 在启用 `ticks_per_slot` 时，logical tick/phase 计算跨节点一致，提案节拍可观测且可回归验证。
  - NFR-P2P-9: S9/S10 若出现失败，必须在同一审计轮次内沉淀 `incident_id/root_cause/fix_commit/regression_command` 四元组证据。
  - NFR-P2P-10: 分布式发布门禁不得接受 `insufficient_data` 作为通过结果；S9/S10 指标门禁结果必须显式为 `pass`。
  - NFR-P2P-11: 控制面参数命名与状态字段在 runtime/game/web/client launcher/scripts 上保持一致，避免语义分叉导致错误调参。
  - NFR-P2P-12: 指标命名必须区分“worker poll”与“consensus tick”；任何对外接口不得将二者混称为同一 tick 语义。
  - NFR-P2P-13: `world_viewer_live` CLI 帮助与错误文案中不得再出现 release/node 控制面入口，避免与 `world_chain_runtime` 控制平面重复。
  - NFR-P2P-14: `world_viewer_live` 仅保留一个生效的 CLI 解析实现；仓内不得存在与生产入口分叉的 legacy 参数解析代码路径。
  - NFR-P2P-15: 模块文档中的源码路径引用必须可解析到当前仓库存在文件，避免审计与回归排障时出现失效链接。
- Security & Privacy: 需保证节点身份、签名、账本与反馈数据链路的完整性；所有关键动作必须具备可审计记录。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化网络/共识/存储统一设计基线。
  - v1.1: 补齐在线长跑失败模式和恢复手册。
  - v2.0: 建立分布式质量趋势看板（稳定性、时延、恢复、失败率）。
- Technical Risks:
  - 风险-1: 多子系统并行演进带来接口漂移。
  - 风险-2: 长跑测试覆盖不足导致线上异常暴露滞后。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-001 | TASK-P2P-001/002/005 | `test_tier_required` | 网络/共识/存储联合验收清单检查 | 协议边界与跨 crate 兼容 |
| PRD-P2P-002 | TASK-P2P-002/003/005 | `test_tier_required` + `test_tier_full` | S9/S10 长跑与恢复演练 | 多节点稳定性与故障恢复 |
| PRD-P2P-003 | TASK-P2P-003/004/005 | `test_tier_full` | 签名与治理链路审计检查 | 资产安全与发布风险控制 |
| PRD-P2P-004 | TASK-P2P-006/007 | `test_tier_required` + `test_tier_full` | 轻客户端 intent/finality/challenge/reconnect 闭环验证 | 移动端接入、公平性与可用性 |
| PRD-P2P-005 | TASK-P2P-008 | `test_tier_required` + `test_tier_full` | 固定时间槽单调性/漏槽/重启恢复/未来槽拒绝回归 | 共识时间语义、提案与投票窗口 |
| PRD-P2P-006 | TASK-P2P-009 | `test_tier_required` + `test_tier_full` | 槽内 tick 相位门控、动态调度等待与跨节点节拍回归 | 共识提案节奏、runtime 调度与可观测 |
| PRD-P2P-007 | TASK-P2P-010 | `test_tier_required` + `test_tier_full` | runtime/game/web/client launcher/scripts 参数映射、状态观测与兼容回归 | 控制面调参、运维门禁与观测一致性 |
| PRD-P2P-008 | TASK-P2P-011 | `test_tier_required` | 状态字段语义对齐、launcher 校验文案回归、文档一致性检查 | 运维观测、发行手册与参数治理 |
| PRD-P2P-009 | TASK-P2P-012 | `test_tier_required` | 默认参数一致性、launcher/web 校验文案与手册口径回归 | 默认启动行为、控制面配置与运维认知一致性 |
| PRD-P2P-009 | TASK-P2P-013 | `test_tier_required` | 默认值切换到 `12s/10/9` 并回归 CLI/脚本/文档口径 | 时间锚定基线一致性与默认运行节拍 |
| PRD-P2P-010 | TASK-P2P-014 | `test_tier_required` | `world_viewer_live` legacy 参数拒绝、帮助文案收敛与文档/示例迁移回归 | Viewer/chain 控制面边界一致性 |
| PRD-P2P-011 | TASK-P2P-015 | `test_tier_required` | `world_viewer_live` 删除 `--runtime-world` 兼容别名、移除旧 split CLI 路径并回归手册/测试口径 | CLI 单一事实源与维护成本收敛 |
| PRD-P2P-012 | TASK-P2P-016 | `test_tier_required` | 历史文档旧路径替换 + 文档门禁 + 旧路径 grep 零残留校验（过程日志除外） | 文档可追溯性与维护效率 |
- S9/S10 长跑结果模板（TASK-P2P-003）:
| 字段 | 说明 | 来源 |
| --- | --- | --- |
| `suite` | `S9` 或 `S10` | `testing-manual.md` 套件定义 |
| `run_id` | 本次运行唯一标识（目录名/时间戳） | `.tmp/p2p_longrun/*` 或 `.tmp/s10_game_longrun/*` |
| `profile` | `soak_smoke/soak_endurance/soak_release` 或 S10 对应档位 | 执行命令 |
| `gate_status` | `pass/fail` | `summary.json` |
| `key_metrics` | `lag_p95/distfs_failure_ratio/consensus_hash_mismatch_count` 等 | `summary.json` |
| `evidence_paths` | `summary.json/timeline.csv/failures.md` | 产物目录 |
- S9/S10 缺陷闭环模板（TASK-P2P-003）:
| 字段 | 填写要求 | 闭环判定 |
| --- | --- | --- |
| `incident_id` | `S9-YYYYMMDD-xxx` 或 `S10-YYYYMMDD-xxx` | 与失败运行一一对应 |
| `symptom` | 失败现象 + 首个告警指标 | 可定位到日志行或指标项 |
| `root_cause` | 技术根因（网络/共识/存储/配置） | 具备可复现实验步骤 |
| `fix_task` | 对应任务 ID（如 `TASK-P2P-00x-*`） | 任务文档可追踪 |
| `fix_commit` | 修复提交 SHA | commit 可检索 |
| `regression_command` | 至少 1 条定向回归 + 1 条长跑复验命令 | 命令可执行且结果通过 |
| `closure_note` | 风险评估与是否阻断发布 | 发布门禁结论一致 |
- 发行门禁分布式质量指标映射（TASK-P2P-004）:
| 指标 | 数据源 | 发布阈值（2026-03-07） | 阻断策略 | 执行责任 |
| --- | --- | --- | --- | --- |
| `S9.topologies[].metric_gate.status` | S9 `summary.json` | 必须为 `pass` | 任何拓扑 `fail/insufficient_data` 直接阻断 | 发行值班工程师 |
| `S9.topologies[].metrics.consensus_hash_consistent` | S9 `summary.json` | 必须为 `true` | 任意 `false` 直接阻断并拉起共识排障 | 共识 owner |
| `S9.topologies[].metrics.consensus_hash_mismatch_count` | S9 `summary.json` | 必须为 `0` | 非 0 直接阻断并要求补 `consensus_hash_mismatch.tsv` 分析 | 共识 owner |
| `S9.topologies[].metrics.lag_p95` | S9 `summary.json` | `<= 50`（由 `--max-lag-p95 50` 注入） | 超阈值阻断并进入网络退化复盘 | 网络 owner |
| `S9.topologies[].metrics.distfs_failure_ratio` | S9 `summary.json` | `<= 0.1`（由 `--max-distfs-failure-ratio 0.1` 注入） | 超阈值阻断并进入 DistFS 复制链路修复 | DistFS owner |
| `S10.run.metric_gate.status` | S10 `summary.json` | 必须为 `pass` | `fail/insufficient_data` 直接阻断 | 发行值班工程师 |
| `S10.run.status` | S10 `summary.json` | 必须为 `ok` | 非 `ok` 直接阻断并要求 `failures.md` | 发行值班工程师 |
| `S10.run.metrics.lag_p95` | S10 `summary.json` | `<= 50`（由 `--max-lag-p95 50` 注入） | 超阈值阻断并回退到性能/网络专项 | 网络 owner |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-001 | 网络/共识/DistFS 统一验收 | 子系统独立验收 | 可降低跨链路隐性回归风险。 |
| DEC-P2P-002 | 长跑结果进入发布门禁 | 仅开发阶段抽样运行 | 发布质量依赖真实长稳证据。 |
| DEC-P2P-003 | 关键动作全链路审计 | 仅关键节点日志 | 审计深度不足会放大安全风险。 |
| DEC-P2P-004 | 移动端采用轻客户端+链下权威模拟 | 手机端参与权威模拟 | 移动端资源受限，权威性和实时性需分层保障。 |
| DEC-P2P-005 | PoS slot 按 wall-clock 统一公式驱动 | 继续本地 tick 自增 slot | 可消除重启/负载抖动造成的时间语义漂移。 |
| DEC-P2P-006 | PoS 增加槽内 tick 相位门控与动态调度 | 仅保留固定 `tick_interval` 与 slot 门控 | 需要稳定落地 `10 tick/slot` 节奏并降低固定 sleep 漂移。 |
| DEC-P2P-007 | 三线联合验收采用“子系统单测基线 + S9/S10 长跑门禁”双层收口 | 仅保留单测或仅保留长跑 | 单一层级无法覆盖“确定性回归 + 长时退化”双维风险。 |
| DEC-P2P-008 | 分布式质量指标按 `release-gate.sh` 参数固化为“硬阻断” | 发布前人工主观评估放行 | 降低人工判断漂移，确保门禁可复现。 |
| DEC-P2P-009 | 将 `node_tick_ms` 定义为 worker 轮询/回退间隔，并显式暴露 slot-clock 参数 | 继续用 `node_tick_ms` 承担出块时间语义 | 减少运维误配与观测误读，保证时间锚定语义可操作。 |
| DEC-P2P-010 | 在状态与文档中显式区分 `worker_poll_count` 与共识 tick/height 指标 | 继续沿用 `tick_count` 作为泛化进度字段 | 避免“轮询次数=出块推进”的误读，降低误判与误调参风险。 |
| DEC-P2P-011 | 统一 runtime/game/web/client launcher 默认 `slot_duration_ms` 为文档基线值，并收敛校验文案为 poll interval 语义 | 继续维持 `slot_duration_ms=1` 且允许文案混用 tick/block 语义 | 减少“默认启动即偏离锚定口径”的隐性配置风险，降低运维误读。 |
| DEC-P2P-012 | 默认 PoS 时间参数采用 `slot_duration_ms=12000`、`ticks_per_slot=10`、`proposal_tick_phase=9` | 保持 `200/1/0` 等压测导向默认组合 | 与“12s 出块、每块 10 tick”设计口径一致，默认体验与协议基线对齐。 |
| DEC-P2P-013 | `world_viewer_live` 移除 `--release-config` 与 `--node-*` 控制面参数，仅保留观察服务 CLI | 继续在 viewer 保留 release/node 控制面兼容入口 | 避免控制面双入口造成运维误配，统一由 `world_chain_runtime` 承担链参数与节点生命周期。 |
| DEC-P2P-014 | `world_viewer_live` 删除 `--runtime-world` 兼容别名与 legacy split CLI 代码，保留单一生产入口 `world_viewer_live.rs` | 继续保留兼容别名和未接入入口的旧解析代码 | 避免“文档/测试改了但真实入口不生效”的双轨风险，降低后续维护和误判成本。 |
| DEC-P2P-015 | 统一将历史文档中的 `world_viewer_live` 旧文件路径替换为当前源码布局路径（`world_viewer_live.rs` / `world_chain_runtime/*`） | 保留旧路径并依赖读者自行映射 | 降低审计误导与排障成本，确保文档可直接定位现行实现。 |
