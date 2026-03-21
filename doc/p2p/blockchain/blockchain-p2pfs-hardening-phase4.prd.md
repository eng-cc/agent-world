# oasis7 Runtime：区块链 + P2P FS 硬改造（Phase 4）设计文档

- 对应设计文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 membership 目录（snapshot/revocation）签名从 HMAC-only 扩展为 ed25519 + HMAC 双栈，补齐跨模块签名口径。
- Proposed Solution: 在不破坏历史数据与既有流程前提下，支持 keyring 管理 ed25519 签名 key，完成签发与验签闭环。
- Success Criteria:
  - SC-1: 为后续“统一公钥信任根 + HSM/KMS”留出接口，不在本期引入协议字段破坏性改动。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want oasis7 Runtime：区块链 + P2P FS 硬改造（Phase 4）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP4-1：MembershipSigner 增强**
  - AC-2: `MembershipDirectorySigner` 支持两种实现：
  - AC-3: `hmac_sha256`
  - AC-4: `ed25519`
  - AC-5: 保持既有 API（`sign_snapshot/verify_snapshot`、`sign_revocation/verify_revocation`）不变。
  - AC-6: ed25519 签名串采用统一格式：`ed25519:v1:<public_key_hex>:<signature_hex>`。
- Non-Goals:
  - membership 网络消息新增 `signature_scheme` 字段（本期不改 wire schema）。
  - KMS/HSM、远程签名服务、多签策略与证书链验证。
  - 撤销列表的全网强一致传播机制改造（仅沿用现有同步流程）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) Membership 签名串（新增）
```text
ed25519:v1:<public_key_hex>:<signature_hex>
```

### 2) Signer 构造（扩展）
```rust
impl MembershipDirectorySigner {
  pub fn hmac_sha256(key: impl Into<Vec<u8>>) -> Self;
  pub fn ed25519(private_key_hex: &str, public_key_hex: &str) -> Result<Self, WorldError>;
}
```

### 3) Keyring 注入（扩展）
```rust
impl MembershipDirectorySignerKeyring {
  pub fn add_hmac_sha256_key(&mut self, key_id: impl Into<String>, key: impl Into<Vec<u8>>) -> Result<(), WorldError>;
  pub fn add_ed25519_key(&mut self, key_id: impl Into<String>, private_key_hex: &str, public_key_hex: &str) -> Result<(), WorldError>;
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP4-M0**：设计文档与项目管理文档。
  - **HP4-M1**：membership signer ed25519 扩展与单测。
  - **HP4-M2**：keyring 双栈支持与 sync/publish 验证闭环。
  - **HP4-M3**：回归测试、文档状态回写、devlog 收口。
- Technical Risks:
  - 双栈并存阶段若 key_id 配置与签名格式不匹配，可能造成验签拒绝；需保证错误原因可诊断。
  - 历史无 `signature_key_id` 的消息在 keyring“逐 key 尝试”下可能增加验签开销；本期保持正确性优先。
  - 若外部系统将签名字段视为“纯 hex”，ed25519 前缀格式可能触发兼容问题，需要在集成侧同步约束。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-048-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-048-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
