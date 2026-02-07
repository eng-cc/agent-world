# Agent World：M5 可视化与调试（Bevy）

## 目标
- 提供一个独立的可视化客户端（Bevy），通过网络连接世界数据源。
- 支持最小调试闭环：世界状态面板、事件浏览、回放控制。
- 以“可重放的可视化”为核心：先支持离线回放（snapshot/journal），再扩展在线实时流。

## 范围
- **范围内**
  - 新增 Bevy 可视化 crate：`crates/agent_world_viewer`。
  - 新增数据服务（网络桥接）：由 `agent_world` 提供一个 viewer server（可为 binary）。
  - 定义最小网络协议（JSON 行协议），支持：握手、快照、事件流、回放控制。
  - 事件浏览器支持按类型筛选（subscribe 时指定 event_kinds）。
  - 支持 headless 模式（`AGENT_WORLD_VIEWER_HEADLESS=1`），默认离线以适配无网络权限环境，可用 `AGENT_WORLD_VIEWER_FORCE_ONLINE=1` 强制联网。
  - 显式离线模式（`AGENT_WORLD_VIEWER_OFFLINE=1`）用于无网络权限环境的功能验证。
  - UI：世界状态面板（地点/Agent/资源摘要）、事件浏览器（列表/筛选）、回放控制（暂停/单步/跳转）。
- **范围外**
  - 复杂 3D 渲染、地形/模型资产、声音系统。
  - 完整的在线多客户端协作与权限体系。
  - 高性能海量事件可视化（先做最小可用）。

## 接口 / 数据

### 目录结构（拟）
- `crates/agent_world_viewer/`：Bevy UI 客户端
- `crates/agent_world/src/viewer/`：网络协议 + 数据服务（server）
- `crates/agent_world/src/bin/world_viewer_server.rs`：启动数据服务（可选）

### 协议形态
- **传输层**：TCP（localhost 默认），单连接、JSON 行（NDJSON）
- **约定**：客户端发 `ViewerRequest`，服务端回 `ViewerResponse` 或 `ViewerEvent`

### 消息类型（草案）
```json
// 客户端 -> 服务端
{ "type": "hello", "client": "viewer", "version": 1 }
{ "type": "subscribe", "streams": ["snapshot", "events", "metrics"], "event_kinds": ["agent_moved", "power"] }
{ "type": "request_snapshot" }
{ "type": "control", "mode": "pause" }
{ "type": "control", "mode": "step", "count": 1 }
{ "type": "control", "mode": "seek", "tick": 120 }

// 服务端 -> 客户端
{ "type": "hello_ack", "server": "agent_world", "version": 1, "world_id": "w-1" }
{ "type": "snapshot", "tick": 120, "world": { /* WorldSnapshot */ } }
{ "type": "event", "tick": 121, "event": { /* WorldEvent */ } }
{ "type": "metrics", "tick": 121, "metrics": { /* RunnerMetrics */ } }
{ "type": "error", "message": "..." }
{ "type": "decision_trace", "trace": { /* AgentDecisionTrace */ } }
```

### 数据结构对齐
- `WorldSnapshot`：复用现有 `simulator` 的快照结构（JSON 可序列化）。
- `WorldEvent`：复用现有事件结构（含 tick/时间/类型）。
- `RunnerMetrics`：复用可观测性指标结构。

### 回放策略
- **离线回放（M5 最小目标）**：服务端读取 `snapshot.json` + `journal.json`，按 tick 流式发送事件。
- **在线模式（后续）**：服务端从 `WorldKernel` 事件队列中实时推送。

### 快速运行（离线回放）
1) 生成 demo 数据：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_demo -- twin_region_bootstrap --out .data/world_viewer_data`
2) 启动 viewer server：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_server -- .data/world_viewer_data 127.0.0.1:5010`
3) 启动 UI：  
`env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5010`

### 在线模式（最小实现）
- 在线模式直接从 `WorldKernel` 推送事件。
- 支持两种决策驱动：
  - 默认 `script`（内置 demo script）
  - 可选 `llm`（`world_viewer_live --llm`，通过 `LlmAgentBehavior` 决策）
- 默认单连接、tick 驱动，不保证多客户端一致性。
- `Seek` 仅支持回到 tick=0（重置世界），其他 tick 会返回错误。

#### 快速运行（在线模式）
1) 启动 live server（脚本驱动，默认）：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- twin_region_bootstrap --bind 127.0.0.1:5010`
2) 启动 live server（LLM 驱动）：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --llm --bind 127.0.0.1:5010 --tick-ms 300`
3) 启动 UI：  
`env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5010`

### 3D 交互说明（2026-02-07 更新）
- 3D 视口支持鼠标拖拽轨道相机：
  - 左键拖拽：旋转视角（orbit）
  - 右键/中键拖拽：平移焦点（pan）
  - `Shift + 左键拖拽`：平移焦点（便于触控板）
  - 滚轮：缩放距离（zoom）
