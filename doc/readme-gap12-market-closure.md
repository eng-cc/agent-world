# README 高优先级缺口收口：模块交易 + 动态电价（设计文档）

## 目标
- 收口 README 高优先级缺口 1：在现有 `DeployModuleArtifact/InstallModuleFromArtifact` 基础上补齐“模块可交易”最小闭环。
- 收口 README 高优先级缺口 2：为 `BuyPower/SellPower` 补齐动态电价机制，让市场价格随供需/距离变化并可审计。
- 保持现有运行路径兼容，不引入破坏性协议重构。

## 范围
- In scope
  - Runtime (`crates/agent_world/src/runtime`)：新增模块上架/购买动作、事件和状态。
  - Simulator (`crates/agent_world/src/simulator`)：新增动态电价参数、报价逻辑、价格带护栏、事件字段增强。
  - 测试：新增/更新 `test_tier_required` 测试覆盖新闭环与拒绝路径。
- Out of scope
  - 世界内 Rust 编译器（仍保持“链下编译后上链/入世界”）。
  - 完整多品类市场（仅本轮电力交易价格机制）。
  - 跨节点订单簿或撮合网络。

## 接口 / 数据
### 1) Runtime：模块交易闭环
- `Action` 新增
  - `ListModuleArtifactForSale { seller_agent_id, wasm_hash, price_kind, price_amount }`
  - `BuyModuleArtifact { buyer_agent_id, wasm_hash }`
- `DomainEvent` 新增
  - `ModuleArtifactListed { seller_agent_id, wasm_hash, price_kind, price_amount }`
  - `ModuleArtifactSaleCompleted { buyer_agent_id, seller_agent_id, wasm_hash, price_kind, price_amount }`
- `WorldState` 新增
  - `module_artifact_owners: BTreeMap<String, String>`（`wasm_hash -> owner_agent_id`）
  - `module_artifact_listings: BTreeMap<String, ModuleArtifactListingState>`
- 约束
  - 仅 owner 可上架。
  - 购买时执行资源结算（`price_kind/price_amount`）并转移 artifact 所有权。
  - `InstallModuleFromArtifact` 在 owner 已登记时要求安装者必须是 owner（未登记 legacy artifact 保持兼容）。

### 2) Simulator：动态电价闭环
- `PowerConfig` 扩展市场参数
  - `dynamic_price_enabled`
  - `market_base_price_per_pu`
  - `market_price_min_per_pu`
  - `market_price_max_per_pu`
  - `market_scarcity_price_max_bps`
  - `market_distance_price_per_km_bps`
  - `market_price_band_bps`
- `PowerEvent::PowerTransferred` 扩展
  - `quoted_price_per_pu`
  - `settlement_amount`
- 行为规则
  - `price_per_pu <= 0` 且动态定价开启时，自动采用 `quoted_price_per_pu`。
  - 显式 `price_per_pu > 0` 时，若偏离 quote 超过 `market_price_band_bps`，动作拒绝。
  - 价格由基础价 + 稀缺加价 + 距离加价计算，并受 min/max 钳制。

## 里程碑
- M1：文档与任务拆解完成。
- M2：Runtime 模块交易动作闭环 + 测试通过。
- M3：Simulator 动态电价闭环 + 测试通过。
- M4：回归、文档状态/devlog 收口。

## 风险
- 兼容风险：新增状态字段需保持快照反序列化默认值兼容。
- 行为风险：动态电价可能改变既有 LLM 行为轨迹，需要价格带护栏避免抖动。
- 经济风险：模块交易结算会引入资源再分配，需覆盖“余额不足/越权上架”等拒绝路径测试。
