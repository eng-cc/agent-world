# world_viewer_live LLM 默认开启（P2P 发行基线）设计文档（2026-02-23）

## 目标
- 将 `world_viewer_live` 的决策模式默认值从 Script 调整为 LLM，减少发行参数遗漏导致的节点行为偏差。
- 在不破坏现有参数模型的前提下，保持 CLI 语义简单且可审计。
- 与已上线的 `--release-config` 锁定参数路径保持一致：未显式写入 `--llm` 时也能默认进入 LLM 决策。

## 范围

### In Scope
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - 调整 `CliOptions::default().llm_mode` 为 `true`。
  - 更新 `--llm` 帮助文案，避免“默认关闭”误导。
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`
  - 更新默认解析测试断言，覆盖“默认开启 LLM”。
- 文档
  - 更新 Viewer 手册，明确 `--llm` 为默认行为。

### Out of Scope
- 不新增 `--no-llm` 反向开关。
- 不调整 `WorldScenario` 默认值与场景集合。
- 不改动 `--release-config` 白名单策略。

## 接口 / 数据

### 1) CLI 默认语义
- 变更前：不传 `--llm` 时 `llm_mode=false`（Script）。
- 变更后：不传 `--llm` 时 `llm_mode=true`（LLM）。

### 2) CLI 参数兼容性
- `--llm` 参数继续保留（显式冗余开启，不影响结果）。
- 发行锁定 `locked_args` 可继续包含或省略 `--llm`，最终都以开启 LLM 运行。

## 里程碑
- M0：完成设计/项目文档建档。
- M1：完成 `llm_mode` 默认值切换与帮助文案更新。
- M2：完成测试与手册更新，执行定向回归。
- M3：完成文档状态/devlog 收口。

## 风险
- 行为变更风险：依赖 Script 默认行为的本地调试脚本将出现行为变化。
- 可观测性风险：若文档未同步，运维可能误判当前决策模式。
- 回归风险：虽然改动点小，仍需确认 `world_viewer_live` 全量单测通过。

## 里程碑状态（2026-02-23）
- M0：完成（设计/项目文档建档）。
- M1：进行中。
- M2：待开始。
- M3：待开始。