- 交互只在左侧 3D 视口生效，右侧 UI 面板不会触发相机移动。
- 输入增量使用光标位置差值（cursor delta），避免部分平台 `MouseMotion` 不稳定导致的拖拽失效。


### 选中对象详情面板（2026-02-07 更新）
- 右侧 UI 新增“Details”区块：点击 3D 视图中的 Agent/Location 后展示详情。
- Agent 详情包含：位置/坐标、机体参数、电力与热状态、资源、最近事件。
- LLM 模式下，Agent 详情额外展示最近 LLM 输入输出：
  - `llm_input`（system+user prompt）
  - `llm_output`（completion 文本）
  - `llm_error / parse_error`（若存在）
- Location 详情包含：名称/坐标、profile、资源、碎片物理与预算摘要、最近相关事件。
- 离线回放与 script 模式无 LLM trace 时，面板显示降级提示（`no llm trace yet`）。

### 测试策略
- UI 自动化测试使用 Bevy 自带 App/ECS（无需额外依赖），以系统级断言 UI 文本/状态更新为主。
- **优先使用 headless 模式验证功能**：在无显示环境下以 `MinimalPlugins` + 逻辑系统驱动 UI 状态变更。
- **每个功能必须有 UI 测试**：世界面板、事件浏览、回放控制、指标展示、订阅筛选等都要有对应断言。
- **前后端联合测试**：使用 headless 协议连通测试，验证 live server 与 viewer 客户端消息往返（独立 integration test，通过 feature 开关）。
- **离线回放联合测试**：基于 `snapshot.json` + `journal.json` 启动 server，验证离线回放链路。
- 重点覆盖：状态栏文本、事件列表刷新、回放控制触发后的 UI 变化。

### 联合测试运行
- 统一测试清单脚本：`./scripts/ci-tests.sh`（包含 viewer 在线/离线联测）。

### Headless UI 测试方法
- **目标**：无需窗口/渲染即可验证 UI 行为与状态更新。
- **方式**：在测试中构建 `App::new()`，添加目标系统（如 `update_ui`），注入必要资源与组件，然后调用 `app.update()` 断言 UI 状态。
- **示例步骤**：
  1) 生成 `App` 并添加系统：`app.add_systems(Update, update_ui)`。
  2) 预置 UI 实体：`Text + 标记组件（StatusText/SummaryText/EventsText）`。
  3) 注入资源：`ViewerState`（包含 snapshot/events/metrics）。
  4) 执行 `app.update()`，用 Query 断言 `Text` 内容变化。
- **要求**：新增 UI 功能必须同步新增 headless UI 测试，覆盖输入/状态变化与输出文本/结构。
- **离线模式**：headless 默认离线；如需联网，设置 `AGENT_WORLD_VIEWER_FORCE_ONLINE=1`；也可显式设置 `AGENT_WORLD_VIEWER_OFFLINE=1` 强制离线。

### 可观测性增强（2026-02-07）
- **背景问题**：当前 3D 视图缺少空间边界/背景参照，事件列表也难以直接回答“每个 Agent 正在做什么”。
- **增强目标**：
  - 在右侧 UI 新增「Agent 活动面板」，按 Agent 展示位置、电力与最近一次动作/功耗活动。
  - 在 3D 场景新增世界背景参照（空间边界盒 + 地板网格线），提升运动与距离感知。
- **接口/数据**：
  - `Agent 活动面板` 从 `WorldSnapshot + WorldEvent` 派生：
    - `snapshot.model.agents[*].location_id`
    - `snapshot.model.agents[*].resources[electricity]`
    - `events` 逆序扫描最近的 Agent 相关事件（move/harvest/transfer/power/refine 等）
  - `背景参照` 从 `snapshot.config.space` 派生：
    - `width_cm/depth_cm/height_cm` 映射到 3D 单位后生成边界盒和网格。
- **验收标准**：
  - 打开 viewer 后可直接看到每个 Agent 的当前状态与最近活动。
  - 场景中可见边界和地板参照，不再是“黑底悬浮点”。

## 里程碑
- **M5.1** 协议与数据服务雏形：定义消息结构与最小 server（能返回快照/事件）
- **M5.2** Bevy UI 骨架：连接、状态面板、事件列表
- **M5.3** 回放控制：暂停/单步/跳转 tick
- **M5.4** 指标与筛选：基础 metrics 展示、事件筛选

## 风险
- **依赖体积与构建时间**：Bevy 依赖较重，离线环境可能无法拉取依赖；需准备锁定版本与镜像策略。
- **回放一致性**：事件顺序与 tick 对齐不一致会导致 UI 误导，需要严格定义回放语义。
- **协议演进**：JSON 协议易变，需版本字段与兼容策略。
- **性能**：大量事件会导致 UI 卡顿，需分页/采样/聚合策略。
- **Camera order 冲突风险**：多相机（3D 场景 + UI）若使用相同优先级会导致渲染歧义与交互异常；通过显式 `Camera.order` 分层（3D=0, UI=1）规避。
