# DistFS 自愈控制面设计

- 对应需求文档: `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.project.md`

## 1. 设计定位
定义 DistFS 自愈的控制面配置与决策模型，为后续 polling loop 和 runtime wiring 提供统一入口。

## 2. 设计结构
- 配置模型层：定义副本维护阈值、执行开关和资源预算。
- 缺失检测层：识别需要修复的副本目标。
- 目标选择层：为修复任务确定优先级和候选来源。
- 治理边界层：控制面只做决策，不直接承载执行逻辑。

## 3. 关键接口 / 入口
- self-healing 配置模型
- 缺失检测输入
- 目标选择输出
- 执行开关

## 4. 约束与边界
- 不重构复制协议。
- 不与 polling/runtime wiring 混淆职责。
- 目标是可配置、可治理，不追求复杂调度最优解。
- 需兼容后续 Node Runtime 接线。

## 5. 设计演进计划
- 先定义配置和阈值。
- 再补缺失检测与目标选择。
- 最后由下游专题接执行闭环。
