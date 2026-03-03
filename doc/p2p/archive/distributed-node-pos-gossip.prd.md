> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node PoS Gossip 协同

## 1. Executive Summary
- Problem Statement: 在 `agent_world_node` 的 PoS 主循环基础上，新增跨进程 gossip 协同能力。
- Proposed Solution: 支持多个节点实例通过 UDP 交换已提交 head 摘要，实现网络视角的 head 追踪。
- Success Criteria:
  - SC-1: 在 `world_viewer_live` 启动参数中暴露 gossip 配置，便于本地多节点联调。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Node PoS Gossip 协同 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `agent_world_node`：
  - AC-2: 增加 gossip 配置（绑定地址、peer 列表）。
  - AC-3: 增加 UDP gossip endpoint（非阻塞接收 + 广播发送）。
  - AC-4: 在 PoS tick 中广播本地 committed head，并摄取远端 committed head。
  - AC-5: 在 snapshot 中暴露网络维度状态（network committed height / known peer heads）。
  - AC-6: `world_viewer_live`：
- Non-Goals:
  - 完整分布式 attestation 传播与跨节点提案驱动。
  - 网络层安全（签名校验、重放防护、加密传输）。
  - P2P 发现、NAT 穿透与生产级拓扑管理。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-node-pos-gossip.prd.md`
  - `doc/p2p/archive/distributed-node-pos-gossip.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### NodeGossipConfig
- `bind_addr: SocketAddr`
- `peers: Vec<SocketAddr>`

### Gossip 消息（提交摘要）
- `version`
- `world_id`
- `node_id`
- `height`
- `slot`
- `epoch`
- `block_hash`
- `committed_at_ms`

### Snapshot 增强
- `network_committed_height`
- `known_peer_heads`

## 5. Risks & Roadmap
- Phased Rollout:
  - NPG-1：设计文档与项目管理文档落地。
  - NPG-2：`agent_world_node` 实现 gossip endpoint 与状态同步。
  - NPG-3：`world_viewer_live` 增加 gossip CLI 接线与测试。
  - NPG-4：回归测试、文档状态与 devlog 收口。
- Technical Risks:
  - UDP 天然不保证可靠投递，网络视角可能短暂滞后。
  - 当前 gossip 仅传播 committed 摘要，不包含提案/投票细节。
  - 本地多节点同机联调可能遇到端口冲突，需要参数校验与错误提示。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-041-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-041-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
