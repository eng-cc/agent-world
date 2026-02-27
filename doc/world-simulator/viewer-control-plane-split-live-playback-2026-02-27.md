# Viewer 控制面拆分：回放/Live 分离（2026-02-27）

## 目标
- 将 viewer 协议中的控制语义从“单一 `Control`”拆分为“回放控制 + live 控制”两条独立控制面。
- 从类型层面避免 live 模式误用 `seek`（回放专属），降低回归风险。
- 保持已有链路可渐进迁移：短期兼容 legacy `Control`，避免一次性破坏集成调用。

## 范围
- 协议层：`agent_world_proto::viewer` 新增 `PlaybackControl` 与 `LiveControl`，并在 `ViewerRequest` 中新增对应请求变体。
- 握手层：`HelloAck` 增加服务端控制面 profile（`playback`/`live`）。
- 服务端执行层：
  - `ViewerServer`（回放）仅执行回放控制。
  - `ViewerLiveServer`（live）仅执行 live 控制。
  - 保留 legacy `Control` 解析分支用于兼容，但收敛为桥接逻辑。
- Viewer 侧：根据 `HelloAck` profile 选择发送 `PlaybackControl` 或 `LiveControl`；live 下不发送 `seek`。
- 测试：覆盖协议 round-trip、回放/live 处理路径、viewer 控制发送路径。

## 接口/数据
- 新增协议类型：
  - `ViewerControlProfile`（`playback`/`live`）
  - `PlaybackControl`（`play`/`pause`/`step`/`seek`）
  - `LiveControl`（`play`/`pause`/`step`）
- `ViewerRequest` 新增：
  - `PlaybackControl { mode: PlaybackControl }`
  - `LiveControl { mode: LiveControl }`
- `ViewerResponse::HelloAck` 扩展字段：
  - `control_profile: ViewerControlProfile`

## 里程碑
- M1：协议结构与 server/live handler 完成拆分并通过编译。
- M2：viewer 发送链路按 profile 路由控制请求，live 下 seek 禁发。
- M3：相关测试通过，项目文档与 devlog 收口。

## 风险
- 风险1：协议字段扩展影响旧客户端。
  - 缓解：保留 legacy `Control` 兼容分支，`HelloAck` 字段使用默认值与 serde 兼容策略。
- 风险2：viewer 在握手前发送控制请求导致 profile 未就绪。
  - 缓解：握手前走 legacy fallback；握手后严格按 profile 路由。
- 风险3：测试面较广导致改动回归面增大。
  - 缓解：优先覆盖协议 round-trip 与关键控制路径（play/pause/step/seek）。
