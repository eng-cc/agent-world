# OpenClaw 本地 HTTP Provider 接入 world-simulator 首期方案（2026-03-12）项目管理文档

- 对应设计文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.design.md`
- 对应需求文档: `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 完成 `OpenClaw(Local HTTP)` 接入方案 PRD / Design / Project 建模，并回写模块主文档、索引与 devlog。
- [ ] T1 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 在 launcher 落地 provider 模式配置、base URL/token/auto-discover 字段与本地 provider 健康检查。
- [ ] T2 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 落地 mock local HTTP provider + adapter contract tests，冻结 `/info`、`/health`、`/decision`、`/feedback` 协议。
- [ ] T3 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 实现 `OpenClawAdapter` 与 `DecisionProvider` 绑定，支持首期低频动作白名单。
- [ ] T4 (PRD-WORLD_SIMULATOR-037) [test_tier_required]: 在 launcher / viewer 补 provider 连接状态、错误、最近延迟与最近动作/trace 摘要展示。
- [ ] T5 (PRD-WORLD_SIMULATOR-037) [test_tier_full]: 以真实本机 `OpenClaw` 完成单一低频 NPC 闭环试点，验证“安装 -> 发现 -> 绑定 -> 决策 -> 恢复”用户路径。

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
- 最近更新：2026-03-12
- 当前阶段: T1 pending
- 当前任务: `冻结本地 HTTP 发现/握手字段并规划 launcher provider 配置入口`
- owner: `agent_engineer`
- 联审: `viewer_engineer`、`runtime_engineer`
- 发起建模: `producer_system_designer`
- 备注: 当前仅完成方案设计，不进入实现；后续必须先完成 T1/T2，确认 mock local HTTP contract 与失败签名后，才允许对接真实用户机上的 `OpenClaw` 服务。
