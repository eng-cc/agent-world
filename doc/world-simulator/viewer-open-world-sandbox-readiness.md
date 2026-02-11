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

### OWR-2（Prompt 控制面）
- 交付：
  - 协议扩展、服务端应用与版本管理；
  - prompt 变更事件与回放支持；
  - viewer 编辑与回滚 UI。
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
- TODO-3：玩家动作玩法闭环（非 prompt 交互）。
- TODO-6：产品化能力（音频/资产管线/新手引导等）。

## 实施进展（2026-02-11）
- 已完成 OWR1：
  - viewer 顶部新增 `模式` 切换（`Observe` / `Prompt Ops`）。
  - `Prompt Ops` 已提供 Prompt-only 约束提示、Agent 目标选择、Prompt 草稿编辑区与审计占位区。
  - 支持环境变量 `AGENT_WORLD_VIEWER_PANEL_MODE=prompt_ops` 直接进入运营态。
- 已完成 OWR3（VPP5~VPP9）：
  - 选中反馈从“仅缩放”升级为“缩放 + 光晕强调”。
  - 标签 LOD 已支持距离衰减、同屏上限、遮挡降权与选中优先。
  - 覆盖层与网格线已接入节流与 LOD（覆盖层按 tick/事件增量刷新、flow 合批、远距 chunk 线隐藏）。
  - 右侧总览新增渲染性能摘要（avg/p95、对象/标签/覆盖层计数、预算状态）。
- 已完成验证：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer` 通过（157 tests）。
  - 截图闭环通过：`./scripts/capture-viewer-frame.sh --scenario llm_bootstrap --addr 127.0.0.1:5163 --tick-ms 300 --viewer-wait 10`。
