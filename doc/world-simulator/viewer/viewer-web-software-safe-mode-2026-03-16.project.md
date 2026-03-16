# viewer-web-software-safe-mode-2026-03-16 项目管理

- 对应设计文档: `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 完成“Viewer Web Software-Safe Mode”PRD / Design / Project 建模，并回写模块主文档、索引与 devlog。
- [ ] T1 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 为 `world_game_launcher` / Web 静态入口增加 bootstrap shell 与 `render_mode=standard|auto|software_safe` 选路契约。
- [ ] T2 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 落地 `software_safe` MVP 前端，覆盖连接状态、目标列表、对象详情、`play/pause/step` 与最近事件/反馈。
- [ ] T3 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 为 `__AW_TEST__` / 自动化脚本补齐 `renderMode`、`rendererClass`、`softwareSafeReason` 等模式可观测字段。
- [ ] T4 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 打通 `oasis7`、`run-game-test-ab.sh`、`testing-manual.md`、`viewer-manual.md` 的 software-safe 执行口径。
- [ ] T5 (PRD-WORLD_SIMULATOR-039) [test_tier_required]: 在 software renderer / SwiftShader 环境复验“加载 -> 选择目标 -> step -> 新反馈”最小闭环，并据此判断 `#39` 是否收口。

## 依赖
- `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.prd.md`
- `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.design.md`
- `doc/world-simulator/viewer/viewer-web-runtime-fatal-surfacing-2026-03-12.prd.md`
- `testing-manual.md`
- `oasis7` / `run-game-test-ab` 现有脚本与 Web 闭环证据路径

## 状态
- 当前阶段：仅 T0 完成；T1~T5 待研发实现。
- 最近更新：2026-03-16。
- 阻塞项：需要先决定 `software_safe` frontend 的技术形态（DOM/SVG/Canvas2D）与其与现有 Viewer 协议的最小共享层边界。

## 备注
- 本专题的目标不是让 software-safe 与标准模式“视觉等价”，而是让弱图形环境下仍然能完成真实玩家/QA/Agent 的最小闭环。
- `standard` 仍然是视觉与交互质量签收口径；`software_safe` 是玩法闭环与环境兼容兜底口径。
