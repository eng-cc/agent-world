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
- `doc/world-simulator/llm/openclaw-agent-profile-agent_world_p0_low_freq_npc-2026-03-13.md`
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world/src/simulator/memory.rs`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world_client_launcher/src/*`

## 状态
- 最近更新：2026-03-15
- 当前阶段: T5 pending
- 当前任务: `继续压缩 OpenClaw system prompt / session 负载并扩面真实 parity 样本，收敛高延迟后准备 QA/producer 签收（experimental）`
- owner: `agent_engineer`
- 联审: `viewer_engineer`、`runtime_engineer`
- 发起建模: `producer_system_designer`
- 备注: `T1/T2` 已完成：launcher 已提供 provider mode/base URL/token/auto-discover/localhost health-check，`agent_world` 已补 mock local HTTP client 与 `/info`、`/health`、`/decision`、`/feedback` contract tests；`T3/T5` 的完成定义继续挂到 `PRD-WORLD_SIMULATOR-038`，真实用户机接入在 parity 未通过前仅能保持 `experimental`。
- 进展备注: `T3` 的实现范围已落地：`OpenClawAdapter` 已完成 mock local HTTP binding、`ProviderBackedAgentBehavior -> runtime -> feedback` 闭环回归，并补齐 `wait` / `wait_ticks` / `move_agent` / `speak_to_nearby` / `inspect_target` / `simple_interact` 六类 phase-1 白名单动作；其中后三者当前以 lightweight event 语义执行。`T3` 的最终签收仍继续挂到 `PRD-WORLD_SIMULATOR-038` parity 通过线，因此项目阶段前移到 `T4`。
- T4 预热进展: 已在 `agent_world_proto` / `viewer::protocol` 补齐 `AgentSpoke`、`TargetInspected`、`SimpleInteractionPerformed` 事件筛选枚举与匹配测试，为后续 Viewer 侧 provider 最近动作展示预留过滤入口。
- T4 完成备注: launcher 已补 `OpenClaw(Local HTTP)` 顶栏状态徽标、probe info/health/total 延迟、最近错误与队列深度摘要；viewer 已补 `Provider Debug` 文本卡片，输出最近 provider/model、最近延迟、最近动作/trace 摘要，并提供 `全部 / 仅 OpenClaw / 仅错误` 三档调试筛选入口。required 回归已覆盖 launcher probe 与 viewer provider debug summary。
- T5 预热补充: 已新增 `doc/world-simulator/llm/openclaw-agent-profile-agent_world_p0_low_freq_npc-2026-03-13.md`，并把 `DecisionRequest.agent_profile` 接通到 `ProviderBackedAgentBehavior -> OpenClawAdapter -> local HTTP` 与 parity bench / batch 脚本，首期 `P0` 默认 profile 固定为 `agent_world_p0_low_freq_npc`。
- T5 bridge 预热: 本机已确认 `OpenClaw Gateway` 正在 `127.0.0.1:18789` 运行，但默认安装未直接暴露 world-simulator provider 协议；因此追加 `world_openclaw_local_bridge` 作为 loopback-only 兼容桥，负责把 `openclaw agent --json` 转译成 `/v1/provider/info|health|/v1/world-simulator/decision|feedback`。
- T5 bridge 完成备注: `world_openclaw_local_bridge` 已落地到 `crates/agent_world/src/bin/world_openclaw_local_bridge.rs`，实机验证 `GET /v1/provider/info`、`GET /v1/provider/health`、`POST /v1/world-simulator/decision`、`POST /v1/world-simulator/feedback` 均可通过已安装的 `OpenClaw Gateway/CLI` 工作；真实 `P0` parity smoke 已能完成 2 步 decision 并产出 trace，但当前样本仍表现为 `wait` x2、`goal_completed=false`、`median_latency_ms≈4799`，所以 T5 仍保持 `experimental`，后续重点转向 prompt/profile 优化与更长样本采证。
- T5 session/guardrail 完成备注: 已补 `provider_config_ref` run-scoped session id，避免 bridge 把历史 `loc-2` 等旧样本上下文泄漏到新 benchmark；同时为 `P0-001` 补齐 scenario memory hint、reachable patrol guardrail 与“最近可见 location = 当前点”估算修正。实机 `P0-001` smoke（`artifacts/openclaw_parity_20260313_170850/...`）现已达到 `goal_completed=true`、`move_agent=4`、`invalid_action_count=0`，但 `llm_api median_latency_ms≈4781` 仍高于最终 parity 通过线，所以 T5 依然保持 `experimental`。
- T5 runtime-agent 补充: 已在 repo 内新增 `tools/openclaw/agent_world_runtime_workspace/*` 与 `scripts/setup-openclaw-agent-world-runtime.sh`，可一键安装轻量 `agent_world_runtime` OpenClaw agent；同时 bridge 的决策调用已切到 `openclaw gateway call agent --expect-final --json` + `sessionKey` 官方 RPC 形态。实机简单 probe 下，轻量 agent 已把 `promptTokens` 从约 `11885` 压到约 `9590`，`result.meta.durationMs` 从约 `4169ms` 降到约 `2191ms`；进一步压缩 repo-owned `BOOTSTRAP/TOOLS/IDENTITY/USER/HEARTBEAT` 后，真实 `P0-001` parity `median_latency_ms` 也从约 `5401` 小幅降到约 `5264`，但仍高于最终门禁，因此 T5 依然保持 `experimental`，下一步继续裁剪 system prompt / bootstrap 注入。
- T5 主链路补充: `agent_world_client_launcher` 已把 `agent_provider_mode/openclaw_base_url/openclaw_auth_token/openclaw_connect_timeout_ms/openclaw_agent_profile` 正式透传到 `world_game_launcher`；后者再通过环境变量把 OpenClaw 设置注入 `world_viewer_live` 的 runtime live sidecar，OpenClaw 现在可以走产品默认启动链路进入真实运行时。
- T5 操作流补充: `oasis7` 已新增 GitHub Release bundle-first 下载入口，`oasis7-run.sh download` 可直接下载并解压 `agent-world-<platform>` 发行包，`play` 则支持 `--bundle-dir` 与 `--repo-root` 显式路径策略；当前真实试玩推荐先拿 release bundle 跑 `run-game.sh`，再按需复用 repo 内 bridge / runtime-agent / parity tooling。
- T5 路径修复补充: `oasis7-run.sh` 现已在 `normalize_path` 中显式展开当前用户 `~`，修复默认 `--download-dir ~/.cache/oasis7/releases` 被误写到 repo-local `~/...` 的问题；同时新增 `.agents/skills/oasis7/scripts/oasis7-run-path-test.sh` 回归脚本，覆盖默认下载目录与 `~/custom-cache` override。
- 当前边界: runtime live 的 `agent_chat` / `prompt_control` 在 OpenClaw 模式下仍显式报 `unsupported`，避免对外误报“已支持玩家直连操控”。
