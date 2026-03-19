# oasis7 Runtime：PoS 固定时间槽（Slot/Epoch）真实时钟驱动设计

- 对应需求文档: `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
- 对应项目管理文档: `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.project.md`

## 1. 设计定位
定义 PoS 固定时间槽真实时钟驱动方案，让 slot/epoch 以稳定 wall clock 推进并驱动节点共识节拍。

## 2. 设计结构
- 时钟锚定层：以真实时间建立 slot/epoch 锚点。
- 调度推进层：按 slot 边界推进提案、见证和提交节拍。
- 状态暴露层：输出当前 slot、epoch 与延迟偏差信号。
- 回归验证层：校验时钟漂移与推进稳定性。

## 3. 关键接口 / 入口
- slot/epoch 时钟源
- PoS 调度入口
- 时间状态观测面
- 时钟回归用例

## 4. 约束与边界
- 时间推进必须单调，不允许倒退。
- 真实时钟偏差需有可观测告警。
- 不在本专题引入新的共识规则。

## 5. 设计演进计划
- 先建立固定 slot 时钟。
- 再接入 PoS 调度。
- 最后补齐观测与回归。
