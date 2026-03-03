> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 1）设计文档

## 1. Executive Summary
- Problem Statement: 以最小阶段把“基础可用”提升为“可跨进程联调、可网络接线、可扩展”的链路。
- Proposed Solution: 聚焦两个主缺口：
- Success Criteria:
  - SC-1: `world_viewer_live` 上层未接入 `libp2p` 复制网络。
  - SC-2: Node PoS 仍缺 proposal/attestation 的 gossip 传播闭环。
  - SC-3: 在不破坏现有 UDP commit/DistFS 复制闭环的前提下，提供可灰度启用的新路径。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 1）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP1-1：`world_viewer_live` 注入 libp2p replication 网络**
  - AC-2: 新增 CLI 参数用于配置 replication libp2p listen/bootstrap/topic。
  - AC-3: 在启动 node runtime 时创建 `Libp2pNetwork`，并通过 `NodeReplicationNetworkHandle` 注入。
  - AC-4: 保持现有 UDP gossip 路径兼容（可并存，复制优先走注入网络）。
  - AC-5: **HP1-2：Node PoS proposal/attestation gossip 扩展**
  - AC-6: 扩展 UDP gossip 消息类型，增加 proposal/attestation。
- Non-Goals:
  - Action/Head 的 ed25519 全链路签名替换（本阶段不改共识签名模型）。
  - Node PoS 完整状态持久化与重启续跑（另起阶段）。
  - 复制与共识统一到同一 libp2p topic 拓扑（本阶段只扩复制上层接线 + 共识 gossip 扩展）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/blockchain-p2pfs-hardening-phase1.prd.md`
  - `doc/p2p/archive/blockchain-p2pfs-hardening-phase1.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### `world_viewer_live` 新增参数（草案）
- `--node-repl-libp2p-listen <multiaddr>`（可重复）
- `--node-repl-libp2p-peer <multiaddr>`（可重复）
- `--node-repl-topic <topic>`（可选）

### Node Gossip 消息扩展（草案）
- `GossipProposalMessage`
  - `version/world_id/node_id/height/slot/epoch/block_hash/proposed_at_ms`
- `GossipAttestationMessage`
  - `version/world_id/node_id/height/slot/epoch/block_hash/validator_id/approve/source_epoch/target_epoch/voted_at_ms/reason`

### 行为约束
- 仅 `world_id` 匹配的 proposal/attestation 才可被消费。
- attestation 仅作用于当前 pending 且 `height + block_hash` 匹配的提案。
- proposal/attestation 传播不改变现有 committed head 广播语义。

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP1-0**：设计文档 + 项目管理文档。
  - **HP1-1**：`world_viewer_live` libp2p replication 注入完成。
  - **HP1-2**：Node proposal/attestation gossip 扩展完成。
  - **HP1-3**：测试回归、文档与 devlog 收口。
- Technical Risks:
  - `crates/agent_world_node/src/lib.rs` 已接近 1200 行，需要拆分模块防止超限。
  - libp2p 参数配置错误会导致“网络未连通但进程可启动”，需提供明确错误与测试覆盖。
  - proposal/attestation 广播引入后，消息乱序可能造成短时 pending；需保持幂等处理。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-003-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-003-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
