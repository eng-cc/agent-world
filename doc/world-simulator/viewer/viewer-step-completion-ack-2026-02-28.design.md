# Viewer Step Completion Ack 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-step-completion-ack-2026-02-28.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-step-completion-ack-2026-02-28.project.md`

## 1. 设计定位
定义 `step` 控制从“accepted 入队”到“completed 结果”分离的协议方案：服务端按 `request_id` 回传是否真的推进，让 Viewer 和 Web Test API 不再只靠帧差分猜测。

## 2. 设计结构
- 请求扩展层：`PlaybackControl/LiveControl/Control` 都可携带 `request_id`。
- Ack 响应层：新增 `ControlCompletionAck`，返回 `advanced` 或 `timeout_no_progress` 及 delta。
- 服务端接线层：live/offline server 在 `step` 完成窗口结束后统一回 Ack。
- Viewer 消费层：透传 `request_id` 并用 ack 更新 `lastControlFeedback`。

## 3. 关键接口 / 入口
- `request_id`
- `ViewerResponse::ControlCompletionAck`
- `advanced | timeout_no_progress`
- `lastControlFeedback`

## 4. 约束与边界
- 只对携带 `request_id` 的 `step` 输出 completion ack。
- 老版本 server 不回 ack 时，Viewer 仍要保留降级推断路径。
- `timeout_no_progress` 只表示窗口内未观测推进，不等于永久失败。
- 协议改造涉及多 crate，必须靠 targeted 回归保护兼容性。

## 5. 设计演进计划
- 先补 request/response 协议字段。
- 再接 live/offline server 的 ack 输出。
- 最后让 Viewer/Web Test API 消费 ack 并完成 A/B 回归。
