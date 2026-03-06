# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 6）设计文档

审计轮次: 3

## 1. Executive Summary
- Problem Statement: 在 Phase 5 signer 公钥白名单治理基础上，补齐**生产可运维**所需的策略配置校验能力。
- Proposed Solution: 将 `accepted_signature_signer_public_keys` 从“原样字符串比较”升级为“规范化后比较”，降低大小写与格式差异导致的误拒绝风险。
- Success Criteria:
  - SC-1: 对策略误配（空白、非法 hex、长度错误、重复键）实现 fail-fast，避免在运行时静默退化。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 6）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP6-1：策略校验与规范化实现**
  - AC-2: 新增 membership policy 校验入口：
  - AC-3: `validate_membership_snapshot_restore_policy(...)`
  - AC-4: `validate_membership_revocation_sync_policy(...)`
  - AC-5: 对 `accepted_signature_signer_public_keys` 做统一规范化：
  - AC-6: trim 后不能为空。
- Non-Goals:
  - CA/证书链、公钥托管、轮换审批流程与审计平台。
  - membership 协议模型扩展（仍使用现有 `signature` 字符串字段）。
  - 对 `signature_key_id` 治理语义重写。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增策略校验入口（crate 内部）
```rust
validate_membership_snapshot_restore_policy(
    policy: &MembershipSnapshotRestorePolicy,
) -> Result<(), WorldError>

validate_membership_revocation_sync_policy(
    policy: &MembershipRevocationSyncPolicy,
) -> Result<(), WorldError>
```

### signer 公钥比较语义
- 当策略白名单非空时：
  - 签名必须是 `ed25519:v1:<public_key_hex>:<signature_hex>`。
  - `<public_key_hex>` 按 32-byte hex 解析并规范化为小写 hex。
  - 与策略白名单规范化集合比较（大小写无关）。

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP6-M0**：设计文档 + 项目管理文档。
  - **HP6-M1**：策略校验与规范化实现并接线。
  - **HP6-M2**：测试回归、文档状态与 devlog 收口。
- Technical Risks:
  - 严格校验会把历史脏配置显式暴露为错误，升级初期可能触发“从隐式容忍到显式拒绝”的运维告警。
  - 策略白名单较大时每次同步/恢复的规范化成本上升，需要保持实现轻量（集合构建与去重一次完成）。
  - 错误信息需保持稳定可读，避免排障成本上升。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-050-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-050-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
