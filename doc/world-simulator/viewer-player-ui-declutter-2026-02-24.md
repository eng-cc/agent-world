# Viewer 玩家模式 UI 去拥挤优化（2026-02-24）

## 目标
- 降低玩家模式下 HUD/引导层在同屏并发时的视觉拥挤与互相覆盖。
- 优先修复已观测到的三类冲突：
  - 顶部反馈 toast 与顶部状态 HUD 重叠。
  - 右下角 Agent chatter 与小地图重叠。
  - 右侧面板展开后仍显示小地图，造成右侧区域密度过高。

## 范围
- 仅调整 `agent_world_viewer` 玩家模式（Player experience）UI 浮层布局策略。
- 不改动世界模拟逻辑、事件语义、控制协议（`__AW_TEST__`）和主流程交互。
- 保持现有玩家引导卡、任务卡、侧边面板功能语义不变。

## 接口/数据
- 涉及模块：
  - `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
  - `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- 布局策略调整：
  - `render_feedback_toasts`：锚点从 `CENTER_TOP` 调整为 `RIGHT_TOP`。
  - `render_player_achievement_popups`：整体下移，避开顶部反馈区域。
  - `render_agent_chatter_bubbles`：增加底部保留高度参数，给小地图留出空间。
  - `render_player_mission_hud`：仅在面板隐藏（world-first）时渲染小地图。
  - `should_show_player_layout_preset_strip`：仅在面板隐藏时显示布局焦点条。
- 新增/调整布局辅助接口：
  - `player_mission_hud_show_minimap(panel_hidden)`
  - `player_mission_hud_minimap_reserved_bottom(panel_hidden)`

## 里程碑
- M0：冻结去拥挤策略（锚点重排 + 条件显示）。
- M1：完成代码改造与单元测试更新。
- M2：完成 Playwright 闭环截图验证并产出总结。

## 风险
- 风险 1：不同窗口尺寸下，右上角堆叠层仍可能在极端分辨率下密度偏高。
  - 缓解：后续可引入基于 `content_rect` 的动态分层间距与断点策略。
- 风险 2：3D WebGL/wgpu 环境在部分驱动上可能有独立稳定性问题，影响 UI 回归截图稳定性。
  - 缓解：闭环验证默认使用 2D 链路；3D 稳定性单独跟踪。
