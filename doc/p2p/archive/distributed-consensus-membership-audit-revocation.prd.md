> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录审计持久化与吊销传播

## 1. Executive Summary
- Problem Statement: 将成员目录恢复审计结果持久化，形成可追溯的恢复记录链。
- Proposed Solution: 增加签名密钥吊销传播机制，在多节点间同步失效 key_id。
- Success Criteria:
  - SC-1: 在恢复校验阶段拒绝已吊销 key_id，降低已泄露密钥继续生效的风险。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录审计持久化与吊销传播 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增审计存储抽象与内存参考实现：
  - AC-2: `MembershipAuditStore`（append/list）
  - AC-3: `InMemoryMembershipAuditStore`
  - AC-4: 新增“恢复 + 审计持久化”入口：
  - AC-5: `restore_membership_from_dht_verified_with_audit_store(...)`
  - AC-6: 新增密钥吊销广播结构与同步能力：
- Non-Goals:
  - 审计日志外部后端（数据库/对象存储）落地。
  - 吊销消息跨 world 的层级管理与租户隔离策略。
  - 基于硬件可信根（HSM/KMS）的签名密钥托管。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-audit-revocation.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-audit-revocation.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 审计持久化
- `trait MembershipAuditStore`：
  - `append(record: &MembershipSnapshotAuditRecord)`
  - `list(world_id: &str)`
- `MembershipSyncClient::restore_membership_from_dht_verified_with_audit_store(...)`
  - 先执行现有 restore+audit
  - 再将 `audit` 写入 store

### 吊销传播
- gossipsub topic：`aw.<world_id>.membership.revoke`
- `MembershipKeyRevocationAnnounce` 字段：
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `key_id`
  - `reason`
- `MembershipSyncClient` 能力：
  - `publish_key_revocation(...)`
  - `drain_key_revocations(...)`
  - `sync_key_revocations(...)`

### keyring 校验
- `MembershipDirectorySignerKeyring` 新增：
  - `revoke_key(key_id)`
  - `is_key_revoked(key_id)`
  - `revoked_keys()`
- 验签路径在以下场景拒绝吊销 key：
  - 快照显式带 `signature_key_id`
  - 按 keyring 遍历尝试时命中吊销 key

### 恢复策略
- `MembershipSnapshotRestorePolicy.revoked_signature_key_ids: Vec<String>`
- restore 校验时若快照 key_id 在策略吊销名单中，直接拒绝。

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：设计文档与项目管理文档完成。
  - **MR2**：审计持久化抽象、恢复入口与测试完成。
  - **MR3**：吊销传播协议、keyring 吊销校验与测试完成。
  - **MR4**：回归验证、总文档和开发日志更新。
- Technical Risks:
  - 内存审计存储仅用于进程级验证，生产需要替换持久化后端。
  - 吊销消息本身若无额外认证，仍依赖上层 requester 信任策略。
  - 吊销同步存在传播延迟窗口，需结合恢复策略吊销名单兜底。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-004-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-004-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
