# Viewer 首局目标清晰度加固（2026-02-27）

## 目标
- 将首局目标提示从“描述性文案”升级为“动作句 + 完成条件 + 预计耗时”，让玩家在 60 秒内明确第一步。
- 将首局信息架构改为“主任务优先、次任务折叠”，降低首次认知负担。
- 增加首局卡住检测与结算回顾，减少“世界在跑但我不知道下一步”的流失点。

## 范围

### In Scope
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
  - 主任务结构化文案（动作/条件/耗时）
  - 次任务折叠呈现
  - 主任务剩余量提示
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
  - 首局进度状态跟踪
  - 5 秒无进展卡住检测
  - 首局完成结算卡片
- `crates/agent_world_viewer/src/egui_right_panel_player_*_tests.rs`
  - 新增/更新首局任务结构、卡住检测、结算触发单测

### Out of Scope
- Runtime/world 规则改动。
- 新增玩法系统（战争/治理/经济）协议。
- 大范围 UI 重排或视觉主题重构。

## 接口/数据
- 首局任务快照扩展（Mission HUD 内部）：
  - `next_action`
  - `completion_condition`
  - `eta`
  - `remaining_hint`
- 首局进度状态（Player Experience Local State）：
  - `session_started_at_secs`
  - `last_progress_tick`
  - `last_progress_event_count`
  - `last_progress_at_secs`
  - `stuck_hint_visible`
  - `summary_visible`
- 触发规则：
  - 卡住：连接成功后 `tick` 与 `event_count` 连续 5 秒无增量。
  - 结算：`guide_progress.explore_ready == true` 后首次展示回顾面板。

## 里程碑
- M1：主任务文案结构化（动作/条件/耗时）+ 次任务折叠。
- M2：下一步按钮语义统一（CTA 明确指向当前步骤）。
- M3：卡住检测与剩余量提示上线。
- M4：首局结算卡片与验收测试补齐。

## 风险
- 首局提示信息过多导致 HUD 再次拥挤。
  - 缓解：次任务默认折叠，仅主任务常显。
- 卡住检测误报（低事件密度场景被误判）。
  - 缓解：同时观察 `tick` 与 `event_count`，仅在连接成功后启用。
- 结算卡片打断沉浸。
  - 缓解：一次性展示，支持立即关闭。

## 验收口径
- Q1（目标理解）主观档位从“有点模糊”提升到“基本清楚/很清楚”为主。
- 首局 60 秒内玩家完成首个有效动作（打开面板/选择目标/触发反馈）的可观测比例提升。
- 卡住场景下 5 秒内出现明确恢复提示，不再静默等待。
