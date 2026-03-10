# 区块链 + P2PFS 硬改造 Phase 2 设计

- 对应需求文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.prd.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.project.md`

## 1. 设计定位
定义区块链 + P2PFS 硬改造第二阶段方案：把节点密钥真正接入 PoS gossip 主链路，并为 Node PoS 增加可恢复的状态持久化。

## 2. 设计结构
- 签名闭环层：proposal/attestation/commit 消息携带公钥与签名，并按策略验签。
- 兼容策略层：未开启签名策略时仍兼容无签名旧消息。
- 状态持久化层：`node_pos_state.json` 记录 next_height/slot/committed_height 等关键状态。
- 恢复启动层：节点重启后从持久化高度继续推进，而不是回到初始高度。

## 3. 关键接口 / 入口
- gossip 消息签名字段
- `node_pos_state.json`
- Node PoS 主循环状态
- 签名策略开关

## 4. 约束与边界
- 不在本阶段重构跨 crate 的统一签名接口。
- observer 运维面板接线推迟到后续阶段。
- 多节点身份治理完整闭环不在本轮范围。
- 持久化频率优先保证正确性，不先追求最优写放大。

## 5. 设计演进计划
- 先补 gossip 签名/验签闭环。
- 再落 Node PoS 状态持久化与恢复。
- 最后通过回归测试和文档收口 phase2。
