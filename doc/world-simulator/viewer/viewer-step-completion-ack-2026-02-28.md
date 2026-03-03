# Viewer `step` Completion Ack（2026-02-28）

## 目标
- 为 `step` 控制增加 completion ack：服务端按 `request_id` 回传“已推进/超时无推进”。
- 将 `step accepted`（入队）与 `step completed`（结果）语义分离，降低控制黑盒感。
- 让 Web Test API 可直接消费协议回执，不再仅依赖本地帧窗口推断。

## 范围
- 协议：扩展 `ViewerRequest` 控制请求（携带可选 `request_id`），新增 `ViewerResponse::ControlCompletionAck`。
- live server：在 `step` 执行完成后发出 completion ack（包含推进状态和增量）。
- offline server：在回放 `step` 后同样发出 completion ack，统一语义。
- viewer/web test api：透传 `request_id`，并按 ack 更新 `lastControlFeedback`。

## 接口/数据
- 请求扩展：
  - `ViewerRequest::PlaybackControl { mode, request_id? }`
  - `ViewerRequest::LiveControl { mode, request_id? }`
  - `ViewerRequest::Control { mode, request_id? }`（legacy）
- 新增响应：
  - `ViewerResponse::ControlCompletionAck { ack }`
  - `ack` 字段：
    - `request_id: u64`
    - `status: advanced | timeout_no_progress`
    - `delta_logical_time: u64`
    - `delta_event_seq: u64`
- 语义：
  - `advanced`：`delta_logical_time > 0 || delta_event_seq > 0`
  - `timeout_no_progress`：在本次 `step` 完成窗口内无可见推进
  - 仅当请求携带 `request_id` 且控制为 `step` 时发 completion ack

## 里程碑
- M1 协议落地：request/response 新字段与类型、序列化兼容测试。
- M2 服务端接线：live/offline `step` completion ack 输出。
- M3 Viewer 接线：`request_id` 透传与 web test api ack 消费。
- M4 验证：targeted 测试 + A/B 闭环回归。

## 风险
- 老版本 server 不回 completion ack：需保留 viewer 侧降级推断路径。
- live + consensus 仍可能超出 step 等待窗口：`timeout_no_progress` 代表窗口内未观测推进，不等于永久失败。
- 控制面新增字段后，跨 crate 枚举匹配点较多，需补足回归测试避免协议回退。
