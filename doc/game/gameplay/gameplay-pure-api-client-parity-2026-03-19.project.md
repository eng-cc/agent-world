# 纯 API 客户端等价玩法专题（项目管理文档）

- 对应设计文档: `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.design.md`
- 对应需求文档: `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.prd.md`

审计轮次: 1

## 任务拆解

- [x] TASK-GAMEPLAY-API-001 (`PRD-GAME-008`) [test_tier_required]: 冻结纯 API 等价专题 PRD / design / project，并完成 `game` 根 PRD、索引、顶层设计主文档与 devlog 挂载。
- [x] TASK-GAMEPLAY-API-002 (`PRD-GAME-008`) [test_tier_required]: `viewer_engineer` / `runtime_engineer` 已将 `player_gameplay` canonical 玩家语义下沉到 live `WorldSnapshot`，补齐 `stage / goal / progress / blocker / next_step / available_actions / recent_feedback` 的统一 schema，并让纯 API 客户端可直接消费 `FirstSessionLoop -> PostOnboarding` 的关键承接字段。
- [x] TASK-GAMEPLAY-API-003 (`PRD-GAME-008`) [test_tier_required]: `runtime_engineer` / `agent_engineer` / `viewer_engineer` 已交付 `world_pure_api_client` 参考纯 API 客户端，覆盖查看、推进、聊天/命令、阶段恢复与继续游玩所需核心动作，并复用现有 player auth 签名链路。
- [ ] TASK-GAMEPLAY-API-004 (`PRD-GAME-008`) [test_tier_required + test_tier_full]: `qa_engineer` 建立 UI/API parity matrix、required-tier 纯 API 长玩验证与 full-tier 长稳抽样，严格区分 `observer_only` 与 `playable parity`。

## 依赖

- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`
- `testing-manual.md`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world/src/viewer/runtime_live.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`

## 状态

- 更新日期: 2026-03-19
- 当前状态: in_progress
- 当前 owner: `producer_system_designer`
- 下一任务: `TASK-GAMEPLAY-API-004`
- 阻断条件:
  - 若关键玩家语义仍仅存在于 UI 组装层，则不得宣称纯 API 为正式可玩客户端。
  - 若 required-tier 仍只有协议 smoke、没有纯 API 长玩验证，则不得给出“持续可玩”结论。
  - 若 UI 可执行动作在 API 无正式入口，则 parity 结论必须保持 blocked。
- 说明:
  - 本专题目标是“玩法等价”，不是“视觉等价”。
  - 允许 CLI / TUI / JSON 等不同表现形式，但不允许降级玩家做决策所需的信息粒度。
  - 现阶段已有的 headless smoke 仅证明协议推进与事件流存在，不构成正式等价验收。
  - `TASK-GAMEPLAY-API-002` 已完成首个实现切片：live 协议快照新增 `player_gameplay`，当前覆盖 `FirstSessionLoop` 与 `PostOnboarding` 的 canonical 阶段目标、进度、阻塞、下一步建议、可执行动作和最近控制反馈。
  - `TASK-GAMEPLAY-API-003` 已完成首个参考客户端交付：`world_pure_api_client` 支持 `snapshot / step / play / pause / agent_chat / prompt_control / reconnect_sync / rotate_session / revoke_session`，并可直接生成/使用 Ed25519 玩家密钥。
  - `qa_engineer` 已新增 `scripts/world-pure-api-parity-smoke.sh`，并在 source required、fresh bundle required 与 bundle full 三条链路产出 `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md` 的证据。
  - 后续实现已补齐两个协议/表现层缺口：`reconnect-sync --with-snapshot` 现在会直接恢复 `player_gameplay`，Viewer Mission HUD / PostOnboarding 主卡也已优先消费 canonical `snapshot.player_gameplay`。
  - 当前剩余阻断收敛为 1 项：pure API required/full 仍未证明 no-LLM 路径能到达首个持续能力里程碑，说明正式动作面还不够完整。
