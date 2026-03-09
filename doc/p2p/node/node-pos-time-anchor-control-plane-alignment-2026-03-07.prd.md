# Agent World Runtime：PoS 时间锚定控制面参数与可观测口径对齐

审计轮次: 2

## 1. Executive Summary
- Problem Statement: `agent_world_node` 内部已实现 wall-clock slot 与 logical tick 语义，但 `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher/scripts` 仍存在残留口径偏差，导致“轮询间隔”和“出块时间参数”容易混淆。
- Proposed Solution: 在 runtime/viewer/launcher/scripts 明确暴露并校验 PoS 时间锚定参数，保留 `node_tick_ms` 仅作为 worker 轮询/回退间隔，并扩展状态接口字段保证运维观测一致。
- Success Criteria:
  - SC-1: `world_chain_runtime/world_game_launcher/world_web_launcher/agent_world_client_launcher` 可配置并透传 `slot_duration_ms/ticks_per_slot/proposal_tick_phase` 等核心参数。
  - SC-2: launcher 可配置并透传上述参数，配置校验失败可定位到具体字段。
  - SC-3: `/v1/chain/status` 暴露 tick-phase/漏 tick/调度开关字段，脚本可直接采样。
  - SC-4: longrun/s10 脚本与发布示例文档口径统一，`node_tick_ms` 不再描述为出块时间。
  - SC-5: required/full 回归通过，且兼容未显式设置新参数的旧调用。

## 2. User Experience & Functionality
- User Personas:
  - 协议工程师：需要确保控制面参数与协议公式一致。
  - 节点运营者：需要可预期的调参入口与状态反馈。
  - QA/发布工程师：需要脚本与门禁口径稳定。
  - 启动器用户：需要 UI 字段语义明确，避免误配。
- User Scenarios & Frequency:
  - 每次 PoS 参数调优时通过 CLI/launcher 调整并验证。
  - 每次 S9/S10 长跑前检查脚本参数与状态采样字段。
  - 每次发布配置固化时维护 release 锁定参数模板。
- User Stories:
  - PRD-P2P-NODE-SURFACE-001: As a 协议工程师, I want runtime + launcher control-plane to accept slot-clock parameters, so that consensus timing is configured explicitly.
  - PRD-P2P-NODE-SURFACE-002: As a 启动器用户, I want launcher fields to separate polling interval from block-time parameters, so that tuning does not drift.
  - PRD-P2P-NODE-SURFACE-003: As a QA/发布工程师, I want status+soak outputs to expose tick-phase metrics, so that gates can audit timing behavior.
- Critical User Flows:
  1. Flow-NODE-SURFACE-001: `用户启动 world_chain_runtime -> 传入 slot/tick 参数 -> 参数校验 -> runtime 生效`
  2. Flow-NODE-SURFACE-002: `用户启动 world_game_launcher/world_web_launcher -> 透传 chain-pos 参数到 world_chain_runtime -> UI/日志不误导 node_tick_ms 语义`
  3. Flow-NODE-SURFACE-003: `launcher 配置参数 -> 构建链运行时参数 -> 启动失败时返回字段级错误`
  4. Flow-NODE-SURFACE-004: `脚本采样 /v1/chain/status -> 聚合 slot/tick_phase/missed_tick -> 输出门禁结论`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| runtime CLI 参数映射 | `--pos-slot-duration-ms`、`--pos-ticks-per-slot`、`--pos-proposal-tick-phase`、`--pos-adaptive-tick-scheduler`、`--pos-slot-clock-genesis-unix-ms`、`--pos-max-past-slot-lag` | 解析并写入 `NodePosConfig` | `parsing -> validated -> running` | 非法值拒绝启动 | 运行节点操作者可配置 |
| game/web/client launcher 参数透传 | `--chain-node-tick-ms` + `--chain-pos-*` 字段 | 校验后构造 `world_chain_runtime` 启动参数 | `editing -> validated -> started` | `chain_node_tick_ms` 仅用于 worker 轮询/回退间隔 | 本地运维与测试操作者可配置 |
| launcher 配置透传 | `chain_node_tick_ms` + PoS 参数字段 | UI 录入、校验、构建 args | `editing -> validated -> launched` | 参数缺失使用默认值，非法值拒绝 | 本地用户可配置 |
| chain status 观测扩展 | `worker_poll_count`（兼容 `tick_count`）+ `ticks_per_slot/tick_phase/proposal_tick_phase/last_observed_tick/missed_tick_count/adaptive_tick_scheduler_enabled` | `/v1/chain/status` 返回并供脚本消费 | `collecting -> reported` | worker 轮询与共识节拍字段必须分离命名 | 只读公开 |
| 脚本/文档口径 | `--node-tick-ms` 文案 + 新参数文案 | 更新 help/示例/run_config 输出 | `legacy -> aligned` | 保持旧参数可用但语义更正 | 测试维护者可调整 |
- Acceptance Criteria:
  - AC-1: `world_chain_runtime` 支持并校验全部 PoS 时间锚定参数。
  - AC-2: `world_game_launcher/world_web_launcher/agent_world_client_launcher` 支持并校验全部 PoS 时间锚定参数，并正确透传到 `world_chain_runtime`。
  - AC-3: launcher 配置项可编辑并透传 PoS 时间锚定参数。
  - AC-4: `/v1/chain/status` 回显 tick-phase 相关字段，脚本采样不依赖推断。
  - AC-5: `p2p-longrun`/`s10`/release 示例文档更新到统一口径。
  - AC-6: required/full 回归覆盖 CLI、参数校验、状态字段与脚本兼容路径。
