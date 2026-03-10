# 区块链 + P2PFS 硬改造 Phase 4 设计

- 对应需求文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.project.md`

## 1. 设计定位
定义 membership 签名双栈（HMAC + ed25519）和发布/同步路径接入方案，为后续 signer 信任根治理建立可兼容的密钥管理底座。

## 2. 设计结构
- 双栈 key 管理层：并存 HMAC 与 ed25519 签名路径。
- membership 发布层：membership 发布和同步链路接入双栈签名。
- 兼容迁移层：在不破坏历史 HMAC 路径的前提下引入 ed25519。
- 回归验证层：membership 相关测试覆盖双栈行为。

## 3. 关键接口 / 入口
- `membership.rs`
- `membership_logic.rs`
- HMAC + ed25519 双栈 key 管理
- membership 发布/同步路径

## 4. 约束与边界
- 本阶段只做双栈接入，不直接引入 signer 公钥白名单治理。
- 签名字段协议保持当前语义，不重写整体 membership 模型。
- 历史 HMAC 路径必须继续兼容。
- 回归需覆盖双栈切换与共存场景。

## 5. 设计演进计划
- 先建立双栈 key 管理。
- 再把 membership 发布/同步接到双栈签名。
- 最后通过测试与日志收口 phase4。
