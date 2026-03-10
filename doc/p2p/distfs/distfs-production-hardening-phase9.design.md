# DistFS 生产化增强 Phase 9 设计

- 对应需求文档: `doc/p2p/distfs/distfs-production-hardening-phase9.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-production-hardening-phase9.project.md`

## 1. 设计定位
定义 DistFS 自适应 challenge 调度的 backoff observability 扩展方案：让生产环境能看见退避跳过轮次、累计退避时长与最近一次退避决策。

## 2. 设计结构
- 观测字段层：在 cursor state 中加入 skipped rounds、applied ms、last reason/multiplier 等字段。
- 行为接线层：调度器与 runtime 在退避发生时同步更新观测字段。
- 兼容恢复层：旧状态文件通过默认值恢复，不阻断升级。
- 报告消费层：现有测试和报告链路稳定消费新增观测数据。

## 3. 关键接口 / 入口
- `StorageChallengeProbeCursorState` 扩展字段
- runtime 持久化链路
- legacy 状态兼容恢复
- 退避观测测试

## 4. 约束与边界
- 不引入新的 challenge 类型和 reward 公式。
- 重点是 observability，不是算法重写。
- 兼容性优先，新增字段不能破坏旧状态恢复。
- 观测更新时机需统一，避免误导运维。

## 5. 设计演进计划
- 先扩状态结构。
- 再接调度行为更新。
- 最后通过兼容与 runtime 测试收口。
