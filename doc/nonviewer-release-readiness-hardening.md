# Non-Viewer 发行准备加固（测试覆盖 + Agent/PublicKey 绑定）

## 目标
- 补齐 non-viewer 发布门禁测试覆盖，避免 `node/consensus/distfs` 回归漏检。
- 将 Agent 控制绑定从“仅 player_id”提升为“player_id + public_key”，收敛控制身份边界。
- 保持现有 `test_tier_required` / `test_tier_full` 与协议兼容，优先增量改造。

## 范围

### In Scope
- 更新 `scripts/ci-tests.sh`：
  - 在默认门禁中补齐 `agent_world_node`、`agent_world_consensus`、`agent_world_distfs`（以及 net 基础库测试）执行。
- 更新 `testing-manual.md`：
  - 回写 CI 已覆盖口径与分层说明。
- Runtime/Viewer 控制绑定增强：
  - Agent 绑定记录新增 `public_key` 维度（支持 legacy 无 key 兼容）。
  - PromptControl/AgentChat 请求新增可选 `public_key` 字段。
  - 已绑定 Agent 在控制时要求 `public_key` 一致。
  - 首次绑定优先建立 `public_key` 绑定，不允许后续漂移。
- 增补对应单测/回归测试。

### Out of Scope
- viewer 视觉表现与交互体验改造。
- 新增链上鉴权协议或网络签名协议重构。
- LLM 策略和 gameplay 数值平衡重做。

## 接口/数据
- Viewer 协议请求字段（新增）：
  - `PromptControlApplyRequest.public_key: Option<String>`
  - `PromptControlRollbackRequest.public_key: Option<String>`
  - `AgentChatRequest.public_key: Option<String>`
- World 模型字段（新增）：
  - `agent_player_public_key_bindings: BTreeMap<AgentId, String>`
- 事件兼容扩展：
  - `WorldEventKind::AgentPlayerBound` 新增可选 `public_key` 字段，保持旧日志可回放。
- Kernel 接口扩展：
  - 提供按 `public_key` 的绑定与查询入口。

## 里程碑
- M0：建档并冻结任务拆解。
- M1：测试门禁覆盖补齐并通过 required 回归。
- M2：Agent/PublicKey 绑定实现与协议测试通过。
- M3：文档与 devlog 收口。

## 风险
- required 门禁耗时上升。
  - 缓解：先纳入 `--lib` 级测试，必要时再分层并行化。
- 绑定规则增强可能影响旧客户端。
  - 缓解：协议字段保持可选，保留 legacy 兼容路径并补充错误码提示。
- 历史快照/日志兼容风险。
  - 缓解：新增字段全部 `serde(default)`，补 replay/roundtrip 测试。
