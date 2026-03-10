# 启动器转账产品级对齐设计（2026-03-06）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.project.md`

## 1. 设计定位
定义转账能力在 native/web 启动器中的产品级对齐方案，补齐字段、状态、错误语义和可观测性，使两端从“功能可用”提升到“产品级一致”。

## 2. 设计结构
- 输入交互层：统一 from/to/amount/nonce 等字段与校验反馈。
- 状态展示层：统一 success/accepted/pending/final/failed 等结果语义。
- 错误对齐层：结构化展示 `error_code/error`，并保留重试路径。
- 可观测层：结合 explorer/transfer tracker 提供转账后的结果追踪入口。

## 3. 关键接口 / 入口
- 转账窗口 UI
- `action_id` / transfer lifecycle 状态
- `error_code` / `error`
- 控制面转账与 explorer 查询接口

## 4. 约束与边界
- native/web 不能出现字段、排序或结果语义分叉。
- 控制面仍以 runtime 业务规则为唯一来源。
- 本阶段对齐产品体验，不扩展钱包托管或跨链能力。
- 失败后必须允许继续重试与诊断。

## 5. 设计演进计划
- 先对齐转账表单和错误语义。
- 再补状态时间线与结果追踪。
- 最后通过跨端回归确认产品级 parity 收口。
