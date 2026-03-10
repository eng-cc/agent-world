# Agent World Simulator：Frag 资源平衡与新手友好生成（设计文档）设计

- 对应需求文档: `doc/world-simulator/scenario/frag-resource-balance-onboarding.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/frag-resource-balance-onboarding.project.md`

## 1. 设计定位
定义 Frag 资源平衡与新手友好生成设计，使碎片场景在早期体验上兼顾资源平衡与引导性。

## 2. 设计结构
- 资源基线层：定义碎片初始资源、稀缺度与分布。
- 新手引导层：确保开局关键资源与行动路径可达。
- 平衡调节层：对过度稀缺或过度富集场景进行修正。
- 验证回归层：通过 onboarding 场景验证首轮体验稳定。

## 3. 关键接口 / 入口
- 碎片资源基线配置
- 新手友好生成规则
- 平衡调节参数
- onboarding 回归场景

## 4. 约束与边界
- 开局资源应支持基本体验闭环。
- 平衡优化不能消除场景差异性。
- 不在本专题扩展完整经济系统重构。

## 5. 设计演进计划
- 先固化资源基线。
- 再补新手友好约束。
- 最后执行可玩性回归。
