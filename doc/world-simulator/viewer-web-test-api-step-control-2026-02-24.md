# Viewer Web Test API `step` 控制补齐（2026-02-24）

## 目标
- 修复 Web 语义测试 API `window.__AW_TEST__.sendControl("step")` 无效的问题。
- 保障 Playwright 可按“逐步操作（step-by-step）”模拟用户流程，而不是只能依赖 `play/pause/seek`。
- 保持现有 `sendControl` 兼容性：`play`、`pause`、`seek` 行为不变。

## 范围

### In Scope
- 文件：`crates/agent_world_viewer/src/web_test_api.rs`
- 为 `sendControl` 增加 `step` 动作解析：
  - 支持 `sendControl("step")`（默认 `count=1`）。
  - 支持 `sendControl("step", { count: n })` 与数值 payload。
- 执行 Web 闭环回归（Playwright）：验证 `step` 能触发 `tick/eventCount/traceCount` 增长。

### Out of Scope
- 不修改 viewer 协议定义（`ViewerControl::Step` 已存在）。
- 不改动 LLM 决策逻辑与超时策略。
- 不改动 `sendControl("seek")` payload 语义。

## 接口 / 数据
- JS 测试接口：`window.__AW_TEST__.sendControl(action, payload?)`
- 本次新增受支持 action：
  - `"step"`
- `step` payload 解析规则：
  - `undefined/null` -> `count=1`
  - number `>=1` -> `count=number`
  - object `{ count: number>=1 }` -> `count=count`
  - 其他非法值 -> 忽略并告警（保持当前 warning 语义）

## 里程碑
- M0：问题复现与根因定位（`step` 未在 `parse_control_action` 中支持）。
- M1：代码修复（补齐 `step` 解析 + 默认 count 规则）。
- M2：回归验证（`cargo check` + Playwright Web 闭环）。
- M3：文档/devlog 回写收口。

## 风险
- payload 解析放宽后，若外部传入浮点值会按 `as usize` 截断；当前语义保持“>=1 视为有效”，风险可接受。
- `step` 生效后，LLM 模式下步进仍可能受外部模型延迟影响；该延迟不属于本次修复范围。

## 里程碑状态（2026-02-24）
- M0：完成。
- M1：完成。
- M2：完成。
- M3：完成。
