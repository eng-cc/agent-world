# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 3）设计文档

## 目标
- 将 `ActionEnvelope` 与 `WorldHeadAnnounce` 从 HMAC-only 过渡到 ed25519 可验签路径，形成跨 crate 一致的签名语义。
- 保持与历史 HMAC 签名兼容，支持灰度迁移，不打断现有测试与旧数据。
- 在 sequencer 主循环落地签名治理：动作签名校验、head 签名生成、允许名单约束。

## 范围

### In Scope
- **HP3-1：跨 crate ed25519 签名器**
  - 在 `agent_world_consensus` 新增 `Ed25519SignatureSigner`。
  - 支持 `ActionEnvelope`、`WorldHeadAnnounce` 的签名与验签。
  - 定义统一签名串格式（不新增协议字段，复用 `signature` 字段）。

- **HP3-2：Sequencer 签名治理接线**
  - `SequencerMainloopConfig` 增加 ed25519 配置项与动作签名允许名单。
  - `submit_action` 验签路径支持 ed25519 与 HMAC 双栈。
  - `sign_head` 支持优先使用 ed25519 签名，兼容 HMAC。

- **HP3-3：测试与回归**
  - 增加签名器单测：签名回放、篡改拒绝、格式非法拒绝。
  - 增加 sequencer 单测：ed25519 签名动作接收、允许名单拒绝、head 签名生成。
  - 保持 `test_tier_required` 回归通过。

### Out of Scope
- membership 目录签名体系从 HMAC 迁移到 ed25519。
- 链上信任根、证书吊销、多签聚合与 HSM/KMS 托管。
- `agent_world_proto` 协议字段改版（本阶段不新增 `signature_scheme` 字段）。

## 接口 / 数据

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

## 里程碑
- **HP3-M0**：设计文档 + 项目管理文档。
- **HP3-M1**：ed25519 签名器落地并完成单测。
- **HP3-M2**：sequencer 双栈验签/签名策略接线。
- **HP3-M3**：回归测试、文档状态与 devlog 收口。

## 风险
- 若启用 `require_action_signature` 但未配置允许名单或签名器，可能导致动作全拒绝；需在配置校验阶段提前报错。
- HMAC 与 ed25519 双栈并存阶段需保证判定顺序稳定，避免同一输入在不同节点出现分歧。
- 签名串复用单字段存储，后续若升级协议字段需明确版本迁移策略。
