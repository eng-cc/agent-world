# Agent World Viewer：面向开放世界沙盒的可玩化准备（阶段计划）

## 目标
- 在不引入“玩家直接动作输入”（移动/建造/采集等）的前提下，把现有 viewer 从“调试器优先”提升到“可运营的沙盒观察与干预终端”。
- 本轮仅覆盖以下缺口：
  - 1）观测强、玩法表达弱（调试视角过重）
  - 2）玩家交互通道（限定为 **仅修改 Agent prompt**）
  - 4）3D 表达偏符号化（可读性与一致性不足）
  - 5）规模化运行能力（事件量/对象量增长下的稳定性）

## 范围

### In Scope
- 建立“观察态/运营态”双视图信息架构（不改变模拟内核语义）。
- 增加 Prompt 控制面：对 Agent 的 system/short-term/long-term prompt 做在线改写、版本管理、回滚与审计。
- 完成 3D 视觉与性能专项剩余工作（对齐 `viewer-3d-polish-performance` 的 VPP5~VPP9）。
- 引入规模化运行的预算与退化策略（事件窗口、标签/覆盖层 LOD、帧预算监控与自动降级）。

### Out of Scope（本轮明确不做）
- 3）完整玩家“动作玩法闭环”（发现-采集-加工-建造）先列为 TODO。
- 6）产品化层（音频、资产管线、新手引导、商业化包装）先列为 TODO。

## 关键约束（补充口径）
- 玩家交互仅允许“修改 Agent prompt”：
  - 不新增玩家直控动作协议；
  - 世界行为仍由 Agent 决策并通过既有规则执行；
  - viewer 角色定位为“运营控制台（Prompt Ops）”，不是“玩家化身控制器”。

## 接口 / 数据

### 1) Prompt 控制协议（viewer <-> live server）
- `ViewerRequest` 扩展：
  - `prompt_control.apply`：对指定 `agent_id` 的 prompt 字段做 replace/patch。
  - `prompt_control.preview`：仅返回校验和差异，不落库。
  - `prompt_control.rollback`：回退到指定版本。
- `ViewerResponse` 扩展：
  - `prompt_control_ack`：返回 `version/tick/applied_fields/digest`。
  - `prompt_control_error`：返回失败原因（agent 不存在、字段非法、版本冲突等）。

### 2) 运行时状态与审计
- 新增 `AgentPromptProfile`（内存态 + 持久化）：
  - `agent_id`
  - `system_prompt_override`
  - `short_term_goal_override`
  - `long_term_goal_override`
  - `version`
  - `updated_at_tick`
  - `updated_by`
- 事件流新增 `AgentPromptUpdated`：
  - 用于 timeline、回放一致性与诊断面板追踪。

### 3) Viewer 侧 UI 结构（运营态）
- 顶部模式切换：`Observe` / `Prompt Ops`。
- Prompt Ops 面板：
  - Agent 列表（当前版本、最近更新时间、风险标记）。
  - Prompt 编辑区（字段级差异高亮）。
  - 预览/提交/回滚按钮。
  - 变更审计列表（按 tick 过滤并联动时间轴）。

### 4) 可读性与规模化指标
- 新增/落地指标：
  - `render_frame_ms_avg/p95`
  - `visible_labels`
  - `overlay_entities`
  - `event_window_size`
  - `prompt_updates_per_100_ticks`
- 右侧总览加入“预算命中状态”与“自动降级状态”。

## 里程碑

### OWR-1（信息架构）
- 交付：
  - 观察态/运营态双模式框架；
  - 运营态总览与 Agent prompt 清单。
- 验收：
  - 现有观测功能不退化；
  - 新模式切换具备回归测试。

### OWR-2（Prompt 控制面，拆分 Server/Client 两任务）
- OWR2-Server 交付：
  - 协议扩展、服务端应用与版本管理；
  - prompt 变更事件与回放支持。
- OWR2-Client 交付：
  - viewer 编辑、差异预览、回滚与错误反馈 UI；
  - 与 server 端联测闭环。
- 验收：
  - `preview/apply/rollback` 全链路联测通过；
  - prompt 改写后，后续 `decision_trace` 可观测到新版本生效。

### OWR-3（3D 可读性与性能）
- 交付：
  - 完成 VPP5~VPP9（选中强化、标签策略、覆盖层优化、性能摘要、回归验证）。
- 验收：
  - 默认档在 `llm_bootstrap` 下帧时间波动收敛；
  - 2D/3D、时间轴联动、右侧面板能力无回归。

### OWR-4（规模化稳定性）
- 交付：
  - 事件窗口与增量刷新策略；
  - 标签/覆盖层 LOD 与自动降级开关；
  - 压测脚本与基线报告模板。
- 验收：
  - 指定压力场景（对象数/事件速率）下无卡死、无失联；
  - 指标可用于跨版本对比。

## 风险
- Prompt 在线改写导致行为发散：
  - 缓解：字段白名单、版本化回滚、风险提示。
- 渲染优化影响选中/联动语义：
  - 缓解：先保留兼容路径，再分步切换。
