# world_viewer_live `--no-llm` 关闭开关设计文档（2026-02-23）

## 目标
- 在 `world_viewer_live` 默认启用 LLM 决策的前提下，提供显式关闭开关 `--no-llm`，用于本地调试或脚本回退到 Script 决策。
- 保持参数语义可预期：默认值稳定、显式参数可覆盖。
- 与现有 P2P 发行锁定模式兼容，不放开发行运行时临时调参口。

## 范围

### In Scope
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - 新增 `--no-llm` 参数解析，设置 `llm_mode=false`。
  - 更新 CLI help/usage 文案，明确 `--llm` 默认开启且可由 `--no-llm` 关闭。
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`
  - 补充 `--no-llm` 参数行为测试。
  - 覆盖 release-config `locked_args` 中 `--no-llm` 的生效行为测试。
- 文档
  - 更新 Viewer 手册，补充 `--no-llm` 使用方式。

### Out of Scope
- 不变更 `--release-config` 模式下 CLI 白名单（仍仅允许 `--release-config`、`--bind`、`--web-bind`、`--help`）。
- 不变更场景默认值与拓扑相关参数语义。
- 不新增额外决策模式枚举，仅在 LLM/Script 之间切换。

## 接口 / 数据

### 1) CLI 新增参数
- `--no-llm`
  - 语义：关闭 LLM 决策，改用 Script 决策。
  - 默认值：`llm_mode=true`（保持不变）。

### 2) 同时出现 `--llm` 与 `--no-llm` 的处理
- 采用“按出现顺序覆盖，最后出现者生效”语义，符合现有线性解析行为。

### 3) 与发行锁定模式的关系
- 运行时 `--release-config` CLI 白名单不变，仍不允许直接传 `--no-llm`。
- 若需发行禁用 LLM，应在发行文件 `locked_args` 中显式写入 `--no-llm`。

## 里程碑
- M0：完成设计/项目文档建档。
- M1：完成 `--no-llm` 参数解析、help 文案与测试。
- M2：完成手册更新与定向回归。
- M3：完成文档状态/devlog 收口。

## 风险
- 参数冲突风险：用户同时传 `--llm`/`--no-llm` 时若缺少文档说明，易误解最终行为。
- 发行配置风险：运维若误以为可在 release-config 运行时覆盖 `--no-llm`，会触发白名单拒绝。
- 回归风险：参数变更虽小，但需确认 `world_viewer_live` 全量单测通过。

## 里程碑状态（2026-02-23）
- M0：完成（设计/项目文档建档）。
- M1：进行中。
- M2：待开始。
- M3：待开始。
