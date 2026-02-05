# Agent World Runtime：模块订阅过滤器（设计分册）

本分册定义模块订阅的过滤规则，用于在路由阶段基于事件/动作内容进行细粒度筛选。

## 目标
- 支持在 `ModuleSubscription.filters` 中声明筛选条件。
- 路由阶段在调用模块前执行过滤，避免无关调用。
- 过滤规则保持简单、确定性、可回放。

## 范围

### In Scope（V2）
- 基于 JSON Pointer 的等值/非等值匹配（`path` + `eq`/`ne`）。
- 数值比较（`gt`/`gte`/`lt`/`lte`）。
- 正则匹配（`re`）。
- 事件/动作分别支持独立规则集（`all`/`any`）。
- 过滤失败时不调用模块（不产生额外事件）。

### Out of Scope（V2 不做）
- 复杂逻辑组合（嵌套 OR/NOT/括号）。
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

### 扩展匹配规则（V2）
支持 OR 逻辑、正则与数值比较，结构示例：

```
{
  "event": {
    "all": [
      { "path": "/body/payload/data/pos/x_cm", "gte": 0 },
      { "path": "/body/payload/data/pos/x_cm", "lt": 10 }
    ],
    "any": [
      { "path": "/body/payload/data/agent_id", "re": "^agent-" }
    ]
  },
  "action": [
    { "path": "/action/data/agent_id", "ne": "agent-legacy" }
  ]
}
```

规则字段：
- `eq` / `ne`：JSON 等值 / 非等值比较。
- `gt` / `gte` / `lt` / `lte`：数值比较（仅在目标为数字时生效）。
- `re`：正则匹配（仅在目标为字符串时生效）。

范围匹配通过两条规则组合实现（例如 `gte` + `lt`）。

### 匹配对象
- 事件路由：使用 `WorldEvent` 的 JSON 表示。
- 动作路由：使用 `ActionEnvelope` 的 JSON 表示。

### 规则语义
- 若 `filters` 为空或缺失：视为通过。
- 若对应规则集为空：视为通过。
- 若规则集为数组：按 AND 语义（全部匹配才通过）。
- 若规则集为对象：`all` 必须全部匹配，`any` 若非空必须至少一条匹配。
- 规则中 `path` 未命中：视为不通过。
- `filters` 解析失败：Shadow/Apply 阶段拒绝该模块变更。
- `path` 必须为空或以 `/` 开头（JSON Pointer 语法）。
- 每条规则必须且只能包含一个操作符字段（`eq`/`ne`/`gt`/`gte`/`lt`/`lte`/`re`）。

## 里程碑
- **F1**：实现过滤解析与匹配（事件/动作）。
- **F2**：补充过滤相关测试与文档示例。
- **F3**：支持 OR/数值比较/正则匹配，并完善校验与测试。

## 风险
- 过滤依赖 JSON 结构稳定性，事件/动作结构变化可能导致过滤失效。
- 过滤配置错误会导致模块不被调用，需要良好配置校验与测试覆盖。
