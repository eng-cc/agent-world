# README P1 缺口收口：分布式网络主路径生产化

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 收口 P1-1：将 Node 共识消息（proposal/attestation/commit）从“仅 UDP gossip”升级为“libp2p pubsub 主路径”，保留 UDP 兼容兜底。
- Proposed Solution: 收口 P1-2：将 libp2p request/response 从“单 peer + 无 peer 本地 handler 回退”升级为“多 peer 轮换重试 + 可控本地回退策略”。
- Success Criteria:
  - SC-1: 保持现有 `world_viewer_live` 生产默认拓扑（triad/triad_distributed）可用，并保证 required-tier 回归稳定。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want README P1 缺口收口：分布式网络主路径生产化 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: In scope
  - AC-2: `crates/agent_world_node/src/libp2p_replication_network.rs`
  - AC-3: request 路由升级为多 peer 轮换 + 失败重试。
  - AC-4: 增加“无 peer 时本地 handler 回退”开关，默认关闭。
  - AC-5: `crates/agent_world_node/src/network_bridge.rs`
  - AC-6: 新增共识 topic endpoint（proposal/attestation/commit）。
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/network/readme-p1-network-production-hardening.prd.md`
  - `doc/p2p/network/readme-p1-network-production-hardening.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) libp2p request 路由
- `Libp2pReplicationNetworkConfig` 新增：
  - `allow_local_handler_fallback_when_no_peers: bool`（默认 `false`）。
- request 语义：
  - 有连接 peer：按轮换顺序选择 peer，失败自动切下一个，直到成功或耗尽。
  - 无连接 peer：默认返回 `NetworkProtocolUnavailable`；仅在开关开启时允许本地 handler 回退。

### 2) 共识 topic（libp2p）
- 主题命名：
  - `aw.<world_id>.consensus.proposal`
  - `aw.<world_id>.consensus.attestation`
  - `aw.<world_id>.consensus.commit`
- 载荷：复用既有 `GossipProposalMessage/GossipAttestationMessage/GossipCommitMessage` JSON 编码。

### 3) Node 主循环广播/消费优先级
- 广播优先级：
  - 有 libp2p 共识 endpoint：走 libp2p。
  - 否则走 UDP gossip。
- 消费优先级：
  - 两路均可 ingest；libp2p 路径用于生产默认主链路，UDP 作为兼容链路。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 文档冻结（设计 + 项管）。
  - M2：T1 libp2p request 路由升级。
  - M3：T2 node 共识消息 libp2p 主路径接线。
  - M4：T3 测试回归、文档和 devlog 收口。
- Technical Risks:
  - 网络行为风险：多 peer 重试引入请求状态机复杂度，需严格处理 response/outbound-failure 的 pending 迁移。
  - 兼容风险：共识消息双路径（libp2p/UDP）并存阶段可能出现重复消息，需依赖现有高度/哈希幂等守卫。
  - 稳定性风险：topic ingest 新增后测试时序更敏感，需避免 flaky 断言。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-086-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-086-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