- 规模化场景下事件洪峰拖垮 UI：
  - 缓解：窗口化、采样、批处理与自动降级。
- 回放一致性被 prompt 变更破坏：
  - 缓解：把 prompt 变更写入事件并纳入 replay。

## TODO（本轮不展开）
- TODO-3：玩家动作玩法闭环（非 prompt 交互）(不做了，只有AI驱动，没有玩家手动操作)。
- TODO-6：产品化能力（音频/资产管线/新手引导等）。

## 实施进展（2026-02-11）
- 已完成 OWR1：
  - viewer 顶部新增 `模式` 切换（`Observe` / `Prompt Ops`）。
  - `Prompt Ops` 已提供 Prompt-only 约束提示、Agent 目标选择、Prompt 草稿编辑区与审计占位区。
  - 支持环境变量 `AGENT_WORLD_VIEWER_PANEL_MODE=prompt_ops` 直接进入运营态。
- 已完成 OWR2-Server：
  - `ViewerRequest/ViewerResponse` 扩展 `prompt_control.preview/apply/rollback`、`prompt_control_ack/error`，并补充协议回环测试。
  - simulator 新增 `AgentPromptProfile` 与 `AgentPromptUpdated` 事件；kernel/replay 支持 prompt 更新事件持久化与回放一致性。
  - live server 接入 PromptControl 处理（版本校验、摘要、冲突错误、回滚），并将生效配置下推到 LLM 行为运行时。
  - `viewer/live.rs` 测试拆分到 `viewer/live/tests.rs`，保持单 Rust 文件行数约束（<1200）。
- 已完成 OWR3（VPP5~VPP9）：
  - 选中反馈从“仅缩放”升级为“缩放 + 光晕强调”。
  - 标签 LOD 已支持距离衰减、同屏上限、遮挡降权与选中优先。
  - 覆盖层与网格线已接入节流与 LOD（覆盖层按 tick/事件增量刷新、flow 合批、远距 chunk 线隐藏）。
  - 右侧总览新增渲染性能摘要（avg/p95、对象/标签/覆盖层计数、预算状态）。
  - 修复 3D 缩放输入兼容性：滚轮 Pixel 单位归一化，并接入 macOS `PinchGesture` 缩放链路。
  - 修复 2D 缩放链路：滚轮/Pinch 在 TwoD 模式下同步更新正交投影 `scale`，恢复俯视缩放可用性。
- 已完成验证：
  - `env -u RUSTC_WRAPPER cargo check -p agent_world` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world prompt_control_ -- --nocapture` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world replay_from_snapshot_applies_agent_prompt_updated_event -- --nocapture` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration -- --nocapture` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（157 tests）。
  - 截图闭环通过：`./scripts/capture-viewer-frame.sh --scenario llm_bootstrap --addr 127.0.0.1:5163 --tick-ms 300 --viewer-wait 10`。

## 实施进展（2026-02-12）
- 已完成 OWR2-Client：
  - viewer Prompt Ops 面板接入真实协议请求：`prompt_control.preview/apply/rollback`（含回滚版本输入）。
  - Prompt 草稿支持“载入当前配置”，并按字段展示草稿与当前生效 profile 的差异预览。
  - viewer 轮询链路新增 Prompt 控制回执状态：接收 `prompt_control_ack/error`，在面板内回显成功/失败信息，不再把业务错误误判为连接断开。
  - Prompt Ops 审计区接入 `AgentPromptUpdated` 事件流（按 Agent 过滤，展示 tick/version/operation/fields/digest）。
  - 为满足单 Rust 文件行数约束，将 Prompt Ops 面板逻辑从 `egui_right_panel.rs` 拆分到 `prompt_ops_panel.rs`。
- 已完成验证：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer prompt_control -- --nocapture` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer prompt_ops -- --nocapture` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（171 tests）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer` 通过。
- 已完成 OWR4.1：
  - 新增事件窗口策略模块 `event_window.rs`，在 `poll_viewer_messages` 入口对事件流执行“滚动窗口 + 采样”压缩：保留近期事件全量细节，对旧事件按步长采样，避免事件洪峰导致 UI 与内存线性增长。
  - `ViewerConfig` 增加事件窗口策略配置，启动时从环境变量读取并归一化：
    - `AGENT_WORLD_VIEWER_EVENT_WINDOW_SIZE`：事件窗口上限；
    - `AGENT_WORLD_VIEWER_EVENT_WINDOW_RECENT`：近期全量保留条数；
    - `AGENT_WORLD_VIEWER_EVENT_SAMPLE_STRIDE`：旧事件采样步长。
  - 保持既有右侧性能摘要指标口径不变，`event_window_size` 继续反映压缩后窗口大小。
