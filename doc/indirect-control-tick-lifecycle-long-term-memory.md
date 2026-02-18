# 间接控制链路 + WASM Tick 生命周期 + 长期记忆持久化（设计文档）

## 目标
- 收口 README 对“玩家间接控制”的约束：玩家不能直接提交世界动作操控 Agent，只能通过间接链路（提示词/对话）影响 Agent 决策。
- 将 Runtime 模块调用从“仅动作/事件触发”扩展为“支持每 tick 生命周期回调”，并由 WASM 模块自行声明下一次唤醒时机，避免无差别全量调用。
- 将 LLM Agent 的长期记忆从进程内临时态升级为世界快照可持久化状态，支持重启恢复。

## 范围
- In scope
  - Simulator
    - `ActionEnvelope` 增加提交者身份（system/agent/player）并在 `WorldKernel::step` 前做授权校验。
    - `AgentRunner` 改为以 agent 身份提交动作。
    - Viewer live 的 `prompt_control` / `agent_chat` 链路增加 player-agent 绑定与鉴权。
  - Runtime
    - `ModuleSubscriptionStage` 新增 tick 生命周期阶段。
    - 新增模块 tick 调度状态与模块输出唤醒指令（wake/suspend）。
    - `step_with_modules` 接入 tick 生命周期调度。
  - LLM memory
    - 世界模型新增持久化长期记忆字段。
    - LLM behavior 增加长期记忆导出/恢复接口。
    - Viewer live 在 driver 初始化与每 tick 后完成记忆读写同步。
  - 测试
    - 覆盖间接控制拒绝路径、tick 调度唤醒/挂起语义、长期记忆持久化 roundtrip。
- Out of scope
  - 完整 RBAC/多租户权限系统。
  - 跨节点分布式玩家身份认证协议重构。
  - 长期记忆向量检索或外部数据库接入。

## 接口 / 数据
### 1) Simulator 间接控制链路
- `ActionEnvelope` 新增 `submitter` 字段（默认 system，兼容旧快照）。
- `WorldKernel` 新增按 submitter 的动作授权：
  - `player` 直接 world action 拒绝；
  - `agent` 仅允许提交与自身一致的 agent/owner 相关动作；
  - `system` 保持现状。
- Viewer live：
  - `prompt_control` / `agent_chat` 请求接入 `player_id` 校验。
  - 维护 `agent -> player` 绑定（首次绑定、后续一致性校验）。

### 2) Runtime 每 tick 生命周期调度
- ABI：
  - `ModuleSubscriptionStage` 增加 `Tick`。
  - `ModuleOutput` 增加可选 `tick_lifecycle` 指令（`wake_after_ticks` 或 `suspend`）。
- Runtime：
  - `World` 维护模块 tick 调度表（模块下一次唤醒 tick）。
  - 激活 tick 订阅模块时进行初次调度。
  - 每个 `step_with_modules` 执行 tick 调度：
    - 仅调度到期模块；
    - 调用前先移除当前计划，是否继续调度由模块输出指令决定。

### 3) 长期记忆持久化
- `WorldModel` 新增 `agent_long_term_memories`（`agent_id -> Vec<LongTermMemoryEntry>`，默认空）。
- `LlmAgentBehavior` 增加长期记忆导出/恢复方法。
- Viewer live：
  - 初始化 LLM driver 时，从 `WorldModel` 恢复每个 agent 记忆。
  - 每 tick 执行后，将 runner 中记忆回写到 `WorldModel`，纳入 snapshot/journal 持久化闭环。

## 里程碑
- M1：文档与任务拆解完成（T0）。
- M2：间接控制链路完成（T1）。
- M3：WASM tick 生命周期与按需调度完成（T2）。
- M4：长期记忆持久化闭环完成（T3）。
- M5：回归、文档状态、devlog 收口（T4）。

## 风险
- 兼容性风险：`ActionEnvelope` / ABI 字段扩展需 `serde(default)`，避免旧快照反序列化失败。
- 行为风险：新增 submitter 授权后，现有调用方若未标注身份可能出现拒绝，需要保留 `system` 默认路径。
- 性能风险：tick 生命周期若调度/移除逻辑不当可能导致重复调用或漏调，需要测试覆盖 wake/suspend/无指令路径。
- 持久化风险：长期记忆回写时机若不稳定，可能造成内存态与快照态漂移，需要在 live tick 闭环中固定同步点。
