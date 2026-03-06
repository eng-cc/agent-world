# P2P Mobile Light Client 权威状态架构（2026-03-06）

审计轮次: 5

## 1. Executive Summary
- Problem Statement: 手机端算力与电量受限，不适合长期运行全量确定性模拟；若继续要求端侧模拟，会导致在线率和公平性同时下降。
- Proposed Solution: 采用 `手机轻客户端 + 链下权威模拟 + 链上状态承诺/挑战` 分层架构，手机只提交签名输入意图并消费可校验状态增量。
- Success Criteria:
  - SC-MLC-1: 移动端运行路径中不再依赖本地权威模拟，低端机可持续在线。
  - SC-MLC-2: 关键对局状态由权威模拟节点产出，并按提交窗口上链 `state_root`。
  - SC-MLC-3: 客户端清晰区分 `pending/confirmed/final` 三段最终性状态，误判率为 0。
  - SC-MLC-4: Watcher 可在挑战窗口内复算并提交 challenge，恶意提交可被惩罚。
  - SC-MLC-5: 断线重连后可通过快照+增量追平，恢复到最新可确认高度。

## 2. User Experience & Functionality
- User Personas:
  - 移动玩家：关注流畅性、低功耗和不掉线。
  - 协议工程师：关注权威状态、最终性和抗作弊闭环。
  - Watcher/验证者：关注可重放、可挑战、可惩罚。
  - 运维值班：关注重连成功率、不同步率和链上提交健康度。
- User Scenarios & Frequency:
  - 日常游玩：移动玩家每次会话都通过轻客户端输入动作并接收状态增量。
  - 协议发布：每次协议版本升级前验证提交窗口、挑战窗口和回滚语义。
  - 风险稽核：每日抽样重放承诺批次，核对 `state_root` 一致性。
  - 故障恢复：弱网、掉线、重连在长跑和灰度阶段持续演练。
- User Stories:
  - PRD-P2P-MLC-001: As a 移动玩家, I want to send signed intents without local simulator, so that low-end phones can still participate.
  - PRD-P2P-MLC-002: As a 协议工程师, I want authoritative off-chain simulation with on-chain commitments, so that fairness and liveness are both preserved.
  - PRD-P2P-MLC-003: As a Watcher, I want deterministic replay and challenge hooks, so that fraudulent roots can be detected and penalized.
  - PRD-P2P-MLC-004: As an 运维值班, I want reconnect and rollback controls, so that reorgs and network faults remain recoverable.
- Critical User Flows:
  1. Flow-MLC-001: `登录 -> 下发 session key -> 客户端按 tick 提交 intent -> 权威模拟回包 delta/proof`
  2. Flow-MLC-002: `批次提交 state_root 到链上 -> 窗口内 watcher 复算 -> challenge/resolve -> final`
  3. Flow-MLC-003: `客户端显示 pending -> confirmed -> final -> 仅 final 计入资产与排行`
  4. Flow-MLC-004: `客户端掉线 -> 拉取快照 -> 回放增量日志 -> 追平到最新确认 tick`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 输入意图上报 | `player_id/session_pubkey/tick/seq/action/payload_hash/sig` | 客户端只发送 intent，不上报位置/血量等权威状态 | `queued -> sequenced -> executed` | 以 `tick` 主序、`seq` 次序；同 `(player_id, seq)` 去重，且 `seq` 必须与签名 `nonce` 一致 | 仅登录后有效 session key 可写入 |
