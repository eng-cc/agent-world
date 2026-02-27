# Viewer Live 完全事件驱动改造 Phase 7（2026-02-27）

## 目标
- 明确 script 路径的节拍策略（保留定时脉冲用于节奏控制，或切换为显式驱动节拍事件）。
- 为 live 主循环补充信号级可观测指标（吞吐、merge/drop、处理延迟），支撑达成度评估。
- 形成“完全事件驱动”验收清单并完成闭环验证。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

不在范围内：
- 不修改对外 viewer 协议。
- 不改动 node/runtime 共识协议。

## 接口/数据
- 新增 live 信号统计快照结构（内部），至少包含：
  - 每类信号入队数、合并数、丢弃数。
  - 主循环处理耗时分位（或最小集合：平均/最大）。
- script 节拍策略配置：
  - `timer_pulse`（兼容）
  - `event_drive`（实验）

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：script 节拍策略可切换实现（默认兼容）。
3. M2：主循环信号级可观测指标接线。
4. M3：回归测试 + Web 闭环验证。
5. M4：文档收口，输出完全事件驱动达成度结论。

## 风险
- script 路径若切换为纯事件驱动，可能出现推进速度与 UI 预期节奏不一致。
- 指标采样若引入额外锁竞争，可能反向放大主循环延迟。

## Phase 7 完成态（T4）

### 交付结果
- script 节拍策略完成双模接线：
  - `timer_pulse`（默认兼容）
  - `event_drive`（实验模式）
- live 主循环补齐信号级可观测性：
  - 每类信号入队数（in）、合并数（merge）、丢弃数（drop）
  - 每类信号处理吞吐（handled）与处理耗时（avg_us/max_us）
- S6 Web 闭环完成最小 smoke 验证（headed + `__AW_TEST__` + console/screenshot）。

### 验收证据
- 回归测试（test_tier_required）：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required viewer::live::tests:: -- --nocapture`
- Web 闭环产物（S6）：
  - `output/playwright/viewer/phase7-t3-web-smoke-20260227-112259.png`
  - `output/playwright/viewer/phase7-t3-live-20260227-112259.log`
  - `output/playwright/viewer/phase7-t3-web-20260227-112259.log`
- 发行 QA 附加门禁结果：
  - `output/playwright/viewer/release-qa-summary-20260227-112032.md`
  - 语义 gate 通过，zoom texture gate 失败（`zoom stage near initial: not connected`）。

### 完全事件驱动达成度结论
- 共识路径：已达成（由 committed 事件与 drive 请求驱动，不依赖播放脉冲）。
- 非共识 LLM 路径：已达成（由 LLM 请求事件驱动，空闲时不重复推进）。
- script 路径：功能上已达成可切换事件驱动（`event_drive`），但默认仍为 `timer_pulse` 兼容模式。

结论：系统已具备“可运行的完全事件驱动模式”，但默认运行口径尚未切换到该模式。

### 后续改造点（切到默认完全事件驱动）
1. 将 script 默认节拍从 `timer_pulse` 切换到 `event_drive`，并提供发布期开关回退策略。
2. 在 script `event_drive` 下引入节奏整形（避免高活跃场景下的连续重入导致 UI 侧瞬时压力）。
3. 收敛/下线 playback pulse 线程在默认路径中的职责，仅保留兼容或压测模式。
4. 修复 release QA 的 zoom texture gate 连接抖动（`near initial` 阶段短暂 `connecting`）并恢复一键全绿。
