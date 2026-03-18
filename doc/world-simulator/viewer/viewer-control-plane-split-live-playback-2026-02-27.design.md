# Viewer 控制面拆分为回放与 Live 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.project.md`

## 1. 设计定位
定义 viewer 控制语义从单一 `Control` 拆分为 `PlaybackControl` 与 `LiveControl` 的协议与路由方案，以类型系统约束 live 模式禁发 seek，并为渐进迁移保留 legacy 兼容桥接。

## 2. 设计结构
- 协议层：新增 `ViewerControlProfile`、`PlaybackControl`、`LiveControl`。
- 握手层：`HelloAck` 返回服务端控制面 profile。
- 服务端执行层：回放与 live 各自只执行所属控制集合。
- Viewer 路由层：握手后按 profile 发送正确请求，握手前走 legacy fallback。
- Viewer 反馈层：发送链路返回结构化 dispatch result，区分 `Sent`、`UnsupportedForProfile`、`ClientChannelSendFailed`。
- Viewer 表现层：timeline / egui / Web test API 共享同一 profile 判定，live 下裁掉 seek 入口，并把“不支持”反馈成明确语义，而不是“重连后再试”。

## 3. 关键接口 / 入口
- `ViewerControlProfile`
- `PlaybackControl`
- `LiveControl`
- `ViewerRequest::{PlaybackControl, LiveControl}`
- `HelloAck.control_profile`
- `dispatch_viewer_control -> ViewerControlDispatchResult`
- `__AW_TEST__.getState().controlProfile`
- `__AW_TEST__.describeControls()` / `fillControlExample()` / `sendControl()`

## 4. 约束与边界
- live 模式不允许发送 seek。
- 协议扩展需兼容旧客户端和 legacy `Control` 路径。
- 握手前控制请求要有安全 fallback，不可空转。
- 本轮不重写既有控制语义，仅做类型分流与执行隔离。
- 若展示层快照仍未拿到 profile，允许临时按 legacy 能力展示；真正发送时仍必须二次校验 profile，防止 live seek 误报成断链。

## 5. 设计演进计划
- 先扩展协议与握手字段。
- 再拆分 server/live handler 与 viewer 路由。
- 最后通过 round-trip 与关键控制测试完成迁移收口。
- 2026-03-18 增补：在既有拆分方案上收紧 UI / Web test API / automation 的 profile 语义，保证 live seek 的错误签名、可见动作集合和时间轴入口一致。
