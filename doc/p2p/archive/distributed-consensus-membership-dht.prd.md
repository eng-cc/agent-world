> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式成员目录 DHT 快照与恢复

## 1. Executive Summary
- Problem Statement: 将成员目录（validators + quorum）写入 DHT，减少节点离线后仅依赖 gossip 才能追平目录的问题。
- Proposed Solution: 在节点重启或冷启动阶段，从 DHT 读取最近成员目录并恢复到本地 `QuorumConsensus`。
- Success Criteria:
  - SC-1: 保持与现有成员广播同步链路兼容：广播用于实时收敛，DHT 用于恢复兜底。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：分布式成员目录 DHT 快照与恢复 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 扩展 `DistributedDht`：支持成员目录快照 `put/get`。
  - AC-2: 为协议命名补充成员目录 DHT key：`/aw/world/<world_id>/membership`。
  - AC-3: 在 `MembershipSyncClient` 新增能力：
  - AC-4: `publish_membership_change_with_dht`：广播后同步写入 DHT 快照。
  - AC-5: `restore_membership_from_dht`：从 DHT 读取并应用 `ReplaceValidators` 恢复。
  - AC-6: 覆盖单测：DHT 快照存取、发布落盘、缺省恢复、恢复应用。
- Non-Goals:
  - 成员目录多版本历史与回滚。
  - 成员目录签名验真与反重放窗口。
  - 周期性 DHT 快照压缩或多副本冗余策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-dht.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-dht.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### DHT Key
- `dht_membership_key(world_id) -> /aw/world/<world_id>/membership`

### DHT 快照结构
- `MembershipDirectorySnapshot`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `reason`
  - `validators`
  - `quorum_threshold`

### DHT 抽象
- `DistributedDht::put_membership_directory(world_id, snapshot)`
- `DistributedDht::get_membership_directory(world_id)`

### 成员同步客户端
- `MembershipSyncClient::publish_membership_change_with_dht(...)`
- `MembershipSyncClient::restore_membership_from_dht(...)`

### 恢复规则
- DHT 存在快照：转换为 `ReplaceValidators` 请求并调用 `QuorumConsensus::apply_membership_change`。
- DHT 无快照：返回 `None`，保持本地成员目录不变。
- 若本地存在 pending 共识记录，沿用既有保护规则拒绝目录切换。

## 5. Risks & Roadmap
- Phased Rollout:
  - **MD1**：扩展 DHT 成员目录快照接口与 key helper。
  - **MD2**：实现 membership 广播 + DHT 快照联动写入。
  - **MD3**：实现从 DHT 恢复成员目录能力。
  - **MD4**：补齐单元测试并更新项目文档/开发日志。
- Technical Risks:
  - 当前恢复逻辑默认信任 DHT 最近记录，恶意写入防护依赖后续签名机制。
  - 仅保留最新快照，不含历史演进链路，排障时需要结合 devlog/治理日志。
  - 离线恢复和实时广播在极端网络抖动下可能短暂不一致，需依赖幂等覆盖收敛。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-006-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-006-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
