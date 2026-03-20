# Viewer 控制面拆分：回放/Live 分离（2026-02-27）

- 对应设计文档: `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.design.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.project.md`

审计轮次: 6

## 1. Executive Summary
- 将 viewer 协议中的控制语义从“单一 `Control`”拆分为“回放控制 + live 控制”两条独立控制面。
- 从类型层面避免 live 模式误用 `seek`（回放专属），降低回归风险。
- 保持已有链路可渐进迁移：短期兼容 legacy `Control`，避免一次性破坏集成调用。

## 2. User Experience & Functionality
- 协议层：`oasis7_proto::viewer` 新增 `PlaybackControl` 与 `LiveControl`，并在 `ViewerRequest` 中新增对应请求变体。
- 握手层：`HelloAck` 增加服务端控制面 profile（`playback`/`live`）。
- 服务端执行层：
  - `ViewerServer`（回放）仅执行回放控制。
  - `ViewerLiveServer`（live）仅执行 live 控制。
  - 保留 legacy `Control` 解析分支用于兼容，但收敛为桥接逻辑。
- Viewer 侧：根据 `HelloAck` profile 选择发送 `PlaybackControl` 或 `LiveControl`；live 下不发送 `seek`。
- Viewer UI / Web test API：按当前 `control_profile` 动态暴露受支持动作；live 下不再渲染时间轴 seek 提交入口，`__AW_TEST__` 的 `describeControls` / `fillControlExample` / `getState` / `sendControl` 需要显式暴露 `controlProfile` 和 “live 不支持 seek” 的结构化反馈。
- 测试：覆盖协议 round-trip、回放/live 处理路径、viewer 控制发送路径。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新增协议类型：
  - `ViewerControlProfile`（`playback`/`live`）
  - `PlaybackControl`（`play`/`pause`/`step`/`seek`）
  - `LiveControl`（`play`/`pause`/`step`）
- `ViewerRequest` 新增：
  - `PlaybackControl { mode: PlaybackControl }`
  - `LiveControl { mode: LiveControl }`
- `ViewerResponse::HelloAck` 扩展字段：
  - `control_profile: ViewerControlProfile`
- Viewer 调度反馈：
  - `dispatch_viewer_control` 需要区分“已发送 / 当前 profile 不支持该 control / client channel send failed”三类结果，避免将 live seek 误诊断为断链。
- Web test API / timeline：
  - `getState()` 公开 `controlProfile`
  - `describeControls()` / `fillControlExample()` 按 profile 标记 `supported`
  - live 下 `seek` 返回显式原因与恢复提示：改用 `play/pause/step`
  - timeline seek 按钮仅在 playback / legacy fallback 可见

## 5. Risks & Roadmap
- M1：协议结构与 server/live handler 完成拆分并通过编译。
- M2：viewer 发送链路按 profile 路由控制请求，live 下 seek 禁发。
- M3：相关测试通过，项目文档与 devlog 收口。

### Technical Risks
- 风险1：协议字段扩展影响旧客户端。
  - 缓解：保留 legacy `Control` 兼容分支，`HelloAck` 字段使用默认值与 serde 兼容策略。
- 风险2：viewer 在握手前发送控制请求导致 profile 未就绪。
  - 缓解：握手前走 legacy fallback；握手后严格按 profile 路由。
- 风险3：测试面较广导致改动回归面增大。
  - 缓解：优先覆盖协议 round-trip 与关键控制路径（play/pause/step/seek）。
- 风险4：握手前后 profile 切换可能让 UI / test API 快照短暂陈旧。
  - 缓解：展示层按当前快照裁剪动作，发送层继续做一次 dispatch-time profile 校验，保证最终错误语义正确。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`；2026-03-18 补充 T4，收口 live seek 的显式语义反馈与前端暴露边界，映射模块任务 `TASK-WORLD_SIMULATOR-163 (PRD-WORLD_SIMULATOR-017/018)`。
