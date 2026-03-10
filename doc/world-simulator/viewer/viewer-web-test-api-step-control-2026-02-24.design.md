# Viewer Web Test API Step 控制设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-test-api-step-control-2026-02-24.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-test-api-step-control-2026-02-24.project.md`

## 1. 设计定位
定义 `window.__AW_TEST__.sendControl("step")` 的补齐方案，让 Playwright 能真正按 step-by-step 方式驱动 Viewer。

## 2. 设计结构
- action 解析层：在 `parse_control_action` 中补齐 `step`。
- payload 规则层：支持默认 1 步、数值 payload 和 `{count}` 对象。
- 兼容告警层：非法值只告警，不破坏现有 `play/pause/seek`。
- 闭环验证层：用 Web Playwright 验证 `tick/eventCount/traceCount` 能增长。

## 3. 关键接口 / 入口
- `window.__AW_TEST__.sendControl(action, payload?)`
- `step`
- `count` 解析规则
- Playwright Web 闭环

## 4. 约束与边界
- 不修改 `ViewerControl::Step` 协议定义。
- 不改 LLM 决策逻辑与超时策略。
- 历史 `seek` 语义保持不变。
- payload 放宽后仍要维持稳定警告语义。

## 5. 设计演进计划
- 先定位 `step` 未接入的解析缺口。
- 再补 count 规则和兼容告警。
- 最后通过 cargo check + Playwright 闭环收口。
