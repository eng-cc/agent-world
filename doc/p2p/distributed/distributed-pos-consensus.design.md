# 分布式 PoS 共识设计

- 对应需求文档: `doc/p2p/distributed/distributed-pos-consensus.prd.md`
- 对应项目管理文档: `doc/p2p/distributed/distributed-pos-consensus.project.md`

## 1. 设计定位
定义分布式 PoS 共识能力的统一方案，为节点出块、投票、提交和状态演进提供稳定主线。

## 2. 设计结构
- 提案投票层：围绕 proposal/attestation/commit 建立基本循环。
- 状态演进层：高度、slot、epoch 与提交状态形成可恢复状态机。
- 签名治理层：和节点 signer/validator 绑定及后续安全硬化专题协同。
- 测试闭环层：通过分布式回归冻结 PoS 主路径。

## 3. 关键接口 / 入口
- proposal/attestation/commit
- height/slot/epoch 状态机
- signer/validator 绑定
- 分布式回归测试

## 4. 约束与边界
- 不在此文档展开所有后续安全细节专题。
- 重点是主线闭环，而非所有性能/治理优化。
- PoS 状态需要可恢复、可持久化。
- 与区块链 hardening 系列保持口径一致。

## 5. 设计演进计划
- 先冻结提案投票提交闭环。
- 再补状态持久化与签名治理。
- 最后通过分布式回归收口 PoS 主线。