- Non-Goals:
  - 不改造 PoS 共识安全阈值或 fork-choice 算法。
  - 不变更链上经济参数与结算逻辑。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `NodePosConfig` 为单一事实源；runtime/viewer/launcher/scripts 只负责参数映射与校验；status 接口提供观测投影。
- Integration Points:
  - `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.project.md`
  - `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
  - `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_game_launcher.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world_client_launcher/src/launcher_core.rs`
  - `crates/agent_world_client_launcher/src/llm_settings.rs`
  - `crates/agent_world_client_launcher/src/llm_settings_web.rs`
  - `crates/agent_world_launcher_ui/src/lib.rs`
  - `scripts/p2p-longrun-soak.sh`
  - `scripts/s10-five-node-game-soak.sh`
  - `world_viewer_live.release.example.toml`
- Edge Cases & Error Handling:
  - `ticks_per_slot == 0` 或 `slot_duration_ms == 0`：CLI/launcher 拒绝并返回字段级错误。
  - `proposal_tick_phase >= ticks_per_slot`：启动前拒绝。
  - `slot_clock_genesis_unix_ms` 非法整型：解析失败并提示参数名。
  - 未提供新参数：沿用 `NodePosConfig` 默认值，保持兼容。
  - `node_tick_ms` 过小导致高 CPU：允许配置但在文档中标注为轮询间隔风险。
  - `world_viewer_live` 误传 `--node-*`/`--triad-*`：必须显式拒绝并提示改用 `world_chain_runtime`。
  - status 投影缺失字段：测试应阻断并提示 schema 漂移。
- Non-Functional Requirements:
  - NFR-1: 参数映射与校验开销保持 O(1)。
  - NFR-2: status 字段投影与快照保持 1:1，不引入推导字段漂移；`worker_poll_count` 与共识 tick 字段不可混名。
  - NFR-3: 旧命令仅含 `--node-tick-ms` 时仍可启动（兼容保留）。
  - NFR-4: 文案不再把 `node_tick_ms` 定义为出块时间。
- Security & Privacy: 参数映射不得绕过现有签名、validator 身份与时间窗口校验逻辑。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0: 建档并定义任务拆解。
  - M1: runtime/viewer CLI 与状态接口对齐。
  - M2: launcher + 脚本/文档对齐。
  - M3: required/full 回归收口。
- Technical Risks:
  - 风险-1: CLI 参数新增后若默认值处理不当，可能改变现网节奏。
  - 风险-2: 多入口参数名不一致会造成运维误配。
  - 风险-3: 状态接口 schema 变化可能破坏既有脚本解析。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-NODE-SURFACE-001 | TASK-P2P-010-T1 | `test_tier_required` | runtime/viewer CLI 解析与参数校验测试 | 节点启动与 PoS 参数映射 |
| PRD-P2P-NODE-SURFACE-002 | TASK-P2P-010-T2 | `test_tier_required` | launcher 字段校验与参数透传测试 | 客户端启动控制面 |
| PRD-P2P-NODE-SURFACE-003 | TASK-P2P-010-T3/T4 | `test_tier_required` + `test_tier_full` | longrun/s10 脚本采样 + 状态 schema + 端到端 smoke | 发布门禁与运维观测 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-NODE-SURFACE-001 | 显式新增 PoS 时间锚定参数入口 | 继续仅暴露 `node_tick_ms` | 后者语义混淆且无法精确调参。 |
| DEC-NODE-SURFACE-002 | 保留 `node_tick_ms` 兼容但重定义为轮询间隔 | 直接删除旧参数 | 兼容现有脚本与外部调用，降低迁移风险。 |
| DEC-NODE-SURFACE-003 | status 直接投影快照字段供脚本消费 | 由脚本自行推导 | 推导路径容易漂移，难以审计。 |
