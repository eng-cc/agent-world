# Agent World Runtime：Observer 同步源策略化（DHT 组合链路，设计文档）设计

- 对应需求文档: `doc/p2p/observer/observer-sync-source-dht-mode.prd.md`
- 对应项目管理文档: `doc/p2p/observer/observer-sync-source-dht-mode.project.md`

## 1. 设计定位
定义 Observer 基于 DHT 组合链路的同步源策略设计，让 observer 在 DHT 能力参与下获得更稳的同步源选择与回退方案。

## 2. 设计结构
- DHT 发现层：通过 DHT 收集可用同步源候选。
- 组合选源层：把 DHT 候选与既有同步源策略组合决策。
- 回退保护层：在 DHT 不稳定或失败时切回安全路径。
- 观测回写层：输出选源与回退证据供运维复核。

## 3. 关键接口 / 入口
- DHT 候选发现入口
- 组合选源策略
- 回退判定条件
- 选源审计记录

## 4. 约束与边界
- DHT 仅作为策略增强，不得破坏主同步语义。
- 回退条件需明确且可观测。
- 不在本专题扩大到完整 DHT 网络治理。

## 5. 设计演进计划
- 先接入 DHT 候选。
- 再实现组合选源。
- 最后固化回退与观测。
