# Viewer 全览图缩放分层设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-overview-map-zoom.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-overview-map-zoom.project.md`

## 1. 设计定位
定义 2D 视图从近景细节态到全览图态的自动切换方案：默认让 Agent 更可读，缩到阈值后再进入简化标记层，减少细节噪声。

## 2. 设计结构
- 缩放状态机层：`TwoDZoomTier` 管理 `Detail/Overview` 双层状态与迟滞阈值。
- 默认倍率层：2D 默认 orbit 半径调整为细节态口径。
- 可见性联动层：Overview 时显示 `TwoDMapMarker`，隐藏 `DetailZoomEntity` 细节几何。
- 启动同步层：自动聚焦后跳过首帧默认半径回写，修复放大后立刻缩回问题。

## 3. 关键接口 / 入口
- `TwoDZoomTier`
- `DetailZoomEntity`
- `TwoDMapMarker`
- `sync_camera_mode` / `auto_focus`

## 4. 约束与边界
- 仅改 Viewer 本地缩放和可见性策略，不改 snapshot 协议。
- 3D 模式继续按细节态处理，不参与全览图切换。
- 阈值必须带迟滞，避免用户在边界附近来回抖动。
- 可见性链要依赖完整 `Visibility/InheritedVisibility` 传播。

## 5. 设计演进计划
- 先建立双层缩放状态机。
- 再补默认倍率和可见性联动。
- 最后通过单测收口首帧同步与全览图行为。
