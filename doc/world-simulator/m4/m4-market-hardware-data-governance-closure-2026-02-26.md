# M4 市场/硬件/数据/治理闭环收口（2026-02-26）

## 目标
- 完成 P0-01~P0-08 的最小可用闭环：
  - 电价模型定义与动态计算落地。
  - 去掉峰谷时段机制，价格完全由供需决定。
  - 补齐测试并纳入 `test_tier_required` / `test_tier_full`。
  - 完成硬件生产/维护/折旧/回收闭环。
  - 完成数据获取/存储/交易/访问控制闭环。
  - 完成合约任务与声誉防通胀、防刷策略。
  - 完成禁区/配额/税费/电费最小治理规则。

## 范围
### In Scope
- `simulator` 电力市场报价逻辑（M4.4）与对应测试。
- `runtime` 经济动作与状态扩展：工厂维护/折旧/回收、数据获取与访问控制。
- `runtime` 合约结算防刷与声誉奖励限流。
- `runtime` 治理策略字段扩展：禁区、配额、税费、电费。
- `testing-manual.md` 增补对应必跑用例入口。

### Out of Scope
- Viewer/UI 展示。
- 新增链上协议。
- 大规模经济平衡调参（仅实现规则闭环和护栏）。

## 接口/数据
### 1) 电价模型（供需）
- 报价口径：
  - `base_price = clamp(market_base_price_per_pu, min, max)`
  - `demand_pressure_bps = clamp((requested * 10000 / max(seller_available_before,1)) - 10000, 0, market_supply_demand_price_max_bps)`
  - `quote = clamp(base_price + base_price * demand_pressure_bps / 10000, min, max)`
- 不再引入峰谷时段因子，不再按时段调整价格。

### 2) 工厂硬件闭环
- 工厂状态新增耐久字段（`durability_ppm`）。
- 每 tick 折旧（基于 `maintenance_per_tick`）。
- 新增维护动作，消耗 `hardware_part` 恢复耐久。
- 新增回收动作，按耐久与回收比例返还硬件材料。

### 3) 数据闭环
- 新增数据采集动作：消耗电力，增加 Agent `Data` 资源。
- 数据交易继续走现有 `TransferResource(kind=Data)` 与经济合约。
- 增加数据访问许可：无许可时拒绝跨 Agent 数据转移与数据合约结算。

### 4) 合约/声誉防通胀防刷
- 成功结算奖励保留“金额相关 + 质押约束 + 全局上限”。
- 增加 pair 级冷却和窗口上限：
  - 冷却内重复结算拒绝。
  - 窗口内奖励累计超过上限后，本窗口奖励衰减到 0。

### 5) 治理最小规则
- `GameplayPolicyState` 增加：
  - `forbidden_location_ids`（禁区）
  - `power_trade_fee_bps`（电费）
- `MoveAgent` 禁止进入禁区。
- 合约结算 `Electricity` 时叠加电费与税费。

## 里程碑
- M0：设计/项目文档建档。
- M1：P0-01 电价模型冻结。
- M2：P0-02/P0-03 代码实现（纯供需定价，无峰谷）。
- M3：P0-04 测试补齐并接入 required/full 口径。
- M4：P0-05 硬件维护/折旧/回收闭环。
- M5：P0-06 数据获取/存储/交易/访问控制闭环。
- M6：P0-07 合约防通胀防刷落地。
- M7：P0-08 治理规则最小版落地与回归收口。

## 风险
- 规则叠加风险：治理/合约/访问控制同时变更，容易引入拒绝路径回归。
  - 缓解：每项配套拒绝路径测试。
- 兼容风险：状态字段扩展影响快照反序列化。
  - 缓解：新增字段全部 `serde(default)`。
- 经济过拟合风险：防刷阈值过严压制正常交易。
  - 缓解：先保守默认值，后续通过回放调整。