| 权威增量下发 | `from_tick/to_tick/batch_id/patches/state_root/authority_sig` | 客户端接收并校验签名/根哈希，执行视觉纠偏 | `predicted -> corrected -> confirmed` | `to_tick` 必须单调递增；越界 patch 拒收 | 仅权威节点签名数据可生效 |
| 链上承诺与挑战 | `batch_id/state_root/data_root/bond/challenge_deadline` | 提交承诺；watcher 在窗口内发起 challenge | `committed -> challenged -> resolved -> final` | 超过 `challenge_deadline` 自动进入 final | 仅被授权提交者可 commit；挑战者需抵押 |
| 最终性展示 | `tx_hash/confirm_height/final_height/finality_state` | 客户端 UI 展示三段最终性并限制可消费动作 | `pending -> confirmed -> final` | 资产结算与排行榜仅使用 final 数据 | 非 final 数据禁止触发资产结算 |
| 重连与追平 | `snapshot_height/snapshot_hash/log_cursor` | 断线后拉快照+增量，必要时回滚到稳定点 | `offline -> syncing -> caught_up` | 优先最近稳定快照，再按 cursor 回放 | 仅同账户同会话（或恢复会话）可追平自身视图 |
- Acceptance Criteria:
  - AC-MLC-001 (PRD-P2P-MLC-001): 手机端主流程不启动本地权威模拟器；只存在输入、渲染和纠偏逻辑。
  - AC-MLC-002 (PRD-P2P-MLC-001): 同一 `(player_id, seq)` 重复上报仅生效一次，且具备审计日志。
  - AC-MLC-002a (PRD-P2P-MLC-001): 同一 `(player_id, agent_id, seq)` 重放请求返回幂等 ACK（`idempotent_replay=true`）；同序号不同载荷必须拒绝。
  - AC-MLC-003 (PRD-P2P-MLC-002): 至少 95% 批次在提交窗口内完成 `commit -> confirmed`。
  - AC-MLC-004 (PRD-P2P-MLC-003): challenge 流程可在窗口内成功阻断错误根并产生惩罚记录。
  - AC-MLC-005 (PRD-P2P-MLC-004): 客户端断线恢复流程可在快照可用前提下追平到最近确认高度。
  - AC-MLC-006 (PRD-P2P-MLC-002/004): 客户端最终性 UI 与链上状态一致，不出现“未 final 被当作 final”。
- Non-Goals:
  - 不在本期实现“手机端本地确定性复算全世界状态”。
  - 不在本期替换现有 token 经济规则与发行策略。
  - 不在本期引入新的 L1/L2 迁移方案。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 生成推理链路）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview:
  - `Mobile Light Client`：输入采集、签名、渲染、插值预测、回滚纠偏。
  - `Gateway/Relay`：鉴权、限流、重放保护、弱网中继。
  - `Sequencer`：按 tick 排序 intent 并形成执行批次。
  - `Authoritative Simulator`：权威执行并产出 `delta + state_root`。
  - `Watcher`：复算批次并发起 challenge。
  - `On-chain Contracts`：记录承诺、处理挑战与惩罚、定义治理升级权限。
  - `DA/Snapshot Store`：保存快照与日志，支撑追平和审计。
- Integration Points:
  - `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
  - `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.project.md`
  - `doc/p2p/prd.md`
  - `doc/p2p/network/readme-p1-network-production-hardening.prd.md`
  - `doc/p2p/distributed/distributed-runtime.prd.md`
  - `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 意图乱序/重复：按 `(tick, seq)` 排序并去重，重复 intent 返回幂等 ACK。
  - 序号篡改：`intent_seq` 与签名 `nonce` 不一致时直接拒绝，避免重放窗口绕过。
  - 长时间无 peer：Gateway 回退到中继链路并触发网络健康告警。
  - 链重组：客户端回滚到最近稳定提交点并重放未最终批次。
  - 挑战超时：窗口超时后状态进入 final，后续仅允许审计不上链回滚。
  - 快照损坏：校验哈希失败即丢弃，回退到上一个可用快照并补拉增量。
  - 会话密钥泄露：支持会话吊销与换钥，吊销后旧会话 intent 全部拒绝。
  - 弱网高抖动：客户端降级为低频 delta + 关键帧同步，保活优先。
- Non-Functional Requirements:
  - NFR-MLC-1 (性能): 目标模拟频率 15Hz；客户端 delta 接收频率默认 5Hz。
  - NFR-MLC-2 (可用性): Sequencer/Simulator 提交路径支持主备切换，避免单点停服。
  - NFR-MLC-3 (安全): intent 与增量消息必须签名验签；链上承诺必须可追溯到批次数据根。
  - NFR-MLC-4 (扩展): 协议字段采用版本化编码，确保旧客户端可识别并安全降级。
  - NFR-MLC-5 (可观测): 暴露 `不同步率/挑战成功率/重连追平时延/提交失败率` 指标。
  - NFR-MLC-6 (可审计): 任意 final 批次均可基于日志和快照进行离线重放核验。
