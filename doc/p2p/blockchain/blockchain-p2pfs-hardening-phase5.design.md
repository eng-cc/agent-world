# 区块链 + P2PFS 硬改造 Phase 5 设计

- 对应需求文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.project.md`

## 1. 设计定位
定义 signer 公钥信任根治理方案：在 HMAC + ed25519 双栈基础上增加可配置的 signer 公钥白名单，降低伪造 signer 身份风险。

## 2. 设计结构
- 策略扩展层：为 snapshot restore 和 revocation sync 策略增加 `accepted_signature_signer_public_keys`。
- 白名单校验层：白名单非空时要求签名走 `ed25519:v1` 且公钥命中列表。
- 兼容保留层：白名单为空时保持当前行为，不额外按 signer 公钥过滤。
- 错误口径层：签名字符串解析和白名单不命中都返回稳定错误语义。

## 3. 关键接口 / 入口
- `MembershipSnapshotRestorePolicy`
- `MembershipRevocationSyncPolicy`
- `accepted_signature_signer_public_keys`
- `ed25519:v1:<public_key_hex>:<signature_hex>`

## 4. 约束与边界
- 不引入证书链/CA/硬件托管等更重的信任体系。
- 不改协议字段形式，`signature` 仍保持单字符串。
- 双栈兼容阶段白名单为可选增强，而非强制全网切换。
- 对已有 key_id 治理语义不做重写。

## 5. 设计演进计划
- 先补策略字段与白名单语义。
- 再接 signer 公钥校验逻辑。
- 最后通过 membership 回归和文档收口 phase5。
