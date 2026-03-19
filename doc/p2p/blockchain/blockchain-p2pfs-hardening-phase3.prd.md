# oasis7 Runtime：区块链 + P2P FS 硬改造（Phase 3）设计文档

- 对应设计文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 `ActionEnvelope` 与 `WorldHeadAnnounce` 从 HMAC-only 过渡到 ed25519 可验签路径，形成跨 crate 一致的签名语义。
- Proposed Solution: 保持与历史 HMAC 签名兼容，支持灰度迁移，不打断现有测试与旧数据。
- Success Criteria:
  - SC-1: 在 sequencer 主循环落地签名治理：动作签名校验、head 签名生成、允许名单约束。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 3）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP3-1：跨 crate ed25519 签名器**
  - AC-2: 在 `agent_world_consensus` 新增 `Ed25519SignatureSigner`。
  - AC-3: 支持 `ActionEnvelope`、`WorldHeadAnnounce` 的签名与验签。
  - AC-4: 定义统一签名串格式（不新增协议字段，复用 `signature` 字段）。
  - AC-5: **HP3-2：Sequencer 签名治理接线**
  - AC-6: `SequencerMainloopConfig` 增加 ed25519 配置项与动作签名允许名单。
- Non-Goals:
  - membership 目录签名体系从 HMAC 迁移到 ed25519。
  - 链上信任根、证书吊销、多签聚合与 HSM/KMS 托管。
  - `agent_world_proto` 协议字段改版（本阶段不新增 `signature_scheme` 字段）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) 统一签名串格式（新增）
```text
ed25519:v1:<public_key_hex>:<signature_hex>
```
- `public_key_hex`：32-byte ed25519 公钥 hex。
- `signature_hex`：64-byte ed25519 签名字节 hex。
- 被签名载荷：将结构体 `signature` 字段清空后的 canonical CBOR bytes。

### 2) 签名器（新增）
```rust
pub struct Ed25519SignatureSigner { ... }

impl Ed25519SignatureSigner {
  pub fn new(private_key_hex: &str, public_key_hex: &str) -> Result<Self, WorldError>;
  pub fn sign_action(&self, action: &ActionEnvelope) -> Result<String, WorldError>;
  pub fn verify_action(action: &ActionEnvelope) -> Result<String, WorldError>; // 返回 signer public key
  pub fn sign_head(&self, head: &WorldHeadAnnounce) -> Result<String, WorldError>;
  pub fn verify_head(head: &WorldHeadAnnounce) -> Result<String, WorldError>; // 返回 signer public key
}
```

### 3) Sequencer 配置（扩展）
```rust
SequencerMainloopConfig {
  require_action_signature: bool,
  sign_head: bool,
  hmac_signer: Option<HmacSha256Signer>, // 兼容
  ed25519_signer: Option<Ed25519SignatureSigner>, // 新增
  accepted_action_signer_public_keys: Vec<String>, // 新增
}
```

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP3-M0**：设计文档 + 项目管理文档。
  - **HP3-M1**：ed25519 签名器落地并完成单测。
  - **HP3-M2**：sequencer 双栈验签/签名策略接线。
  - **HP3-M3**：回归测试、文档状态与 devlog 收口。
- Technical Risks:
  - 若启用 `require_action_signature` 但未配置允许名单或签名器，可能导致动作全拒绝；需在配置校验阶段提前报错。
  - HMAC 与 ed25519 双栈并存阶段需保证判定顺序稳定，避免同一输入在不同节点出现分歧。
  - 签名串复用单字段存储，后续若升级协议字段需明确版本迁移策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-047-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-047-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