- Security & Privacy:
  - 客户端采用短期 session key，主私钥不常驻移动端热路径。
  - 权威状态写入权限受治理约束，提交者/仲裁者权限链上可审计。
  - 日志与快照按账户最小化暴露，避免泄露无关玩家 AOI 数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-10): 完成 intent-only 轻客户端协议、权威批次承诺与最终性 UI。
  - v1.1 (2026-03-20): 完成 challenge/slash 闭环、重组回滚与断线追平演练。
  - v2.0 (2026-03-31): 完成高可用切换与质量看板，纳入发布门禁。
- Technical Risks:
  - 风险-MLC-1: Sequencer 失效导致输入积压和状态漂移。
  - 风险-MLC-2: 挑战窗口与游戏实时性冲突，影响玩家“结果已完成”认知。
  - 风险-MLC-3: 快照/日志数据可用性不足导致新节点与重连节点追平失败。
  - 风险-MLC-4: 协议版本并存阶段出现字段兼容偏差。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MLC-001 | TASK-P2P-MLC-001/002 | `test_tier_required` | intent 签名/去重/幂等 ACK 回归 | 移动端输入链路与网关 |
| PRD-P2P-MLC-002 | TASK-P2P-MLC-003/004 | `test_tier_required` + `test_tier_full` | commit/finality 状态机 + UI 最终性一致性检查 | 模拟执行与结算可见性 |
| PRD-P2P-MLC-003 | TASK-P2P-MLC-004/005 | `test_tier_full` | challenge 复算、错误根阻断与惩罚验证 | 安全与反作弊 |
| PRD-P2P-MLC-004 | TASK-P2P-MLC-005/006 | `test_tier_required` + `test_tier_full` | 掉线恢复、链重组回滚、快照追平演练 | 可用性与稳定性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-MLC-001 | 手机端仅发送 intent，不上传权威状态 | 手机端上报状态并“服务端参考” | 避免作弊入口并统一权威状态来源。 |
| DEC-MLC-002 | 链下实时模拟 + 链上周期承诺 | 每 tick 全量上链执行 | 实时性和链上成本不可兼得，需分层。 |
| DEC-MLC-003 | 设置 challenge 窗口并引入 watcher | 无挑战直接最终确认 | 缺少外部复核会放大错误提交风险。 |
| DEC-MLC-004 | 客户端强制展示三段最终性 | 仅显示“已完成”单状态 | 可显著降低重组/回滚造成的体验误导。 |

## 7. Review Gate（Self-Review）
| 检查项 | 结论 | 证据 |
| --- | --- | --- |
| 目标与背景（Why） | ✔ | 第 1 章定义问题、方案与 5 条可量化成功标准。 |
| 用户与场景（Who/When） | ✔ | 第 2 章包含 4 类角色、频次场景与关键流程。 |
| 范围定义（Scope） | ✔ | 第 2 章给出 Non-Goals，边界清晰。 |
| 功能规格完整性（What） | ✔ | 第 2 章规格矩阵覆盖字段、动作、状态、规则、权限。 |
| 异常与边界 | ✔ | 第 4 章列出乱序、重组、快照损坏、泄露等异常处理。 |
| NFR 可量化 | ✔ | 第 4 章给出频率、可用性、安全、观测与审计指标。 |
| 可测试性 | ✔ | 第 6 章提供 PRD-ID -> Task -> Test tier 映射。 |
| 逻辑一致性 | ✔ | Flow、AC、NFR 与决策记录互相可追溯。 |
| 依赖与影响分析 | ✔ | 第 4 章 Integration Points 指向跨模块依赖。 |
| 决策透明度 | ✔ | 第 6 章记录选型与否决方案。 |
| 文档树一致性 | ✔ | 文档归属 `doc/p2p/network/`，并由 `doc/p2p/prd.index.md` 索引。 |

- Gate Result: 🟢 Ready
