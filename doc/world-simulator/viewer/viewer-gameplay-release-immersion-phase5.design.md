# Viewer 发行体验沉浸改造 Phase 5 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.project.md`

## 1. 设计定位
定义第五阶段的发行体验细化方案：继续收敛 Player 默认噪声，强化任务导向与世界反馈之间的过渡，让玩家更稳定地进入“知道目标并愿意继续”的状态。

## 2. 设计结构
- 任务导向层：继续优化目标提示、步骤推进与默认入口可发现性。
- 反馈整合层：把前序阶段的 toast、成就、气泡和 HUD 统一成更稳定的节奏。
- 默认减噪层：压缩非关键调试信息默认可见度，保留按需唤出路径。
- 验证收口层：通过 Web 闭环与截图产物确认发行体验一致性。

## 3. 关键接口 / 入口
- 任务提示/HUD 相关入口
- Player 反馈整合状态
- 面板显隐与默认入口策略
- `testing-manual.md`

## 4. 约束与边界
- 不改变 runtime 规则与协议，只做 Viewer 体验层细化。
- 默认减噪不能导致重要反馈缺失，必须保留最小可行动作线索。
- 阶段细化需与总改造文档保持同口径，避免 phase 文档自成体系。
- 验收仍以真实 Web 闭环为准，而不是静态文案审阅。

## 5. 设计演进计划
- 先对齐第五阶段的任务导向与减噪目标。
- 再补体验节奏整合与入口优化。
- 最后通过回归验收把阶段结果并回总改造路线。
