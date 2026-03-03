> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销来源鉴权与审计落盘归档

## 1. Executive Summary
- Problem Statement: 为成员目录 `membership.revoke` 吊销消息增加来源鉴权能力，降低伪造吊销广播风险。
- Proposed Solution: 在吊销同步阶段提供策略化校验（trusted requester / signature / key_id 白名单与吊销名单）。
- Success Criteria:
  - SC-1: 提供可落盘的审计存储实现，支持跨重启查询成员目录恢复审计记录。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录吊销来源鉴权与审计落盘归档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 扩展 `MembershipKeyRevocationAnnounce`：
  - AC-2: 增加可选 `signature_key_id`
  - AC-3: 增加可选 `signature`
  - AC-4: 扩展签名器与 keyring：
  - AC-5: `MembershipDirectorySigner` 增加吊销消息签名/验签能力
  - AC-6: `MembershipDirectorySignerKeyring` 增加吊销消息签名/验签能力
- Non-Goals:
  - 吊销消息的租约/角色级授权（如必须由 lease holder 发起）。
  - 吊销审计与外部 SIEM/数据库集成。
  - 分布式多副本审计归档一致性协议。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-revocation-auth-archive.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-revocation-auth-archive.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 吊销消息
- `MembershipKeyRevocationAnnounce`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `key_id`
  - `reason`
  - `signature_key_id?`
  - `signature?`

### 吊销同步策略
- `MembershipRevocationSyncPolicy`
  - `trusted_requesters`
  - `require_signature`
  - `require_signature_key_id`
  - `accepted_signature_key_ids`
  - `revoked_signature_key_ids`

### 吊销同步报告
- `MembershipRevocationSyncReport`
  - `drained`
  - `applied`
  - `ignored`
  - `rejected`

### 审计落盘
- `FileMembershipAuditStore`
  - 文件格式：每行一条 `MembershipSnapshotAuditRecord` JSON（JSONL）
  - 路径约定：`<root>/<world_id>.jsonl`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：文档与任务拆解完成。
  - **MR2**：吊销消息签名与来源校验策略落地。
  - **MR3**：吊销同步报告与测试回归通过。
  - **MR4**：审计落盘实现、总文档/日志更新完成。
- Technical Risks:
  - 若各节点 keyring 配置不一致，可能出现“部分节点接受、部分节点拒绝”的吊销分歧。
  - JSONL 落盘适合轻量归档，高并发场景需后续替换专用后端。
  - 鉴权策略配置过严可能导致合法吊销消息被拒收，需要运维流程配套。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-010-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-010-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
