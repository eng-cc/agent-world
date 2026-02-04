# Agent World Runtime：模块订阅过滤器（设计分册）

本分册定义模块订阅的过滤规则，用于在路由阶段基于事件/动作内容进行细粒度筛选。

## 目标
- 支持在 `ModuleSubscription.filters` 中声明筛选条件。
- 路由阶段在调用模块前执行过滤，避免无关调用。
- 过滤规则保持简单、确定性、可回放。

## 范围

### In Scope（V1）
- 基于 JSON Pointer 的等值匹配（`path` + `eq`）。
- 事件/动作分别支持独立规则集。
- 多条规则按 AND 语义（全部匹配才通过）。
- 过滤失败时不调用模块（不产生额外事件）。

### Out of Scope（V1 不做）
- 正则/范围/数值比较等复杂匹配。
- 逻辑组合（OR/NOT）。
- 自定义脚本或 host 过滤逻辑。

## 接口 / 数据

### Subscription 过滤结构
`ModuleSubscription.filters` 为可选 JSON 对象，结构如下：

```
{
  "event": [
    { "path": "/body/payload/data/agent_id", "eq": "agent-1" }
  ],
  "action": [
    { "path": "/action/data/agent_id", "eq": "agent-1" }
  ]
}
```

- `event`：用于事件路由。
- `action`：用于动作路由。
- `path`：JSON Pointer（RFC 6901）。
- `eq`：与指向值做 JSON 等值比较。

### 匹配对象
- 事件路由：使用 `WorldEvent` 的 JSON 表示。
- 动作路由：使用 `ActionEnvelope` 的 JSON 表示。

### 规则语义
- 若 `filters` 为空或缺失：视为通过。
- 若对应规则集为空：视为通过。
- 规则中 `path` 未命中：视为不通过。
- `filters` 解析失败：视为不通过（避免误放行）。

## 里程碑
- **F1**：实现过滤解析与匹配（事件/动作）。
- **F2**：补充过滤相关测试与文档示例。

## 风险
- 过滤依赖 JSON 结构稳定性，事件/动作结构变化可能导致过滤失效。
- 过滤配置错误会导致模块不被调用，需要良好配置校验与测试覆盖。
