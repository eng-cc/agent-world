# Viewer Live Step 控制推进稳定性设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-step-control-progress-stability-2026-02-28.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-step-control-progress-stability-2026-02-28.project.md`

## 1. 设计定位
定义 live + consensus 路径下 `step` 控制的可兑现性修复方案：减少“accepted 但无推进”的黑盒体验，把问题从路径性阻断收敛为可解释的异步时延波动。

## 2. 设计结构
- 请求处理层：`handle_step_request` 负责 step 请求与短时重试窗口。
- 提交消费层：paused 会话下继续消费 `ConsensusCommitted`，避免提交后卡住。
- 观测指标层：以 `tick/eventSeq` 增长和 A/B 指标衡量 step 是否实际推进。
- 后续增强层：为 completion ack 之类更强语义预留扩展空间。

## 3. 关键接口 / 入口
- `ViewerControl::Step { count }`
- `handle_step_request`
- `handle_consensus_committed`
- `scripts/run-game-test-ab.sh --headless`

## 4. 约束与边界
- 本阶段不改协议字段，只修复 server 侧推进与测试稳定性。
- 短时重试只能缓解异步抖动，不能伪造已完成推进。
- live + consensus 路径是修复重点，不能误伤非共识场景行为。
- 后续若要强语义 step，需另开 completion ack 专题，不在本轮硬塞。

## 5. 设计演进计划
- 先复现 paused+consensus 卡住根因。
- 再修提交消费和短时重试逻辑。
- 最后用回归测试与 A/B 复测冻结修复效果。
