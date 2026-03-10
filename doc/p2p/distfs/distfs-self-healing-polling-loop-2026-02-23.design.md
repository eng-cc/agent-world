# DistFS 自愈轮询循环设计

- 对应需求文档: `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.project.md`

## 1. 设计定位
定义 DistFS 自愈 polling loop 的节奏、任务生成和失败统计方案，让修复任务按节拍平滑运行。

## 2. 设计结构
- 轮询节奏层：定义每 tick/每轮的扫描与执行上限。
- 任务生成层：从控制面输出生成可执行修复项。
- 非阻塞执行层：单次失败不阻断后续轮次。
- 统计观测层：记录执行、跳过和失败原因。

## 3. 关键接口 / 入口
- polling loop 配置
- 修复任务生成
- 执行上限
- 轮询统计

## 4. 约束与边界
- 不定义 Node Runtime 具体接线。
- 轮询应与主链路资源竞争保持边界。
- 失败要可观测但不能升级为全局阻断。
- 执行依赖控制面配置输入。

## 5. 设计演进计划
- 先固定轮询节奏。
- 再补任务生成与失败统计。
- 最后由 runtime wiring 专题接到活跃节点。
