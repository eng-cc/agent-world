# Viewer Web Software-Safe Mode 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.project.md`

## 1. 设计定位
为 Web Viewer 增加一个真正不依赖 GPU 硬件能力的安全模式，在 `SwiftShader` / software renderer / 无 WebGL 环境下仍能保持“可连接、可观测、可控制、可采证”的最小玩法闭环。

## 2. 核心设计决策
- **保留**现有 Bevy/WGPU Viewer 作为 `standard` 高保真路径。
- **新增**一个 `software_safe` 轻量 Web frontend，技术上与 WGPU/WebGL 解耦。
- **新增** bootstrap shell 负责模式探测与选路；不要让重型 wasm viewer 先启动再崩溃。

## 3. 设计结构

### 3.1 Bootstrap Shell
职责：
- 解析 `render_mode`（query / CLI / env）
- 探测浏览器环境：WebGL 可用性、renderer 信息、已知 software renderer 标记
- 决定加载：
  - `standard` 资源入口
  - `software_safe` 资源入口
- 在页面级显式展示：当前模式、fallback 原因、如何切回标准模式

### 3.2 Standard Viewer
- 沿用现有 `oasis7_viewer` wasm 路径。
- 继续承担视觉质量验收、3D 视角、既有艺术表现与高保真交互。
- `auto` 模式下只有在环境足够好时才进入该路径。

### 3.3 Software-Safe Frontend
技术方向：
- 优先采用 DOM/SVG/Canvas2D 组合，而不是继续依赖 WGPU/WebGL。
- UI 目标是“世界控制台/语义地图”而不是“美术展示”。

建议模块：
- `status_bar`：连接状态、tick、eventSeq、provider info、render mode
- `semantic_map`：简化 2D 语义图（点/标签/区域）
- `entity_list`：Agent / Location 列表与过滤
- `detail_panel`：选中对象详情
- `control_panel`：play/pause/step
- `event_feed`：最近事件/控制反馈

### 3.4 Shared Data / Control Adapter
- 复用当前 viewer/runtime 协议。
- 如有必要，在前端增加一层 `safe_mode_view_model` 聚合层，把现有状态整理成安全模式 UI 更容易消费的数据结构。
- `__AW_TEST__` 保持统一入口，确保脚本不因模式切换而完全重写。

## 4. 为什么不是“继续在 Bevy/WGPU 里降特效”
- 降 deferred / 后处理可以降低 shader 复杂度，但不能保证避开 WGPU/WebGL 初始化失败。
- `#39` 证明问题发生在更底层的 renderer / pipeline 建立阶段；若底层仍绑在 software WebGL 不稳定路径，就无法满足“无 GPU 硬件依赖”。
- 因此，`software_safe` 必须在技术栈层面与标准 Viewer 解耦，而不是仅靠运行时配置降级。

## 5. 模式切换策略
- `render_mode=standard`：始终尝试标准模式；失败则报错，不自动隐藏。
- `render_mode=software_safe`：始终走安全模式，用于 CI/agent-browser/弱机。
- `render_mode=auto`：
  1. 启动 bootstrap
  2. 探测环境
  3. 能稳定跑标准模式则走标准模式
  4. 否则显式降级到 `software_safe`

## 6. 与现有专题关系
- 与 `viewer-web-runtime-fatal-surfacing-2026-03-12` 的关系：
  - 该专题负责“错误透明度与快失败”
  - 本专题负责“在已知弱图形环境下不直接失败，而是进入安全模式”
- 与 `viewer-webgl-deferred-compat-2026-02-24` 的关系：
  - 该专题负责标准模式下减少部分 WebGL 兼容问题
  - 本专题不再试图把所有弱环境都塞进标准模式，而是引入独立 fallback 前端

## 7. 演进计划
- Phase 1：bootstrap shell + query/env/CLI render_mode 选路
- Phase 2：software-safe MVP（状态、列表、step、反馈）
- Phase 3：selection / semantic map / `oasis7` & testing tooling 对齐
- Phase 4：根据使用数据决定是否补更多 viewer 能力，而不是一开始追求完全视觉等价

## 8. 标准模式 Loading Overlay 生命周期（2026-03-18）
- overlay 继续由静态 `index.html` 提供，但职责仅限于标准模式 wasm 尚未启动前的短暂引导。
- overlay 层必须改为独立覆盖层：
  - 不能再依赖 `body` flex 排版与 canvas 并排；
  - 默认覆盖在页面中心，可淡出，但不可继续吃掉标准 Viewer 的宽度。
- bootstrap shell 负责注册一次性 cleanup：
  - 优先监听 `TrunkApplicationStarted`；
  - 若事件先于 canvas 插入，则用轻量轮询或 `requestAnimationFrame` 等待标准 canvas 出现；
  - cleanup 触发后，将 overlay 标记为 hidden，并在过渡结束后从 DOM 移除。
- software-safe 路径不复用该 cleanup：
  - `render_mode=software_safe` 或 auto fallback 重定向仍由新页面负责自身初始态；
  - 本次只收口标准模式 overlay 残留，不改变 software-safe 的引导页面。
- 回归重点：
  - 标准模式启动后 overlay 会被 cleanup；
  - cleanup 不依赖连接态 `connected`，避免“Viewer 已可交互但 runtime 尚未连上时仍一直显示 loading”；
  - cleanup 后 `body` 不再保留 loading 文案的持续可见节点。
