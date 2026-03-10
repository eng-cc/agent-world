# Agent World Simulator：Asteroid Fragment 命名替换（设计文档）设计

- 对应需求文档: `doc/world-simulator/scenario/asteroid-fragment-renaming.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/asteroid-fragment-renaming.project.md`

## 1. 设计定位
定义 Asteroid Fragment 命名替换设计，统一碎片实体、显示名和配置命名口径。

## 2. 设计结构
- 命名模型层：收敛碎片相关 ID、显示名与文档口径。
- 配置映射层：让场景文件与运行时使用统一命名。
- 兼容迁移层：为旧命名提供可审计的兼容或替换路径。
- 回归校验层：验证重命名后配置、渲染与测试名称一致。

## 3. 关键接口 / 入口
- fragment 命名常量/配置
- 场景命名映射
- 兼容别名入口
- 命名回归检查

## 4. 约束与边界
- 新旧命名映射必须明确。
- 重命名不得导致场景配置失效。
- 不在本专题改写碎片生成算法。

## 5. 设计演进计划
- 先统一命名口径。
- 再接配置与兼容映射。
- 最后清理旧名称残留。
