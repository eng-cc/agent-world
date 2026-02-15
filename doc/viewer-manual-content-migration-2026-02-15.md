# Viewer 使用手册内容搬迁（2026-02-15）设计文档

## 目标
- 将分散在 `doc/world-simulator/viewer-*` 与 `doc/scripts/capture-viewer-frame.md` 的“用户可操作内容”并入 Viewer 使用手册。
- 形成单一入口：`doc/viewer-manual.md`（中文基线）与 `site/doc/cn|en/viewer-manual.html`（站点发布版）。
- 保持现有“Web 默认、native fallback”的闭环策略不变。

## 范围
- 范围内
  - 把以下能力并入手册：
    - 自动步骤（auto select）
    - 右侧面板模块显隐与本地缓存
    - 选中详情面板能力
    - 快速定位 Agent
    - 2D 全览图缩放分层
    - 文本可选中/复制面板
    - UI 语言切换
    - native fallback 高级参数
  - 同步 `doc/viewer-manual.md` 与 `site/doc/cn|en/viewer-manual.html`。
- 范围外
  - 迁移 `.project.md`、`devlog`、runtime 架构设计文档。
  - 改动 viewer 协议或功能实现代码。

## 接口/数据
- 输入文档
  - `doc/viewer-manual.md`
  - `doc/scripts/capture-viewer-frame.md`
  - `doc/world-simulator/viewer-auto-select-capture.md`
  - `doc/world-simulator/viewer-right-panel-module-visibility.md`
  - `doc/world-simulator/viewer-selection-details.md`
  - `doc/world-simulator/viewer-agent-quick-locate.md`
  - `doc/world-simulator/viewer-overview-map-zoom.md`
  - `doc/world-simulator/viewer-copyable-text.md`
  - `doc/world-simulator/viewer-i18n.md`
- 输出文件
  - `doc/viewer-manual.md`
  - `site/doc/cn/viewer-manual.html`
  - `site/doc/en/viewer-manual.html`

## 里程碑
- M1：文档与任务拆解。
- M2：完成中文基线手册合并。
- M3：完成站点 CN/EN 手册同步。
- M4：验证收口（`cargo check`、项目文档、devlog）。

## 风险
- 风险：中英文手册内容漂移。
  - 缓解：同任务内成对更新 `site/doc/cn|en`。
- 风险：搬迁后出现口径冲突（历史文档过时）。
  - 缓解：以当前已上线行为为准，不直接搬旧开关语义。
