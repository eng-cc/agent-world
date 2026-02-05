# M4 电力子系统（项目管理文档）

## 背景设定与约束

- 初始状态：硅基个体未工业化，每个 Agent 自带基础发电与储能零件，可维持短期自给自足
- 初始世界不预置大型工业电力设施
- 工业设备需要由 Agent 后续建造，电力子系统需保持设施创建与扩展的开放性
- 初始阶段不存在市场概念；市场可能在文明发展过程中演化出现
- 框架需支持演化扩展点：Agent 自行设计模块可编译为 WASM 并动态调用（系统基础能力）

## 任务拆解

### M4.1 基础电力消耗与状态管理
- [x] 定义 AgentPowerState 枚举（Normal/LowPower/Critical/Shutdown）
- [x] 定义 PowerConfig 配置参数
- [x] 扩展 Agent 结构添加 power 字段（AgentPowerStatus）
- [x] 扩展 WorldConfig 添加 power 配置
- [x] 实现空闲消耗：process_power_tick() 方法
- [x] 实现电力状态检测与自动切换
- [x] 实现 Shutdown Agent 的动作拒绝（AgentShutdown RejectReason）
- [x] 添加 PowerConsumed/PowerStateChanged/PowerCharged 事件类型
- [x] 实现 consume_agent_power/charge_agent_power 方法
- [x] 编写单元测试（空闲消耗、状态切换、休眠恢复、充电）- 5 个新测试

### M4.2 发电与储能设施
- [x] 定义 FacilityId 类型别名
- [x] 定义 PowerPlant 结构（发电设施）
- [x] 定义 PowerStorage 结构（储能设施）
- [x] 定义 PlantStatus 枚举（Running/Offline/Maintenance）
- [x] 扩展 WorldModel 添加 facilities 字段
- [x] 实现 RegisterPowerPlant/RegisterPowerStorage 动作
- [x] 实现每 tick 发电逻辑
- [x] 实现充放电逻辑
- [x] 编写单元测试（发电、储能、设施状态）

### M4.3 电力传输与交易
- [x] 定义 DrawPower/StorePower 动作
- [x] 定义 BuyPower/SellPower 动作
- [x] 实现电力传输损耗计算
- [x] 实现跨 Location 传输限制
- [x] 添加 PowerTransferred 事件类型
- [x] 编写单元测试（传输、交易、损耗）

### M4.4 电价与市场机制（可选）
- [x] 定义电价模型（基于 Location 库存）
- [x] 实现动态电价计算（base/min/max + sensitivity）
- [x] 实现市场价交易动作（Buy/Sell at market）
- [x] 编写测试（市场价/库存变化）
- [ ] 实现峰谷电价时段
- [ ] 预留市场演化扩展点（WASM 规则模块接入）

## 依赖

- simulator 模块中的 ResourceStock, WorldKernel, Agent 等类型
- 已有的 Action/Event 处理框架

## 状态

- 当前阶段：M4.4 电价与市场机制（部分完成）
- 下一阶段：峰谷电价与可插拔市场规则
- 上一阶段：M4.1 基础电力消耗与状态管理（已完成）
- 背景设定：初始自给自足、未工业化，设施由 Agent 后续创造（已同步设计文档）
