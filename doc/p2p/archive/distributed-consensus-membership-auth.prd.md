> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录快照签名与来源校验

## 1. Executive Summary
- Problem Statement: 为成员目录 DHT 快照增加签名能力，降低离线恢复时读取伪造快照的风险。
- Proposed Solution: 在恢复入口引入来源校验策略（trusted requester），避免任意节点覆盖本地 validator 目录。
- Success Criteria:
  - SC-1: 保持与现有 P3.12 接口兼容：未启用策略时仍可按旧路径恢复。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：成员目录快照签名与来源校验 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 为成员目录广播/快照增加可选签名字段（向后兼容旧数据）。
  - AC-2: 新增 `MembershipDirectorySigner`（HMAC-SHA256）用于快照签名与验签。
  - AC-3: 新增恢复策略 `MembershipSnapshotRestorePolicy`：
  - AC-4: `trusted_requesters` 来源白名单
  - AC-5: `require_signature` 是否强制签名
  - AC-6: 新增 `restore_membership_from_dht_verified`：在恢复前执行 world/requester/signature 校验。
- Non-Goals:
  - 非对称密钥基础设施（PKI）与节点证书分发。
  - 快照多版本链路、历史追踪与回滚策略。
  - 跨世界全局授权中心与治理投票流程。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-consensus-membership-auth.prd.md`
  - `doc/p2p/archive/distributed-consensus-membership-auth.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 数据结构
- `MembershipDirectorySnapshot` 新增可选字段：
  - `signature: Option<String>`（hex）
- `MembershipDirectoryAnnounce` 新增可选字段：
  - `signature: Option<String>`

### 签名器
- `MembershipDirectorySigner::hmac_sha256(key)`
- `MembershipDirectorySigner::sign_snapshot(snapshot) -> signature_hex`
- `MembershipDirectorySigner::verify_snapshot(snapshot) -> Result<()>`

### 恢复策略
- `MembershipSnapshotRestorePolicy`
  - `trusted_requesters: Vec<String>`
  - `require_signature: bool`

### 同步客户端新增 API
- `publish_membership_change_with_dht_signed(...)`
- `restore_membership_from_dht_verified(...)`

## 5. Risks & Roadmap
- Phased Rollout:
  - **MA1**：补充签名字段并保持序列化兼容。
  - **MA2**：实现快照签名器与签名发布链路。
  - **MA3**：实现恢复阶段来源与签名校验策略。
  - **MA4**：补充测试、更新项目文档与 devlog。
- Technical Risks:
  - HMAC 方案依赖共享密钥管理，密钥泄露会影响信任边界。
  - 仅在恢复入口做校验，实时广播路径默认仍可走兼容模式。
  - `trusted_requesters` 为空时策略退化为宽松模式，部署时需显式配置。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-005-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-005-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
