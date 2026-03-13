# OpenClaw 本地 HTTP Provider 接入 world-simulator 首期方案（2026-03-12）项目管理文档

- 对应设计文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.design.md`
- 对应需求文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 完成 `OpenClaw(Local HTTP)` 接入方案 PRD / Design / Project 建模，并回写模块主文档、索引与 devlog。
- [x] T1 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 在 launcher 落地 provider 模式配置、base URL/token/auto-discover 字段与本地 provider 健康检查。
- [x] T2 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 落地 mock local HTTP provider + adapter contract tests，冻结 `/info`、`/health`、`/decision`、`/feedback` 协议。
- [ ] T3 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 实现 `OpenClawAdapter` 与 `DecisionProvider` 绑定，支持首期低频动作白名单；完成定义改挂到 `PRD-WORLD_SIMULATOR-038` 的 parity 通过线。
- [x] T4 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 在 launcher / viewer 补 provider 连接状态、错误、最近延迟与最近动作/trace 摘要展示，并输出 parity 诊断所需字段。
- [ ] T5 (PRD-WORLD_SIMULATOR-037) [test_tier_full]: 以真实本机 `OpenClaw` 完成单一低频 NPC 闭环试点，验证“安装 -> 发现 -> 绑定 -> 决策 -> 恢复”用户路径；未通过 parity 前仅允许 `experimental`。

## 依赖
- `doc/world-simulator/llm/llm-decision-provider-standard-openclaw-feasibility-2026-03-12.prd.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world/src/simulator/memory.rs`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world_client_launcher/src/*`

## 状态
- 最近更新：2026-03-13
- 当前阶段: T5 pending
- 当前任务: `准备真实 OpenClaw(Local HTTP) 单 NPC parity 试点（experimental）`
- owner: `agent_engineer`
- 联审: `viewer_engineer`、`runtime_engineer`
- 发起建模: `producer_system_designer`
- 备注: `T1/T2` 已完成：launcher 已提供 provider mode/base URL/token/auto-discover/localhost health-check，`agent_world` 已补 mock local HTTP client 与 `/info`、`/health`、`/decision`、`/feedback` contract tests；`T3/T5` 的完成定义继续挂到 `PRD-WORLD_SIMULATOR-038`，真实用户机接入在 parity 未通过前仅能保持 `experimental`。
- 进展备注: `T3` 的实现范围已落地：`OpenClawAdapter` 已完成 mock local HTTP binding、`ProviderBackedAgentBehavior -> runtime -> feedback` 闭环回归，并补齐 `wait` / `wait_ticks` / `move_agent` / `speak_to_nearby` / `inspect_target` / `simple_interact` 六类 phase-1 白名单动作；其中后三者当前以 lightweight event 语义执行。`T3` 的最终签收仍继续挂到 `PRD-WORLD_SIMULATOR-038` parity 通过线，因此项目阶段前移到 `T4`。
- T4 预热进展: 已在 `agent_world_proto` / `viewer::protocol` 补齐 `AgentSpoke`、`TargetInspected`、`SimpleInteractionPerformed` 事件筛选枚举与匹配测试，为后续 Viewer 侧 provider 最近动作展示预留过滤入口。
- T4 完成备注: launcher 已补 `OpenClaw(Local HTTP)` 顶栏状态徽标、probe info/health/total 延迟、最近错误与队列深度摘要；viewer 已补 `Provider Debug` 文本卡片，输出最近 provider/model、最近延迟、最近动作/trace 摘要，并提供 `全部 / 仅 OpenClaw / 仅错误` 三档调试筛选入口。required 回归已覆盖 launcher probe 与 viewer provider debug summary。
