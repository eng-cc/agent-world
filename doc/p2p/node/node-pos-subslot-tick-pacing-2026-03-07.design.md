# Agent World Runtime：PoS 槽内 Tick 相位门控与自适应节拍（10 Tick/Slot）设计

- 对应需求文档: `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.project.md`

## 1. 设计定位
定义 PoS 槽内 Tick 相位门控与自适应节拍设计，使 10 Tick/Slot 在真实时钟与运行时负载下仍保持稳定推进。

## 2. 设计结构
- 相位门控层：把 slot 细分为稳定的 subslot/tick 相位。
- 节拍调节层：依据运行时负载调整 tick 间隔但保持总槽长约束。
- 执行协调层：让执行、广播与校验在相位边界有序发生。
- 观测回归层：输出 tick 漂移、欠拍和超拍信号。

## 3. 关键接口 / 入口
- subslot/tick 调度器
- 10 Tick/Slot 相位规则
- 运行时负载反馈
- tick 节拍观测面

## 4. 约束与边界
- 总槽长约束优先于局部 tick 平滑。
- 相位门控必须避免跨相位乱序执行。
- 不在本专题扩大到动态 slot 长度治理。

## 5. 设计演进计划
- 先固化 10 Tick/Slot 相位规则。
- 再接自适应节拍反馈。
- 最后执行时序稳定性回归。
