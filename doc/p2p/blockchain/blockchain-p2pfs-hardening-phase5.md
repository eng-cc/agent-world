# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 5）设计文档

## 目标
- 在 membership 签名双栈（HMAC + ed25519）基础上，增加“签名公钥信任根”治理能力。
- 支持按策略限制可接受的 ed25519 signer 公钥，降低伪造 signer 身份风险。
- 保持历史 HMAC 路径兼容；仅在配置策略要求时强制 ed25519 公钥白名单校验。

## 范围

### In Scope
- **HP5-1：策略模型扩展**
  - `MembershipSnapshotRestorePolicy` 新增 `accepted_signature_signer_public_keys`。
  - `MembershipRevocationSyncPolicy` 新增 `accepted_signature_signer_public_keys`。
  - 新增策略语义：
    - 为空时：保持现有行为（不按 signer 公钥过滤）。
    - 非空时：要求签名必须是 `ed25519:v1` 且 signer 公钥命中白名单。

- **HP5-2：校验逻辑接线**
  - 在 `membership_logic::validate_membership_snapshot` / `validate_key_revocation` 接入 signer 公钥白名单校验。
  - 白名单不命中、签名格式非 ed25519、缺失 signer 公钥时返回明确拒绝原因。

- **HP5-3：测试与回归**
  - 新增 membership 测试覆盖：
    - 白名单命中通过。
    - 白名单不命中拒绝。
    - 白名单开启但 HMAC 签名拒绝。
  - 执行 `agent_world_consensus` 回归与 `agent_world` `test_tier_required` 回归。

### Out of Scope
- 证书链/CA 信任体系、公钥轮换审计平台、硬件密钥托管。
- 协议字段变更（`signature` 仍为单字段字符串）。
- 对已有 key_id 治理策略的语义重写。

## 接口 / 数据

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

## 里程碑
- **HP5-M0**：设计文档 + 项目管理文档。
- **HP5-M1**：策略字段与校验逻辑实现。
- **HP5-M2**：测试与回归收口。

## 风险
- 白名单配置错误会导致合法签名被拒绝，需要在运行手册明确配置规范。
- 双栈兼容阶段若未启用白名单，安全性仍依赖既有 key_id/keyring 配置质量。
- 签名字符串解析错误需要稳定错误口径，避免运维排障困难。
