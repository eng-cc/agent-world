# Agent World：M5 可视化与调试（Bevy）

## 目标
- 提供一个独立的可视化客户端（Bevy），通过网络连接世界数据源。
- 支持最小调试闭环：世界状态面板、事件浏览、回放控制。
- 以“可重放的可视化”为核心：先支持离线回放（snapshot/journal），再扩展在线实时流。

## 设计原则（2026-02-07 更新）
- 希望通过可视化能以最直接的方式获取所有模拟相关的信息。
- 信息直达优先：默认视图应直接回答“谁在什么位置、正在做什么、为何这么做（事件与 LLM 决策）”。
- 先完整呈现，再按需折叠：优先减少在终端/日志/UI 之间来回切换。

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
- `Seek` 支持任意 tick：`seek` 到历史 tick 会触发 reset + replay，到未来 tick 会在当前状态继续推进。

#### 快速运行（在线模式）
1) 启动 live server（脚本驱动，默认）：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- twin_region_bootstrap --bind 127.0.0.1:5010`
2) 启动 live server（LLM 驱动）：  
`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --llm --bind 127.0.0.1:5010 --tick-ms 300`
3) 启动 UI：  
`env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5010`


### 在线模式时间轴 UI（2026-02-07 更新）
- 顶部控制区新增 `Timeline` 区块：显示 `now/target/max` 三个 tick 指标。
- 提供 `-100/-10/-1/+1/+10/+100` 目标 tick 微调按钮，支持快速定位。
- 新增时间轴拖拽条（scrub bar）：按住拖拽可在可见 tick 范围内连续设置目标 tick。
- 点击 `Seek Target` 后发送 `ViewerControl::Seek { tick: target }`，服务端返回目标 `Snapshot + Metrics`。
- 时间轴默认跟随当前 tick；当用户手动调整目标后切换为“手动目标”直到发送 seek。
- 新增关键事件标注摘要：`err/llm/peak` 计数 + 关键 tick 列表（错误、LLM 决策、资源峰值）。
- 新增事件密度 sparkline：按 tick 分桶显示近期事件分布，快速识别事件高密度区间。
- 新增标注联动按钮（`Jump Err/LLM/Peak`）：点击后自动跳到下一目标 tick，并将事件列表切换到该 tick 的上下文窗口。
- 新增标注类别独立开关（`Err/LLM/Peak`）：可分别开启/关闭三类标注，跳转按钮、计数与 tick 列表同步按开启状态过滤。

### 世界覆盖层（2026-02-07 实现）
- 目标：在 3D 视图中直接补齐“chunk 探索态 / 资源热力 / 流动路径”三类全局信息，减少纯文本检索。
- 覆盖项：
  - Chunk 探索态覆盖：按开关控制 chunk 覆盖可见性（基于现有 unexplored/generated/exhausted 颜色）。
  - 资源热力覆盖：按地点资源强度绘制热力柱（优先 electricity，兼顾 hardware/data）。
  - 电力/交易流覆盖：根据近期 `PowerTransferred/ResourceTransferred` 事件绘制方向连线。
- 交互：顶部新增覆盖层开关（Chunk/Heat/Flow），可独立启停，状态摘要实时更新。
- 实现口径：
  - Heat 强度评分 = `electricity + 4*hardware + 2*data`，并映射到分段颜色与柱高。
  - Flow 取最近窗口事件（默认 28 条）中的 `PowerTransferred/ResourceTransferred`，按类型着色并按量级映射线宽。
  - Chunk 覆盖层复用现有 chunk 实体，只做显隐切换，避免重复几何开销。
- 降级策略：无快照或无可用事件时保留基础场景，覆盖层状态文本明确提示无数据。

### 事件-对象双向联动（2026-02-07 更新）
- 事件定位对象：在时间轴区新增 `Locate Focus Event Object` 控件，按当前 focus tick 选择最近事件，并将 3D 选中对象定位到该事件关联实体。
- 对象跳转事件：在选中对象详情区新增 `Jump Selection Events` 控件，按当前选中对象筛选相关事件并将时间轴目标跳转到下一相关 tick。
- 联动口径：
  - `事件 -> 对象`：优先映射 Agent/Location/PowerPlant/PowerStorage/Chunk；资源转移与精炼事件按 owner 映射。
  - `对象 -> 事件`：对 Agent/Location/设施按 ID 精确匹配；Asset 按 owner 关联匹配；Chunk 按坐标匹配。
- 降级策略：当事件无可映射对象或对象无相关事件时，UI 保留当前状态并输出明确提示。

### 在线模式时间轴语义（2026-02-07 更新）
- `Seek {tick}` 会让 live world 真正切到目标 tick，而不是只移动日志游标。
- 当 `target_tick < current_tick`：先重置世界到初始场景，再按当前驱动模式重放到目标 tick。
- 当 `target_tick >= current_tick`：从当前状态继续推进到目标 tick。
- Seek 期间不会回放中间事件流；完成后只返回目标状态的 `Snapshot + Metrics`，便于 UI 直接落在目标时刻。
- 若世界无法继续推进（例如无可执行动作）会返回错误并停在当前可达 tick。
- 在 `llm` 模式下，回放会重新触发 LLM 决策调用，可能带来额外耗时/费用，且结果不保证与先前运行完全一致。

### 3D 交互说明（2026-02-07 更新）
- 3D 视口支持鼠标拖拽轨道相机：
  - 左键拖拽：旋转视角（orbit）
  - 右键/中键拖拽：平移焦点（pan）
  - `Shift + 左键拖拽`：平移焦点（便于触控板）
  - 滚轮：缩放距离（zoom）
- 交互只在左侧 3D 视口生效，右侧 UI 面板不会触发相机移动。
- 输入增量使用光标位置差值（cursor delta），避免部分平台 `MouseMotion` 不稳定导致的拖拽失效。


### 选中对象详情面板（2026-02-07 更新）
- 右侧 UI 新增“Details”区块：点击 3D 视图中的对象后展示详情。
- 已支持对象：Agent、Location、Asset、PowerPlant、PowerStorage、Chunk。
- Agent 详情包含：位置/坐标、机体参数、电力与热状态、资源、最近事件。
- LLM 模式下，Agent 详情额外展示最近 LLM 输入输出与诊断字段：
  - `llm_input`（system+user prompt）
  - `llm_output`（completion 文本）
  - `llm_error / parse_error`（若存在）
  - `model / latency_ms / prompt_tokens / completion_tokens / total_tokens / retry_count`
- Location 详情包含：名称/坐标、profile、资源、碎片物理与预算摘要、最近相关事件。
- Asset 详情包含：种类、数量、归属者、归属者关联事件。
- PowerPlant/PowerStorage 详情包含：设施参数、电力状态与相关电力事件。
- 离线回放与 script 模式无 LLM trace 时，面板显示降级提示（`no llm trace yet`）。

### 选中对象状态与 trace 一键导出（2026-02-07 实现）
- 目标：将当前选中对象的完整状态、关联事件与决策 trace 一次性导出为结构化 JSON，便于离线分析与问题复盘。
- 交互：顶部新增 `Export Selection` 按钮，点击即导出；旁边状态文本显示成功路径或失败原因。
- 导出目录：默认写入 `.tmp/selection-exports/`，可通过 `AGENT_WORLD_VIEWER_EXPORT_DIR` 覆盖。
- 导出内容：
  - 选择元数据：对象类型、ID、名称、导出时间、当前 tick。
  - 对象状态快照：从 `WorldSnapshot` 提取对应对象完整结构（Agent/Location/Asset/PowerPlant/PowerStorage/Chunk）。
  - 关联事件：按选中对象匹配最近事件窗口（默认最近 40 条）。
  - 决策 trace：Agent 导出对应 `decision_traces`；非 Agent 保留空列表。
  - 详情文案：同步保留 UI `Details` 面板文本，便于人工快速阅读。
- 降级策略：未选中对象、无快照、对象不存在、写文件失败时，状态文本直接返回可读错误，不中断 viewer 主循环。

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

### 现状缺口（信息直达视角，2026-02-07）
- 对象覆盖：选中详情已覆盖 Agent/Location/Asset/PowerPlant/PowerStorage/Chunk。
- 时间轴仍有增强空间：当前已支持标注跳转、类别开关与联动定位，但仍缺更细粒度的多级刻度与跨窗口聚合浏览。
- 联动检索仍可增强：当前已支持“focus event -> 定位对象”与“selection -> 跳转事件”，后续可补“事件列表逐条点击定位对象”。
- LLM 诊断维度仍可增强：已展示 `model/latency/token/retry`，后续可补链路级成本估算与请求 ID 追踪。
- 世界层表达仍可增强：已补齐 chunk/热力/流动覆盖层，后续可进一步增加分层图例与时间衰减轨迹。
- 信息导出能力已补齐：支持一键导出当前选中对象状态、关联事件与 LLM trace；后续可补“复制到剪贴板”和批量导出。

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
