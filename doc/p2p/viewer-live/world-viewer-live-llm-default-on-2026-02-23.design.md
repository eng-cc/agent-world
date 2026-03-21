# oasis7_viewer_live LLM 默认开启（P2P 发行基线）设计文档（2026-02-23）设计

- 对应需求文档: `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.prd.md`
- 对应项目管理文档: `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.project.md`

## 1. 设计定位
定义 `oasis7_viewer_live` 在 P2P 发行基线下默认开启 LLM 的设计，统一默认行为、回退策略与发行口径。

## 2. 设计结构
- 默认开关层：把 LLM 默认开启纳入 viewer live 启动配置。
- 运行回退层：在 LLM 不可用或受限时提供明确降级路径。
- 发行口径层：让 release 默认值与文档、CLI 说明一致。
- 验证收口层：围绕默认开启路径执行定向回归。

## 3. 关键接口 / 入口
- viewer live 启动配置
- LLM 可用性检查
- 降级/回退路径
- 发行回归矩阵

## 4. 约束与边界
- 默认开启不能破坏无 LLM 时的基本可用性。
- 默认值需与发行文档和 CLI 行为一致。
- 不在本专题扩展新的模型编排能力。

## 5. 设计演进计划
- 先固定默认开启策略。
- 再补不可用时降级。
- 最后统一发行口径与回归。
