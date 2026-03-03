> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录快照密钥轮换与审计

## 1. Executive Summary
- Problem Statement: 让成员目录快照签名支持 `key_id`，使签名密钥可以在不破坏恢复能力的前提下轮换。
- Proposed Solution: 提供“多密钥验签”能力：新旧密钥共存期可同时验证历史快照与新快照。
- Success Criteria:
  - SC-1: 为恢复链路增加结构化审计输出，记录恢复成功/忽略/拒绝/缺失等状态。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录快照密钥轮换与审计 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在成员目录广播与 DHT 快照结构中增加 `signature_key_id`（可选，向后兼容）。
  - AC-2: 新增 `MembershipDirectorySignerKeyring`：
  - AC-3: 管理多把签名密钥。
  - AC-4: 指定 active key。
  - AC-5: 支持按 `key_id` 签名、按 `key_id` 验签、兼容无 `key_id` 的历史签名。
  - AC-6: 扩展恢复策略：
- Non-Goals:
  - 自动密钥下发与远程 KMS 集成。
  - 密钥失效吊销广播协议。
  - 审计日志持久化后端（当前仅结构化返回）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-rotation-audit.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-rotation-audit.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 数据结构
- `MembershipDirectorySnapshot.signature_key_id: Option<String>`
- `MembershipDirectoryAnnounce.signature_key_id: Option<String>`

### Keyring
- `MembershipDirectorySignerKeyring::add_hmac_sha256_key(key_id, key)`
- `MembershipDirectorySignerKeyring::set_active_key(key_id)`
- `MembershipDirectorySignerKeyring::sign_snapshot_with_active_key(snapshot)`
- `MembershipDirectorySignerKeyring::verify_snapshot(snapshot)`

### 恢复策略
- `MembershipSnapshotRestorePolicy`
  - `trusted_requesters`
  - `require_signature`
  - `require_signature_key_id`
  - `accepted_signature_key_ids`

### 审计输出
- `MembershipSnapshotAuditOutcome`：`missing_snapshot/applied/ignored/rejected`
- `MembershipSnapshotAuditRecord`
- `MembershipRestoreAuditReport`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MR1**：数据结构补充 `signature_key_id` 并保持兼容。
  - **MR2**：实现 keyring 轮换签名/验签能力。
  - **MR3**：恢复策略补充 key_id 控制项。
  - **MR4**：恢复审计输出与测试回归。
- Technical Risks:
  - keyring 仅进程内配置，跨节点配置不一致会导致恢复拒绝。
  - 兼容无 `key_id` 历史签名时，策略若过宽可能降低约束力度。
  - 审计结果目前未自动落盘，仍需上层系统收集归档。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-034-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-034-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
