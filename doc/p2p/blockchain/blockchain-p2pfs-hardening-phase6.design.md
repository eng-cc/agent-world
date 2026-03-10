# 区块链 + P2PFS 硬改造 Phase 6 设计

- 对应需求文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.project.md`

## 1. 设计定位
定义 signer 公钥白名单策略的生产可运维硬化方案：把字符串比较升级为规范化后比较，并对误配进行 fail-fast 校验。

## 2. 设计结构
- 策略校验层：为 snapshot restore 和 revocation sync 增加显式 validate 入口。
- 规范化层：白名单公钥统一 trim、hex 校验、长度校验、小写化与去重。
- 兼容比较层：签名公钥按 32-byte hex 规范化后再与白名单比较。
- 错误暴露层：空白、非法 hex、长度错误、重复键都在加载期显式失败。

## 3. 关键接口 / 入口
- `validate_membership_snapshot_restore_policy`
- `validate_membership_revocation_sync_policy`
- `accepted_signature_signer_public_keys`
- 规范化后的小写 hex 集合

## 4. 约束与边界
- 不引入 CA/证书链/公钥托管体系。
- 不扩展 membership 协议模型，继续沿用现有 `signature` 字段。
- 严格校验会把历史脏配置显式暴露出来，这是有意为之。
- 规范化实现要保持轻量，避免同步路径额外放大成本。

## 5. 设计演进计划
- 先补 validate 入口。
- 再实现白名单规范化和大小写无关比较。
- 最后通过误配与兼容测试收口 phase6。
