# Agent World Simulator：场景 Asteroid Fragment 配置覆盖（设计文档）设计

- 对应需求文档: `doc/world-simulator/scenario/scenario-asteroid-fragment-overrides.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/scenario-asteroid-fragment-overrides.project.md`

## 1. 设计定位
定义场景级 Asteroid Fragment 配置覆盖设计，让特定场景可按需覆盖碎片世界参数而不破坏主配置。

## 2. 设计结构
- 覆盖模型层：定义场景可覆盖的碎片参数集合。
- 优先级层：明确场景覆盖与全局默认值的生效顺序。
- 初始化接线层：在场景加载阶段把覆盖应用到世界配置。
- 验证回归层：校验不同覆盖组合下的生效结果。

## 3. 关键接口 / 入口
- fragment override 字段
- 配置优先级解析
- 场景加载应用入口
- 覆盖回归用例

## 4. 约束与边界
- 覆盖字段范围必须明确。
- 场景覆盖不得污染全局默认值。
- 不在本专题扩展运行时热更新。

## 5. 设计演进计划
- 先定义 override 字段。
- 再打通加载优先级。
- 最后补齐组合场景回归。
