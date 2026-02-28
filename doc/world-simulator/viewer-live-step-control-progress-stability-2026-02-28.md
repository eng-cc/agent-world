# Viewer Live `step` 控制推进稳定性修复（2026-02-28）

## 目标
- 复现并收敛 Web 可玩性测试中 `step` 控制“accepted 但无推进”的不稳定问题。
- 在不改协议的前提下，提升 live + consensus 路径下 `step` 的可兑现性，降低玩家黑盒感。

## 范围
- 修改 `crates/agent_world/src/viewer/live_split_part1.rs` 中 live 主循环的控制推进逻辑。
- 修改 `crates/agent_world/src/viewer/live/tests.rs`，新增针对 paused 会话下 consensus 提交回放的回归测试。
- 通过 `scripts/run-game-test-ab.sh --headless` 做实机复测并记录量化结果。

## 接口/数据
- 输入控制接口：`ViewerRequest::{Control, LiveControl}` 中 `ViewerControl::Step { count }`。
- 关键链路：
  - `handle_step_request`：step 请求处理与执行。
  - `handle_consensus_committed`：共识提交动作回放到 world。
- 关键观测：
  - `tick/eventSeq` 是否在 step 控制窗口内增长。
  - A/B 指标：TTFC、有效控制命中率、无进展窗口、B 段 pass/fail。

## 里程碑
- M1：复现场景并定位根因（step 触发后 session 处于 paused，已提交共识动作在部分路径未被及时消费）。
- M2：代码修复
  - 允许 paused 会话下继续消费 `ConsensusCommitted`。
  - `step` 在 consensus 路径增加短时重试窗口，降低“提交瞬时无回放”的概率。
- M3：测试与复测
  - 新增单测覆盖 paused 会话下 consensus committed 仍可推进。
  - 执行 targeted 单测与 A/B 实测。

## 风险
- 在 LLM + consensus 异步提交模型下，`step` 仍可能受提交时延与动作产出波动影响，短时重试只能缓解不能完全消除。
- 若后续要求 `step(count=n)` 强语义（严格 n 次可见推进），可能需要引入请求-确认协议或显式 completion ack。
