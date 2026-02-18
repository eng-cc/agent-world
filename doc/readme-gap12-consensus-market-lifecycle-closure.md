# README 缺口 1/2 收口：Live 共识提交主路径 + LLM/Simulator 模块市场生命周期（设计文档）

## 目标
- 收口缺口 1：`world_viewer_live` 主链路从“本地 kernel 直接推进”切换到“先提交 node 共识，再仅回放已提交动作推进状态”。
- 收口缺口 2：补齐 LLM/Simulator 的模块市场/生命周期动作入口，支持上架/购买/下架/销毁/出价/撤单完整可执行闭环。
- 保持现有接口兼容，避免破坏 runtime 既有流程与历史 payload。

## 范围
- In scope
  - `crates/agent_world_node`
    - 暴露已提交共识动作批次消费接口，供 live viewer 按提交结果回放。
  - `crates/agent_world/src/viewer/live.rs`
    - 增加 node 共识提交桥接能力，替换直接 `kernel.submit_action + step` 的推进方式。
    - LLM 驱动改为“决策提交”和“提交后回执驱动记忆更新”。
  - `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
    - 兼容带 envelope 的共识 payload，非 runtime action 不再导致桥接失败。
  - `crates/agent_world/src/simulator`
    - 扩展 `Action`、`WorldEventKind`、`WorldModel`、kernel 执行/replay，支持模块市场动作。
  - `crates/agent_world/src/simulator/llm_agent`
    - 扩展 decision schema/parser/prompt/openai tool schema，开放模块市场动作入口。
  - `test_tier_required` 覆盖新增主路径与拒绝路径。
- Out of scope
  - 共识协议本身（PoS/签名）重构。
  - runtime 模块市场语义重写（本轮聚焦 simulator + viewer live）。
  - 撮合网络与跨节点订单簿优化。

## 接口 / 数据
### 1) Live 共识提交主路径
- 新增共识 payload envelope（版本化）
  - `runtime_action`
  - `simulator_action { action, submitter }`
- `NodeRuntime` 新增已提交动作批次对外消费能力（drain API）。
- live viewer 新行为
  1. 驱动器产出动作（script/llm）
  2. 动作编码为 envelope 并提交到 node
  3. 仅消费 node 已提交动作并回放到 kernel
  4. 由回放结果更新 LLM 行为回执（`on_action_result`）

### 2) Simulator 模块市场动作
- `Action` 新增
  - `ListModuleArtifactForSale`
  - `BuyModuleArtifact`
  - `DelistModuleArtifact`
  - `DestroyModuleArtifact`
  - `PlaceModuleArtifactBid`
  - `CancelModuleArtifactBid`
- `WorldEventKind` 新增
  - `ModuleArtifactListed`
  - `ModuleArtifactDelisted`
  - `ModuleArtifactBidPlaced`
  - `ModuleArtifactBidCancelled`
  - `ModuleArtifactSaleCompleted`
  - `ModuleArtifactDestroyed`
- `WorldModel` 新增
  - `module_artifact_listings`
  - `module_artifact_bids`
  - `next_module_market_order_id`
  - `next_module_market_sale_id`
- 规则约束
  - 仅 artifact owner 可上架/下架/销毁
  - 买方/出价方需具备足额资源
  - 上架/出价可触发撮合成交（价格满足时）
  - 成交后转移 artifact owner，并清理 listing/bids

### 3) LLM 决策入口扩展
- 新决策字符串
  - `list_module_artifact_for_sale`
  - `buy_module_artifact`
  - `delist_module_artifact`
  - `destroy_module_artifact`
  - `place_module_artifact_bid`
  - `cancel_module_artifact_bid`
- 新字段
  - `price_kind`、`price_amount`、`bid_order_id`、`bidder`
- 约束
  - agent 身份字段仅允许 `self|agent:<id>`（模块市场动作）
  - `price_amount > 0`，`bid_order_id > 0`

## 里程碑
- M1：T0 文档冻结（设计 + 项目管理）。
- M2：T1 live 共识提交主路径 + payload 兼容桥接 + required tests。
- M3：T2 simulator/llm 模块市场生命周期入口 + required tests。
- M4：T3 回归检查、文档状态与 devlog 收口。

## 风险
- 行为一致性风险：LLM 从“本地即时回执”切换为“提交后回执”，需保持执行控制与记忆更新稳定。
- 兼容风险：execution bridge 需同时兼容历史 runtime payload 与新 envelope。
- 回放风险：模块市场新增事件必须保持 replay 幂等且可拒绝脏数据。
- 测试风险：live 共识路径涉及线程和异步提交，需避免 flaky（轮询超时/端口冲突）。
