# Agent World Runtime：节点贡献积分多节点闭环测试（设计文档）

## 目标
- 为“区块链 + P2P FS 场景下，提供算力和存储节点获得收益积分”补齐多节点闭环测试。
- 验证节点积分引擎在多节点输入下的关键经济语义：
  - 额外算力贡献获得收益；
  - 存储贡献获得收益；
  - 基础义务不足/低可靠性节点被惩罚；
  - epoch 奖池分配守恒与跨 epoch 累计正确。

## 范围

### In Scope
- 基于 `NodePointsLedger` 构建多节点（>=3）两轮 epoch 的闭环测试。
- 覆盖 compute/storage/uptime/reliability/penalty 的组合场景。
- 验证结算输出：`distributed_points`、排序、惩罚效果、`cumulative_points`。

### Out of Scope
- 接入真实网络证明链路（PoSt/挑战响应）。
- 与 viewer UI 的积分展示联动。
- 代币化清算与兑换逻辑。

## 接口 / 数据
- 使用现有接口：
  - `NodePointsConfig`
  - `NodeContributionSample`
  - `NodePointsLedger::settle_epoch`
  - `EpochSettlementReport`
- 多节点样本组合：
  - 节点 A：高额外计算 + 中等存储 + 高在线；
  - 节点 B：中计算 + 高存储 + 高在线；
  - 节点 C：高计算但义务不足且惩罚高。

## 里程碑
- NCPM-1：补齐设计文档与项目管理文档。
- NCPM-2：实现多节点闭环测试用例。
- NCPM-3：执行 test_tier_required 回归并收口文档/devlog。

## 风险
- 测试若依赖精确浮点值，未来调权重可能导致脆弱；应优先断言业务不变量（守恒、排名、惩罚生效、累计单调）。
- 单一测试覆盖不足以替代真实网络闭环，后续仍需将贡献采样接入 runtime/node 实际数据面。
