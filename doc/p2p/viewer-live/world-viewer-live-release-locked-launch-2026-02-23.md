# world_viewer_live 发行锁定启动（P2P）设计文档（2026-02-23）

## 目标
- 将 `world_viewer_live` 从“开发期高自由度 CLI”收口为“发行期锁定参数启动”，避免 P2P 上线后因单节点临时调参导致网络语义漂移。
- 发行部署提供稳定、可复现、可审计的启动输入；运行时仅允许极少数非共识语义覆盖项。
- 保持现有开发/联调体验兼容：未启用发行锁定时，原有 CLI 行为不变。

## 范围

### In Scope
- `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - 新增发行锁定入口：`--release-config <path>`。
  - 支持从 TOML 读取 `locked_args` 并复用现有 `parse_options` 解析。
  - 在发行锁定模式限制 CLI 覆盖项白名单（仅允许 `--bind`、`--web-bind`、`--help`、`--release-config`）。
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
  - 启动入口改为新解析函数（支持发行锁定模式）。
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests*.rs`
  - 补充发行锁定模式解析测试（成功/拒绝/覆盖）。
- 文档与样例
  - 新增发行锁定配置样例（`locked_args`）。
  - 在 Viewer 手册补充发行启动方式。

### Out of Scope
- 不重构现有 `CliOptions` 数据模型。
- 不引入远程配置中心或链上治理参数热更新。
- 不改变当前 triad/triad_distributed 拓扑语义与默认值。

## 接口 / 数据

### 1) 新增 CLI 接口
- `--release-config <path>`
  - 启用发行锁定模式。
  - 从 `<path>` 读取 TOML 文件，要求包含 `locked_args` 数组。

### 2) 发行锁定配置文件格式
- 文件格式（TOML）：
  - `locked_args = ["llm_bootstrap", "--topology", "triad_distributed", ...]`
- 语义：
  - `locked_args` 代表“发行固定参数全集”。
  - 加载后复用现有参数校验逻辑，保证与开发路径一致。

### 3) 发行锁定模式 CLI 白名单
- 允许：
  - `--release-config <path>`
  - `--bind <addr>`
  - `--web-bind <addr>`
  - `--help/-h`
- 拒绝：
  - 其他任意运行时调参项（例如 `--topology`、`--node-*`、`--reward-*` 等）。
- 冲突处理：
  - 如果白名单覆盖项与 `locked_args` 同时存在，CLI 覆盖项优先（仅对白名单项生效）。

## 里程碑
- M0：完成设计/项目文档建档。
- M1：完成发行锁定入口与文件解析接线。
- M2：完成 CLI 白名单约束与错误信息收口。
- M3：补齐测试、样例、手册，完成回归与文档状态闭环。

## 风险
- 兼容性风险：现有脚本若误加 `--release-config` 且仍传大量 CLI，会被拒绝，需要同步脚本模板。
- 运维风险：`locked_args` 文件配置错误会导致节点启动失败，需要清晰报错和样例模板降低误配概率。
- 可观测性风险：发行模式下参数来源从 CLI 转为文件，需在文档中明确“锁定文件为单一事实来源”。

## 里程碑状态（2026-02-23）
- M0：完成（设计/项目文档建档）。
- M1：完成（`--release-config` 与 `locked_args` 加载接线落地）。
- M2：完成（发行锁定模式 CLI 白名单约束落地）。
- M3：完成（测试、样例、手册与回归闭环）。
