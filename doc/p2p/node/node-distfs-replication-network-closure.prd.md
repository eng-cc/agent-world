# oasis7 Runtime：Node DistFS 复制网络化收敛

- 对应设计文档: `doc/p2p/node/node-distfs-replication-network-closure.design.md`
- 对应项目管理文档: `doc/p2p/node/node-distfs-replication-network-closure.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 `agent_world_distfs` 中已具备的 `FileReplicationRecord` 能力接入 node 运行时网络路径，形成“可广播、可验签、可恢复”的最小跨节点复制闭环。
- Proposed Solution: 复用现有 node UDP gossip 主循环，避免引入第二套并行传输栈。
- Success Criteria:
  - SC-1: 以最小代价把 `config.toml` 中的节点密钥用于复制消息签名链路。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Node DistFS 复制网络化收敛 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **NRX-1：复制消息网络接线**
  - AC-2: 扩展 node gossip 消息模型，新增 DistFS 复制消息分支。
  - AC-3: 节点在本地 committed 事件时产出复制记录并广播。
  - AC-4: 远端节点接收并应用复制记录到本地 CAS/FileStore。
  - AC-5: 单写者 guard 持久化（重启后可恢复）。
  - AC-6: **NRX-2：复制消息签名/验签**
- Non-Goals:
  - 生产级密钥分发与 PKI 信任管理。
  - 多写者 CRDT 合并协议。
  - 与 libp2p/kad 的完整复制索引协议。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-distfs-replication-network-closure.prd.md`
  - `doc/p2p/node/node-distfs-replication-network-closure.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- `NodeReplicationConfig`：节点复制配置（路径、签名密钥、状态持久化文件）。
- `NodeConfig::with_replication(...)`：启用复制能力。
- gossip 复制消息数据：`FileReplicationRecord + bytes + signature + public_key`。
- 持久化数据：
  - guard 状态（`SingleWriterReplicationGuard`）
  - 本地 writer 序列号状态（单调递增）

## 5. Risks & Roadmap
- Phased Rollout:
  - **NRX-0**：设计文档 + 项目管理文档。
  - **NRX-1**：复制消息网络接线 + guard 持久化。
  - **NRX-2**：复制消息签名验签接线（消费 config 节点密钥）。
  - **NRX-3**：多节点与重启恢复测试收口。
- Technical Risks:
  - `crates/agent_world_node/src/lib.rs` 文件接近 1200 行，需拆分测试/模块防止超限。
  - 当前 node 主循环以最小 UDP gossip 实现，吞吐与可靠性有限；本阶段仅保证功能闭环。
  - 验签策略先做最小可用，后续仍需补充节点身份绑定策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-092-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-092-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
