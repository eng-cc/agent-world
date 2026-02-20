# README 高优先级差距收口（二期）：模块生命周期 + 单一订单簿撮合（设计文档）

## 目标
- 收口差距 A：补齐模块生命周期管理，支持上架撤销（delist）与工件销毁（destroy），并引入可审计成本模型。
- 收口差距 B：落地最小可用单一订单簿（single orderbook）撮合能力，覆盖模块交易与电力交易的挂单/撤单/撮合闭环。
- 保证事件驱动可重放，确保跨节点在同一动作序列下得到一致的撮合结果。

## 范围
- In scope
  - Runtime（`crates/agent_world/src/runtime`）
    - 新增模块生命周期动作：`DelistModuleArtifact`、`DestroyModuleArtifact`。
    - 新增模块撮合动作：`PlaceModuleArtifactBid`、`CancelModuleArtifactBid`。
    - 新增模块市场状态：bid 订单簿、订单序号、成交序号。
    - 新增模块动作成本事件：统一记账到世界资源池（treasury）。
  - Simulator（`crates/agent_world/src/simulator`）
    - 新增电力订单簿动作：`PlacePowerOrder`、`CancelPowerOrder`。
    - 新增单一订单簿状态与撮合事件（价格优先、时间优先、确定性排序）。
    - 撮合成交仍走 `PowerTransferred` 语义，保证与现有功率损耗/动态报价一致。
  - 测试
    - `test_tier_required` 覆盖：生命周期拒绝路径、成本扣费、挂单/撤单/撮合、重放一致性。
- Out of scope
  - 多资产跨簿撮合（仅单簿）。
  - 跨节点异步订单网关和网络订单广播协议重构。
  - 复杂撮合策略（冰山单、IOC/FOK、做市商策略）。

## 接口 / 数据
### 1) Runtime：模块生命周期与撮合
- `Action` 新增
  - `DelistModuleArtifact { seller_agent_id, wasm_hash }`
  - `DestroyModuleArtifact { owner_agent_id, wasm_hash, reason }`
  - `PlaceModuleArtifactBid { bidder_agent_id, wasm_hash, price_kind, price_amount }`
  - `CancelModuleArtifactBid { bidder_agent_id, wasm_hash, bid_order_id }`
- `DomainEvent` 新增/扩展
  - 新增：`ModuleArtifactDelisted`、`ModuleArtifactDestroyed`、`ModuleArtifactBidPlaced`、`ModuleArtifactBidCancelled`、`ModuleActionFeeCharged`
  - 扩展：`ModuleArtifactListed` / `ModuleArtifactSaleCompleted` 增加订单与成交关联字段（带 `serde(default)` 兼容旧快照）
- `WorldState` 新增
  - `module_artifact_bids: BTreeMap<wasm_hash, Vec<ModuleArtifactBidState>>`
  - `next_module_market_order_id: u64`
  - `next_module_market_sale_id: u64`
- 撮合规则
  - 单工件单卖单 + 多买单，按 `price desc -> order_id asc` 选择最优买单。
  - 价格类型必须一致，买价 `>=` 卖价才可成交。
  - 成交后转移所有权、结算资源、清理 listing/bid。
- 成本模型
  - 通过 `ModuleActionFeeCharged` 统一扣费（从操作者资源扣减，转入世界资源池）。
  - 费用在动作成功路径上生效，失败动作不扣费。

### 2) Simulator：电力单一订单簿
- `Action` 新增
  - `PlacePowerOrder { owner, side, amount, limit_price_per_pu }`
  - `CancelPowerOrder { owner, order_id }`
- `WorldModel` 新增
  - `power_order_book: PowerOrderBookState`（含 `next_order_id` 与 `open_orders`）
- `WorldEventKind` 新增
  - `PowerOrderPlaced { order, fills, auto_cancelled_order_ids }`
  - `PowerOrderCancelled { owner, order_id }`
- 撮合规则
  - 买单：最高限价优先；卖单：最低限价优先；同价按 `order_id` 先后。
  - 仅在 `best_bid >= best_ask` 且动态报价落在双方限价区间时成交。
  - 采用确定性迭代，保证同输入序列跨节点结果一致。

## 里程碑
- M1：文档与任务拆解完成。
- M2：Runtime 生命周期 + 成本模型实现并通过测试。
- M3：模块/电力单簿撮合实现并通过测试。
- M4：回归（`cargo check` + 定向 required tests）与文档/devlog 收口。

## 风险
- 兼容风险：事件字段扩展需 `serde(default)` 保证旧快照可反序列化。
- 一致性风险：撮合顺序必须完全确定，避免 map 遍历顺序导致分叉。
- 经济风险：费用模型引入资源再分配，需覆盖余额不足与拒绝路径测试。
