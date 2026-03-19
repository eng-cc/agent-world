# oasis7 Simulator：默认电力设施语义下沉为场景配置设计

- 对应需求文档: `doc/world-simulator/scenario/scenario-power-facility-baseline.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/scenario-power-facility-baseline.project.md`

## 1. 设计定位
定义默认电力设施语义下沉为场景配置的设计，让是否注入 power facility 由场景显式声明。

## 2. 设计结构
- 设施语义层：将默认电力设施从硬编码迁移到场景语义。
- 场景声明层：允许场景显式开启、关闭或定制设施基线。
- 初始化接线层：在世界初始化时按场景配置注入设施。
- 回归验证层：验证不同场景下设施注入结果一致。

## 3. 关键接口 / 入口
- 场景 power facility 字段
- 世界初始化设施注入
- 设施默认值策略
- 设施回归用例

## 4. 约束与边界
- 设施语义必须由场景显式控制。
- 未声明场景不得隐式注入旧默认设施。
- 不在本专题扩展完整电力经济系统。

## 5. 设计演进计划
- 先下沉设施语义。
- 再补场景声明与初始化接线。
- 最后执行场景回归。
