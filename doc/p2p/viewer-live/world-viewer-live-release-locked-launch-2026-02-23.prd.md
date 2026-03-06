# world_viewer_live 发行锁定启动（P2P）设计文档（2026-02-23）

审计轮次: 3

## 1. Executive Summary
- Problem Statement: 将 `world_viewer_live` 从“开发期高自由度 CLI”收口为“发行期锁定参数启动”，避免 P2P 上线后因单节点临时调参导致网络语义漂移。
- Proposed Solution: 发行部署提供稳定、可复现、可审计的启动输入；运行时仅允许极少数非共识语义覆盖项。
- Success Criteria:
  - SC-1: 保持现有开发/联调体验兼容：未启用发行锁定时，原有 CLI 行为不变。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want world_viewer_live 发行锁定启动（P2P）设计文档（2026-02-23） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - AC-2: 新增发行锁定入口：`--release-config <path>`。
  - AC-3: 支持从 TOML 读取 `locked_args` 并复用现有 `parse_options` 解析。
  - AC-4: 在发行锁定模式限制 CLI 覆盖项白名单（仅允许 `--bind`、`--web-bind`、`--help`、`--release-config`）。
  - AC-5: `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`
  - AC-6: 启动入口改为新解析函数（支持发行锁定模式）。
- Non-Goals:
  - 不重构现有 `CliOptions` 数据模型。
  - 不引入远程配置中心或链上治理参数热更新。
  - 不改变当前 triad/triad_distributed 拓扑语义与默认值。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.md`
  - `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
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

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：完成（设计/项目文档建档）。
  - M1：完成（`--release-config` 与 `locked_args` 加载接线落地）。
  - M2：完成（发行锁定模式 CLI 白名单约束落地）。
  - M3：完成（测试、样例、手册与回归闭环）。
- Technical Risks:
  - 兼容性风险：现有脚本若误加 `--release-config` 且仍传大量 CLI，会被拒绝，需要同步脚本模板。
  - 运维风险：`locked_args` 文件配置错误会导致节点启动失败，需要清晰报错和样例模板降低误配概率。
  - 可观测性风险：发行模式下参数来源从 CLI 转为文件，需在文档中明确“锁定文件为单一事实来源”。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-115-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-115-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
