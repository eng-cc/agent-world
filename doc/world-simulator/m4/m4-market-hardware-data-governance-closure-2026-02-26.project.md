# M4 市场/硬件/数据/治理闭环收口（项目管理文档）

## 任务拆解
- [x] T0 建档（设计文档 + 项目管理文档）
- [x] T1 P0-01：完成 M4.4 电价模型定义（供需平衡）
- [x] T2 P0-02/P0-03：完成动态电价实现并移除峰谷时段机制
- [ ] T3 P0-04：补齐电价/市场机制测试并纳入 required/full
- [ ] T4 P0-05：完成硬件生产/维护/折旧/回收闭环
- [ ] T5 P0-06：完成数据获取/存储/交易/访问控制闭环
- [ ] T6 P0-07：完成合约任务与声誉反通胀/防刷策略
- [ ] T7 P0-08：完成禁区/配额/税费/电费最小治理规则
- [ ] T8 回归与收口（文档、devlog、测试）

## 依赖
- `doc/world-simulator/m4/m4-power-system.md`
- `crates/agent_world/src/simulator/*`
- `crates/agent_world/src/runtime/*`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（T3）
- 已完成：T0, T1, T2
- 进行中：T3
- 阻塞项：无
