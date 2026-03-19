# oasis7 Simulator：LLM 请求层迁移至 async-openai Responses API（设计文档）设计

- 对应需求文档: `doc/world-simulator/llm/llm-async-openai-responses.prd.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-async-openai-responses.project.md`

## 1. 设计定位
定义 LLM 请求层迁移至 async-openai Responses API 的设计，统一异步请求、流式响应与错误处理。

## 2. 设计结构
- 请求适配层：把现有 LLM 请求收敛到 Responses API。
- 流式处理层：支持异步响应、增量结果与中断控制。
- 错误恢复层：统一超时、限流与响应格式异常处理。
- 兼容回归层：保证迁移后调用方语义稳定。

## 3. 关键接口 / 入口
- Responses API 请求入口
- 流式响应处理
- 错误/重试策略
- 兼容回归用例

## 4. 约束与边界
- 迁移不能破坏现有上层调用语义。
- 异步流式状态需可中断、可清理。
- 不在本专题扩展额外模型提供方。

## 5. 设计演进计划
- 先完成 API 适配。
- 再补流式与错误恢复。
- 最后执行兼容回归。
