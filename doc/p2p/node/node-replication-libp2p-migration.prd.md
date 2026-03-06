# Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 `crates/agent_world_node` 的 DistFS 复制消息从“仅 UDP gossip”迁移为“优先使用 `distributed_net` 统一网络抽象（可接入 libp2p 实现）”。
- Proposed Solution: 保持现有 UDP 共识提交广播不变，先迁移复制数据通道，降低改造风险。
- Success Criteria:
  - SC-1: 提供可由上层集成的复制 topic 与网络注入能力，支持多节点复制走统一网络栈。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Node Replication 迁移到 libp2p 统一网络栈 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **NRM-1：Node 复制网络桥接**
  - AC-2: 在 `NodeRuntime` 增加可注入 `DistributedNetwork` 的复制通道。
  - AC-3: 复制消息发布/订阅走统一 topic；未配置网络时保持 UDP 复制回退。
  - AC-4: 增加基于 `InMemoryNetwork` 的复制回归测试。
  - AC-5: **NRM-2：Node 侧外部注入接线增强**
  - AC-6: 增加 replication topic 配置能力，支持按 world 隔离网络通道。
- Non-Goals:
  - 将 PoS commit gossip 也迁移到 libp2p（本轮仅复制消息迁移）。
  - DHT/provider 索引协议重构。
  - 生产级 NAT 穿透与复杂拓扑自动发现策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-replication-libp2p-migration.prd.md`
  - `doc/p2p/node/node-replication-libp2p-migration.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- `NodeRuntime::with_replication_network(...)`：注入统一网络对象。
- `NodeReplicationNetworkHandle::with_topic(...)`：配置复制 topic。
- 复制 topic：`aw.<world_id>.replication`（默认）。
- 复制消息仍沿用 `GossipReplicationMessage` 结构，序列化为 JSON payload 发布到 topic。

## 5. Risks & Roadmap
- Phased Rollout:
  - **NRM-0**：设计文档 + 项目管理文档。
  - **NRM-1**：Node 支持统一网络复制通道 + InMemory 回归。
  - **NRM-2**：Node 外部注入接线增强（topic 配置）。
  - **NRM-3**：回归收口与文档状态完成。
  - **NRM-4**：crate 路径标准化（`crates/agent_world_node`）。
- Technical Risks:
  - `crates/agent_world_node/src/lib.rs` 行数压力较高，改动需继续控制在 1200 行以下。
  - 双通道（UDP+网络）并存期间需避免重复应用同一复制记录，依赖单调序列守卫兜底。
  - 工作区存在 `agent_world_net -> agent_world -> agent_world_node` 依赖链约束，上层 libp2p 适配需避免形成反向循环依赖。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-099-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-099-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
