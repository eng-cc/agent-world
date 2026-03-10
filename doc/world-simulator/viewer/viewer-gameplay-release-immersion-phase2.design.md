# Viewer 发行体验沉浸改造 Phase 2 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase2.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase2.project.md`

## 1. 设计定位
定义发行体验改造第二阶段方案：在 Player 模式下落地事件反馈、新手引导和轻量 HUD，形成“反馈 + 引导 + 目标”三件套的低噪声可玩体验。

## 2. 设计结构
- 反馈队列层：从 `ViewerState.events` 增量事件生成 toast/情绪反馈。
- 引导状态层：Player 首屏引导、下一步目标提示与可关闭状态统一管理。
- 轻量 HUD 层：围绕连接状态、tick、事件数和选择状态输出玩家导向摘要。
- 模式兼容层：`ViewerExperienceMode` 决定 Player 启用引导/HUD，Director 保持调试导向。

## 3. 关键接口 / 入口
- `ViewerExperienceMode`
- `egui_right_panel.rs`
- `window.__AW_TEST__` Web 闭环入口
- `egui_right_panel_tests.rs` / `i18n.rs`

## 4. 约束与边界
- 本阶段只增强 Viewer 本地体验层，不改仿真逻辑和大型美术资源。
- 反馈要分层限流，避免沉浸改造反而制造新的视觉噪声。
- 引导层需支持关闭/自动收起，不能长期遮挡主场景。
- 快捷键与聊天输入焦点冲突必须在设计层显式规避。

## 5. 设计演进计划
- 先落事件反馈与反馈队列。
- 再补新手引导和下一步目标提示。
- 最后收敛 Player HUD 风格并完成 Web 闭环验收。
