# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 5）设计文档

## 1. Executive Summary
- Problem Statement: 在 membership 签名双栈（HMAC + ed25519）基础上，增加“签名公钥信任根”治理能力。
- Proposed Solution: 支持按策略限制可接受的 ed25519 signer 公钥，降低伪造 signer 身份风险。
- Success Criteria:
  - SC-1: 保持历史 HMAC 路径兼容；仅在配置策略要求时强制 ed25519 公钥白名单校验。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 5）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP5-1：策略模型扩展**
  - AC-2: `MembershipSnapshotRestorePolicy` 新增 `accepted_signature_signer_public_keys`。
  - AC-3: `MembershipRevocationSyncPolicy` 新增 `accepted_signature_signer_public_keys`。
  - AC-4: 新增策略语义：
  - AC-5: 为空时：保持现有行为（不按 signer 公钥过滤）。
  - AC-6: 非空时：要求签名必须是 `ed25519:v1` 且 signer 公钥命中白名单。
- Non-Goals:
  - 证书链/CA 信任体系、公钥轮换审计平台、硬件密钥托管。
  - 协议字段变更（`signature` 仍为单字段字符串）。
  - 对已有 key_id 治理策略的语义重写。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 策略新增字段
```rust
MembershipSnapshotRestorePolicy {
  accepted_signature_signer_public_keys: Vec<String>,
  // 其余字段保持不变
}

MembershipRevocationSyncPolicy {
  accepted_signature_signer_public_keys: Vec<String>,
  // 其余字段保持不变
}
```

### 校验语义
- 当 `accepted_signature_signer_public_keys` 非空时：
  - `signature` 必须是 `ed25519:v1:<public_key_hex>:<signature_hex>`。
  - `<public_key_hex>` 必须命中白名单。
  - 之后继续走 signer/keyring 的常规验签。

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP5-M0**：设计文档 + 项目管理文档。
  - **HP5-M1**：策略字段与校验逻辑实现。
  - **HP5-M2**：测试与回归收口。
- Technical Risks:
  - 白名单配置错误会导致合法签名被拒绝，需要在运行手册明确配置规范。
  - 双栈兼容阶段若未启用白名单，安全性仍依赖既有 key_id/keyring 配置质量。
  - 签名字符串解析错误需要稳定错误口径，避免运维排障困难。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-049-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-049-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
