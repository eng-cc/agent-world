# Viewer Live 完全事件驱动改造 Phase 3（2026-02-26）

## 目标
- 将 LLM 决策触发从布尔门控改为 mailbox 事件化模型。
- 将“请求导致的 LLM 唤醒”统一走 live 内部事件总线，不在请求处理逻辑里直接改 world 状态。
- 保持现有 viewer 协议语义（Play/Pause/Step/Seek、AgentChat、PromptControl）不变。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/consensus_bridge.rs`
- `crates/agent_world/src/viewer/live/tests.rs`

不在范围内：
- 不引入跨 crate 的统一事件总线实现。
- 不修改 viewer 对外协议字段。

## 接口/数据
- 新增 live 内部信号：`LlmDecisionRequested`。
- `ViewerRequest` 处理产出“效果”（是否请求 LLM 决策），由主循环投递 `LlmDecisionRequested`。
- `LiveWorld` 中 LLM 触发状态由 `llm_decision_mailbox`（计数）管理：
  - 请求事件写入 mailbox（+1，饱和）
  - LLM tick 消费 mailbox（-1）
  - 若本次决策产生 action/event，可继续写回 mailbox（+1）维持连贯推进

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：mailbox + `LlmDecisionRequested` 事件接线。
3. M2：回归测试（mailbox 语义 + live 基线语义）。
4. M3：文档收口并明确后续总线收敛阶段。

## 风险
- mailbox 计数策略若与原布尔语义偏离，可能引起推进节奏变化。
- 请求线程/主循环事件投递时序错误可能导致漏触发。

## Phase 3 完成态（2026-02-26）
- 已完成：LLM 决策触发由布尔门控升级为 `llm_decision_mailbox` 计数模型，避免空请求重复空跑。
- 已完成：`ViewerRequest` 处理改为返回“效果”，通过 `LlmDecisionRequested` 信号在主循环中统一触发 LLM 唤醒。
- 已完成：补齐 mailbox 多请求消费与播放脉冲门控回归测试，`viewer::live::tests` 全组通过。

## Phase 4 入口（完全事件驱动收敛）
1. 将 `ViewerControl::Step/Seek` 从请求处理中的同步 world 变更，收敛为主循环事件执行。
2. 将共识提交动作应用从“step 内拉取”改为“提交即投递”的显式事件，统一顺序语义。
3. 为 live 事件队列引入有界容量、背压指标与丢弃/降级策略，避免慢消费者积压。
