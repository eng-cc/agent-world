> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离

## 1. Executive Summary
- Problem Statement: 新建独立 crate：`agent_world_proto`，专门承载分布式相关的**协议类型定义**与**抽象 trait**。
- Proposed Solution: 降低 `agent_world` runtime 对协议定义层的耦合，避免“核心运行时 + 协议契约 + 网络实现”长期混杂。
- Success Criteria:
  - SC-1: 为后续将 `libp2p` 适配器、分布式执行链路进一步拆 crate 提供稳定边界。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：`agent_world_proto` 协议类型与 Trait 抽离 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新建 `crates/agent_world_proto` 并加入 workspace。
  - AC-2: 迁移以下协议类型定义：
  - AC-3: topic / DHT key 约定、RR 协议名、错误码、请求响应结构。
  - AC-4: `WorldBlock` / `WorldHeadAnnounce` / `ActionEnvelope` / `SnapshotManifest` 等协议载荷。
  - AC-5: 共识成员变更相关协议载荷：`ConsensusMembershipChange*`、`ConsensusStatus`、`ConsensusVote`、`HeadConsensusRecord`。
  - AC-6: 迁移以下协议 trait 定义：
- Non-Goals:
  - `libp2p_net` 独立成单独 crate。
  - observer/bootstrap/validation 等与 `World` 强耦合逻辑拆出 `agent_world`。
  - 网络与 DHT 错误模型的语义重构。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/agent-world-proto.prd.md`
  - `doc/p2p/archive/agent-world-proto.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新 crate 结构（拟）
- `agent_world_proto::distributed`
  - 协议常量与命名 helper。
  - gossipsub / RR / DHT key 相关类型。
  - 分布式错误码与错误响应结构。
- `agent_world_proto::distributed_net`
  - `NetworkMessage` / `NetworkRequest` / `NetworkResponse` / `NetworkSubscription`。
  - `DistributedNetwork<E>` trait（错误类型泛型）。
- `agent_world_proto::distributed_dht`
  - `ProviderRecord` / `MembershipDirectorySnapshot`。
  - `DistributedDht<E>` trait（错误类型泛型）。
- `agent_world_proto::distributed_consensus`
  - 成员变更协议结构：`ConsensusMembershipChange*`。
  - 共识状态/投票与记录：`ConsensusStatus` / `ConsensusVote` / `HeadConsensusRecord`。

### 兼容策略
- `agent_world` 继续对外导出原有 runtime 命名：
  - `runtime::distributed`、`runtime::distributed_net`、`runtime::distributed_dht`。
- `agent_world` 内部使用 wrapper trait 将错误类型固定为 `WorldError`，尽量减少调用点改动。

## 5. Risks & Roadmap
- Phased Rollout:
  - **P1**：文档与任务拆解完成。
  - **P2**：`agent_world_proto` crate 新建并完成协议类型迁移。
  - **P3**：trait 迁移 + `agent_world` wrapper 适配 + 编译/测试回归通过。
  - **P4**：补齐共识成员变更协议类型迁移并保持 `agent_world` 外部 API 稳定。
- Technical Risks:
  - trait 泛型化后若 wrapper 不完整，可能导致 trait object 推断歧义。
  - 协议类型迁移后若 re-export 漏项，可能引发大量编译失败。
  - 现有测试对模块路径较敏感，需注意保持 API 路径稳定。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-001-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-001-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
