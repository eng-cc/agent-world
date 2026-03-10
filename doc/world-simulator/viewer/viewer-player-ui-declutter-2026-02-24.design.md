# Viewer Player UI 去拥挤设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-player-ui-declutter-2026-02-24.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-player-ui-declutter-2026-02-24.project.md`

## 1. 设计定位
定义玩家模式 HUD/引导层去拥挤方案：通过重新锚定 toast、Agent chatter 和小地图，减少多浮层并发时的遮挡与密度冲突。

## 2. 设计结构
- 顶部反馈层：把 toast 与顶部状态 HUD 锚点拆开，避免同屏重叠。
- 右下角冲突层：协调 Agent chatter 与小地图位置，降低互相覆盖。
- 侧边联动层：右侧面板展开时收敛小地图显示，控制右侧区域密度。
- 玩家模式限定层：仅作用于 Player experience，不改主流程语义和测试 API。

## 3. 关键接口 / 入口
- `egui_right_panel_player_experience.rs`
- `egui_right_panel_player_guide.rs`
- toast / chatter / minimap 布局锚点
- Player 模式浮层状态

## 4. 约束与边界
- 不改世界模拟逻辑、事件语义和 `__AW_TEST__` 协议。
- 去拥挤只调整布局，不改变任务卡、引导卡和侧边面板功能语义。
- 多浮层优先顺序需稳定，不能因状态抖动来回跳位。
- 布局优化要兼顾中英文长度差异。

## 5. 设计演进计划
- 先对齐三类冲突场景。
- 再调整各浮层锚点和联动显隐。
- 最后通过玩家模式 UI 测试收口去拥挤效果。
