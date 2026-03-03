> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式成员目录同步与变更广播

## 1. Executive Summary
- Problem Statement: 在多节点部署中同步共识验证者目录，避免各节点成员视图分叉。
- Proposed Solution: 将本地成员变更结果广播到网络，让其他节点可按同一目录收敛。
- Success Criteria:
  - SC-1: 提供可测试的订阅、发布、同步闭环接口，便于在 InMemory/libp2p 网络下复用。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式成员目录同步与变更广播 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增成员目录广播 topic：`aw.<world_id>.membership`。
  - AC-2: 定义成员目录广播消息结构 `MembershipDirectoryAnnounce`。
  - AC-3: 提供 `MembershipSyncClient`：
  - AC-4: 发布成员变更结果广播。
  - AC-5: 订阅并 drain 广播消息。
  - AC-6: 将广播目录同步到本地 `QuorumConsensus`。
- Non-Goals:
  - 成员目录的 DHT 持久化与历史版本追踪。
  - 成员变更的签名验真与反重放窗口。
  - 跨世界批量同步与压缩广播。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-sync.prd.md`
  - `doc/p2p/archive/distributed-consensus-sync.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### Topic
- `topic_membership(world_id) -> aw.<world_id>.membership`

### 广播消息
- `MembershipDirectoryAnnounce`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `reason`
  - `validators`
  - `quorum_threshold`

### 同步客户端
- `MembershipSyncClient::publish_membership_change(world_id, request, result)`
- `MembershipSyncClient::subscribe(world_id)`
- `MembershipSyncClient::drain_announcements(subscription)`
- `MembershipSyncClient::sync_membership_directory(subscription, consensus)`

### 同步规则
- 同步时将广播目录转换为 `ReplaceValidators` 请求并调用 `QuorumConsensus::apply_membership_change`。
- 若目录已一致，返回 `ignored`（幂等）；若目录变更成功，计入 `applied`。
- 若本地存在 pending 提案，沿用共识层保护策略，阻断目录切换。

## 5. Risks & Roadmap
- Phased Rollout:
  - **CS1**：定义成员目录 topic 与广播数据结构。
  - **CS2**：实现发布/订阅与 drain。
  - **CS3**：实现目录同步应用与幂等处理。
  - **CS4**：单元测试与项目文档/日志更新。
- Technical Risks:
  - 当前广播消息未做签名验真，仍需依赖上层可信网络与租约约束。
  - 若节点长期离线，恢复后需要额外补偿同步（后续可结合 DHT 快照）。
  - pending 提案期间拒绝目录切换会降低灵活性，但可避免中间态不一致。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-037-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-037-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
