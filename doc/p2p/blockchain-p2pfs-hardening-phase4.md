# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 4）设计文档

## 目标
- 将 membership 目录（snapshot/revocation）签名从 HMAC-only 扩展为 ed25519 + HMAC 双栈，补齐跨模块签名口径。
- 在不破坏历史数据与既有流程前提下，支持 keyring 管理 ed25519 签名 key，完成签发与验签闭环。
- 为后续“统一公钥信任根 + HSM/KMS”留出接口，不在本期引入协议字段破坏性改动。

## 范围

### In Scope
- **HP4-1：MembershipSigner 增强**
  - `MembershipDirectorySigner` 支持两种实现：
    - `hmac_sha256`
    - `ed25519`
  - 保持既有 API（`sign_snapshot/verify_snapshot`、`sign_revocation/verify_revocation`）不变。
  - ed25519 签名串采用统一格式：`ed25519:v1:<public_key_hex>:<signature_hex>`。

- **HP4-2：Keyring 与同步链路增强**
  - `MembershipDirectorySignerKeyring` 增加 ed25519 key 注入接口。
  - keyring 的签发与验签逻辑支持双栈，兼容已有 HMAC key。
  - `MembershipSyncClient` 签名发布路径无需协议字段变更即可支持 ed25519 signer。

- **HP4-3：测试与收口**
  - 增加 membership ed25519 签发/验签单测（snapshot/revocation）。
  - 增加 keyring 混合 key（HMAC + ed25519）验证单测。
  - 执行 `agent_world_consensus` 全测与 `agent_world` `test_tier_required` 回归。

### Out of Scope
- membership 网络消息新增 `signature_scheme` 字段（本期不改 wire schema）。
- KMS/HSM、远程签名服务、多签策略与证书链验证。
- 撤销列表的全网强一致传播机制改造（仅沿用现有同步流程）。

## 接口 / 数据

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

## 里程碑
- **HP4-M0**：设计文档与项目管理文档。
- **HP4-M1**：membership signer ed25519 扩展与单测。
- **HP4-M2**：keyring 双栈支持与 sync/publish 验证闭环。
- **HP4-M3**：回归测试、文档状态回写、devlog 收口。

## 风险
- 双栈并存阶段若 key_id 配置与签名格式不匹配，可能造成验签拒绝；需保证错误原因可诊断。
- 历史无 `signature_key_id` 的消息在 keyring“逐 key 尝试”下可能增加验签开销；本期保持正确性优先。
- 若外部系统将签名字段视为“纯 hex”，ed25519 前缀格式可能触发兼容问题，需要在集成侧同步约束。
