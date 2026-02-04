# M4 电力子系统（项目管理文档）

## 任务拆解

### M4.1 基础电力消耗与状态管理
- [x] 定义 AgentPowerState 枚举（Normal/LowPower/Critical/Shutdown）
- [x] 定义 PowerConfig 配置参数
- [x] 扩展 Agent 结构添加 power 字段（AgentPowerStatus）
- [x] 扩展 WorldConfig 添加 power 配置
- [ ] 实现空闲消耗：每 tick 扣除电力
- [ ] 实现电力状态检测与自动切换
- [ ] 实现 Shutdown Agent 的调度器跳过逻辑
- [ ] 添加 PowerConsumed/PowerStateChanged 事件类型
- [ ] 编写单元测试（空闲消耗、状态切换、休眠恢复）

### M4.2 发电与储能设施
- [ ] 定义 FacilityId 类型别名
- [ ] 定义 PowerPlant 结构（发电设施）
- [ ] 定义 PowerStorage 结构（储能设施）
- [ ] 定义 PlantStatus 枚举（Running/Offline/Maintenance）
- [ ] 扩展 WorldModel 添加 facilities 字段
- [ ] 实现 RegisterPowerPlant/RegisterPowerStorage 动作
- [ ] 实现每 tick 发电逻辑
- [ ] 实现充放电逻辑
- [ ] 编写单元测试（发电、储能、设施状态）

### M4.3 电力传输与交易
- [ ] 定义 DrawPower/StorePower 动作
- [ ] 定义 BuyPower/SellPower 动作
- [ ] 实现电力传输损耗计算
- [ ] 实现跨 Location 传输限制
- [ ] 添加 PowerTransferred 事件类型
- [ ] 编写单元测试（传输、交易、损耗）

### M4.4 电价与市场机制（可选）
- [ ] 定义电价模型（供需平衡）
- [ ] 实现动态电价计算
- [ ] 实现峰谷电价时段
- [ ] 编写测试

## 依赖

- simulator 模块中的 ResourceStock, WorldKernel, Agent 等类型
- 已有的 Action/Event 处理框架

## 状态

- 当前阶段：M4.1 基础电力消耗与状态管理
- 上一阶段：M3 Agent 运行时与 SDK（已完成）
