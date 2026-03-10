# Agent World Runtime：Observer 同步源策略可观测性设计

- 对应需求文档: `doc/p2p/observer/observer-sync-mode-observability.prd.md`
- 对应项目管理文档: `doc/p2p/observer/observer-sync-mode-observability.project.md`

## 1. 设计定位
定义 Observer 同步源策略的可观测性设计，统一同步模式的日志、指标、诊断信号与排障入口。

## 2. 设计结构
- 模式暴露层：明确当前同步源模式、切换原因与生效配置。
- 日志指标层：输出同步过程的关键事件与健康指标。
- 诊断排障层：围绕 lag、回退、切换失败暴露诊断信号。
- 运维闭环层：把关键结论沉淀到发布和治理口径。

## 3. 关键接口 / 入口
- 同步模式状态
- observer 日志/metrics
- 诊断事件与失败签名
- 运维排障入口

## 4. 约束与边界
- 模式切换必须可观测、可解释。
- 关键失败需要稳定失败签名。
- 不在本专题扩展新的控制面。

## 5. 设计演进计划
- 先暴露模式状态。
- 再补日志指标与失败签名。
- 最后联动运维口径。
