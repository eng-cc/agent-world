# oasis7 Runtime：Membership Reconciliation 调度门控与对账计数算术语义硬化（15 点清单第十三阶段）设计

- 对应需求文档: `doc/world-runtime/runtime/runtime-numeric-correctness-phase13.prd.md`
- 对应项目管理文档: `doc/world-runtime/runtime/runtime-numeric-correctness-phase13.project.md`

## 1. 设计定位
定义数值正确性硬化第 13 阶段设计，聚焦 `Agent World Runtime：Membership Reconciliation 调度门控与对账计数算术语义硬化（15 点清单第十三阶段）` 涉及的计数、时间、比率或状态转移边界，消除溢出、截断与顺序漂移风险。

## 2. 设计结构
- 问题收敛层：识别本阶段涉及的算术、计数器、时间源或状态转移边界。
- 语义硬化层：为关键字段建立窄化、饱和、单调或原子更新规则。
- 运行接线层：把数值语义约束落实到对应 runtime/consensus/membership 主路径。
- 回归验证层：通过定向测试与失败签名验证边界条件被正确覆盖。

## 3. 关键接口 / 入口
- 本阶段涉及的 runtime 数值字段
- 边界检查与窄化/饱和入口
- 状态推进与回滚路径
- 数值正确性回归用例

## 4. 约束与边界
- 数值修正必须保持行为可解释、可回放。
- 边界保护优先于局部性能优化。
- 不在本阶段扩大到无关子系统重构。

## 5. 设计演进计划
- 先冻结本阶段问题清单。
- 再逐项落地数值语义硬化。
- 最后固化回归并推进到下一阶段。