- 已完成验证（OWR4.1）：
  - `env -u RUSTC_WRAPPER cargo fmt --all` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（176 tests，含事件窗口采样新增测试）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer` 通过。
- 已完成 OWR4.2：
  - 引入场景增量刷新模块 `scene_dirty_refresh.rs`，将 3D 场景更新拆为“全量重建判定 + 脏区更新”：
    - 仅在首次快照、时间回退（seek 回放）、空间配置变化、fragment 可见性切换时触发 `rebuild_scene_from_snapshot`；
    - 常规 tick 走脏区刷新路径，按对象级别更新 location/agent/chunk，避免每 tick 全量销毁与重建。
  - `apply_events_to_scene` 的事件增量应用改为基于 `last_event_id` 去重，不再受 `event.time <= snapshot_time` 过滤影响，确保“先收 event、后收 snapshot”时也能稳定增量落地。
  - 为增量刷新补充回归测试（全量重建判定、location/agent 脏区判定）。
- 已完成验证（OWR4.2）：
  - `env -u RUSTC_WRAPPER cargo fmt --all` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（179 tests）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer` 通过。
- 已完成 OWR4.3：
  - 新增自动降级模块 `auto_degrade.rs`，基于渲染压力实现三级降级与回升（带滞回）：
    - Level 1：收紧 `label_lod.max_visible_labels`；
    - Level 2：继续收紧标签并关闭 `heat overlay`；
    - Level 3：继续收紧标签并关闭 `flow overlay`。
  - 降级策略在压力缓解后按级别回升，并恢复到进入降级前的基线标签与覆盖层开关状态，避免一次降级后长期锁死。
  - 启动配置新增 `AGENT_WORLD_VIEWER_AUTO_DEGRADE`（默认开启），支持显式关闭自动降级。
  - 渲染主链路接入：`sample_render_perf_summary` 之后运行 `update_auto_degrade_policy`，确保降级决策使用最新采样窗口。
- 已完成验证（OWR4.3）：
  - `env -u RUSTC_WRAPPER cargo fmt --all` 通过。
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（181 tests，含自动降级新增测试）。
  - `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer` 通过。
- 已完成 OWR4.4：
  - 新增高负载压测脚本 `scripts/viewer-owr4-stress.sh`：
    - 默认执行 `triad_region_bootstrap,llm_bootstrap`；
    - `llm_bootstrap` 自动附带 `--llm`；
    - headless-online 方式采集事件窗口计数并导出 `metrics.csv` / `summary.md`。
  - 新增报告模板 `doc/world-simulator/viewer-open-world-sandbox-readiness.stress-report.template.md`，统一记录脚本结果、渲染指标补录、异常与结论。
  - 压测脚本输出目录标准化为 `.tmp/viewer_owr4_stress/<timestamp>/`，便于后续 OWR4.5 跨版本横向对比。
- 已完成验证（OWR4.4）：
  - `bash -n scripts/viewer-owr4-stress.sh` 通过。
  - `./scripts/viewer-owr4-stress.sh --help` 通过。
  - `./scripts/viewer-owr4-stress.sh --duration-secs 5 --tick-ms 300 --scenarios triad_region_bootstrap --no-prewarm --out-dir .tmp/viewer_owr4_stress_smoke` 通过。
- 已完成 OWR4.5：
  - 固定记录三类基线指标：
    - 帧时间：`frame_ms_avg` / `frame_ms_p95`；
    - 事件吞吐：`events/s`；
    - 卡顿指标：`over_budget_pct`（以 `AGENT_WORLD_VIEWER_PERF_BUDGET_MS=33` 为阈值）。
  - 为基线采样补齐自动化可观测链路：
    - viewer 新增 `perf_probe`（`AGENT_WORLD_VIEWER_PERF_PROBE*`）周期输出运行时性能摘要；
    - headless 模式新增自动 `Play`（默认开启）并落 `viewer status/events` 指标日志；
    - UI 模式支持 `AGENT_WORLD_VIEWER_AUTO_PLAY=1`，用于截图闭环场景下的动态负载采样。
  - 当前批次基线结论已收口到项目管理文档 `doc/world-simulator/viewer-open-world-sandbox-readiness.project.md`，后续版本沿用同口径复跑并更新该文档。
- 已完成验证（OWR4.5）：
  - `./scripts/viewer-owr4-stress.sh --duration-secs 12 --tick-ms 200 --out-dir artifacts/owr4_baseline --no-prewarm` 通过。
  - `AGENT_WORLD_VIEWER_AUTO_PLAY=1 AGENT_WORLD_VIEWER_PERF_PROBE=1 AGENT_WORLD_VIEWER_PERF_PROBE_INTERVAL_SECS=1 AGENT_WORLD_VIEWER_PERF_BUDGET_MS=33 ./scripts/capture-viewer-frame.sh --scenario triad_region_bootstrap --addr 127.0.0.1:5640 --tick-ms 200 --viewer-wait 10 --no-prewarm --keep-tmp` 通过。
  - `AGENT_WORLD_VIEWER_AUTO_PLAY=1 AGENT_WORLD_VIEWER_PERF_PROBE=1 AGENT_WORLD_VIEWER_PERF_PROBE_INTERVAL_SECS=1 AGENT_WORLD_VIEWER_PERF_BUDGET_MS=33 ./scripts/capture-viewer-frame.sh --scenario llm_bootstrap --addr 127.0.0.1:5641 --tick-ms 200 --viewer-wait 10 --no-prewarm --keep-tmp` 通过。
