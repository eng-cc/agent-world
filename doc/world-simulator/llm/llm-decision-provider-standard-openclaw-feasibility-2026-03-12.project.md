# LLM Agent Decision Provider 标准层 + OpenClaw 外部适配可行性（2026-03-12）项目管理文档

- 对应设计文档: `doc/world-simulator/llm/llm-decision-provider-standard-openclaw-feasibility-2026-03-12.design.md`
- 对应需求文档: `doc/world-simulator/llm/llm-decision-provider-standard-openclaw-feasibility-2026-03-12.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-036) [test_tier_required]: 完成 `Decision Provider` 标准层 + `OpenClaw` 外部适配可行性 PRD / Design / Project 建模，并回写模块主文档、索引与 devlog。
- [ ] T1 (PRD-WORLD_SIMULATOR-036) [test_tier_required]: 在 simulator 侧冻结 provider contract 类型与 golden observation fixture，形成 provider-agnostic 契约测试样本。
- [ ] T2 (PRD-WORLD_SIMULATOR-036) [test_tier_required]: 实现 `MockProvider`，验证 `AgentBehavior facade -> DecisionProvider -> runtime -> trace` 最小闭环可离线运行。
- [ ] T3 (PRD-WORLD_SIMULATOR-036) [test_tier_full]: 实现 `OpenClawAdapter` PoC，限定在低频、低破坏性动作集上试点。
- [ ] T4 (PRD-WORLD_SIMULATOR-036) [test_tier_required]: 完成 provider trace / memory write intent / error policy 映射，保持与 viewer/QA 诊断契约一致。
- [ ] T5 (PRD-WORLD_SIMULATOR-036) [test_tier_full]: 选取单一低频 NPC 场景做闭环评估，对比本地 provider 与 `OpenClaw` provider 的动作有效率、超时率、成本与 trace 完整度。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world/src/simulator/memory.rs`
- `crates/agent_world_proto/src/viewer.rs`
- `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.md`
- `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.prd.md`

## 状态
- 最近更新：2026-03-12
- 当前阶段: T1 pending
- 当前任务: `冻结 provider contract 类型与 golden observation fixture`
- owner: `agent_engineer`
- 联审: `runtime_engineer`、`viewer_engineer`
- 发起建模: `producer_system_designer`
- 备注: 当前仅完成文档建模，不进入实现；后续若要真正验证 `OpenClaw` 可行性，必须先完成 `MockProvider` 与 fixture 契约测试，禁止跳过 T1/T2 直接接外部 provider。
