# README 生产级收口：LLM 制度动作 + DistFS 状态主路径 + 去中心化默认拓扑（设计文档）

## 目标
- 收口差距 1：将 LLM 主决策链路扩展为可直接使用市场/制度相关动作，避免仅限工业闭环动作。
- 收口差距 2：将世界状态持久化升级为 DistFS 默认写入与优先恢复路径，保留 JSON 兼容兜底。
- 收口差距 3：将 live 运行默认拓扑从单节点调试形态升级为本地多节点生产预设（triad），并保持共识门控默认开启。

## 范围
- In scope
  - `crates/agent_world/src/simulator/llm_agent`
    - 扩展 decision schema 与 parser，支持 power 市场与 social 制度动作。
    - 增加参数校验与拒绝路径，避免无效动作进入内核。
    - 新增/更新 `test_tier_required` 用例覆盖新动作解析与提示词契约。
  - `crates/agent_world/src/runtime/world/persistence.rs`
    - `save_to_dir` 默认追加 DistFS 分段写入（snapshot manifest + journal segments）。
    - `load_from_dir` 优先使用 DistFS 元数据恢复，失败后回退 JSON。
    - 新增持久化回归测试，覆盖 DistFS 恢复主路径与 JSON 兜底路径。
  - `crates/agent_world/src/bin/world_viewer_live*`
    - 新增生产默认拓扑（local triad）配置项。
    - 在默认配置下启动 sequencer/storage/observer 三节点，互联 gossip，viewer 绑定主节点门控。
    - 增加 CLI 解析与拓扑构建测试。
- Out of scope
  - 跨主机自动发现/自动组网（仅本地生产预设）。
  - Node POS 身份学/密码学模型重构（保持现有签名与 attestation 框架）。
  - 完整链上执行存储协议重写（仅收口状态持久化主路径）。

## 接口 / 数据
### 1) LLM 决策动作扩展
- 文件
  - `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
  - `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- 新增可解析决策（映射到 `simulator::Action`）
  - `buy_power` / `sell_power`
  - `place_power_order` / `cancel_power_order`
  - `publish_social_fact` / `challenge_social_fact` / `adjudicate_social_fact` / `revoke_social_fact` / `declare_social_edge`
- 约束
  - 所有数量字段必须 >0（或按语义允许 0 的字段按规则明确约束）。
  - owner 类字段统一复用 `self|agent:<id>|location:<id>` 解析。
  - 枚举字段（`power_order_side`、`social_adjudication_decision`）严格白名单。

### 2) DistFS 状态持久化主路径
- 文件
  - `crates/agent_world/src/runtime/world/persistence.rs`
  - `crates/agent_world/src/runtime/tests/persistence.rs`
- 新增 sidecar 元数据
  - `snapshot.manifest.json`
  - `journal.segments.json`
  - DistFS CAS root：`<world_dir>/.distfs-state/`
- 写入流程
  1. 保持原有 `snapshot.json/journal.json` 写入（兼容）。
  2. 默认执行 `segment_snapshot/segment_journal`，写入 sidecar 元数据。
  3. 对 sidecar 执行一次 assemble 验证，失败即返回错误。
- 恢复流程
  1. 若 sidecar 完整可读，则优先 DistFS assemble 恢复。
  2. DistFS 不可用时回退 `snapshot.json/journal.json`。

### 3) 去中心化默认拓扑（live）
- 文件
  - `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests.rs`
- 拓扑模式
  - `single`（兼容旧行为）
  - `triad`（默认，sequencer/storage/observer）
- triad 规则
  - 自动构建 3 节点 validator 集合与 gossip mesh。
  - 主节点（sequencer）作为 viewer 共识门控数据源。
  - 默认拒绝 `--no-node` 与 `--viewer-no-consensus-gate` 的组合退化到“无共识驱动”路径（需显式选择 `single` 模式才允许）。

## 里程碑
- M1：T0 文档冻结（设计+项目管理）。
- M2：T1 LLM 决策扩展完成并通过 required tests。
- M3：T2 DistFS 默认持久化主路径完成并通过 persistence tests。
- M4：T3 live 生产默认拓扑完成并通过 CLI/集成测试。
- M5：T4 全量回归（`cargo check` + 定向 required tests）与文档/devlog 收口。

## 风险
- 兼容风险：LLM schema 扩展可能影响现有提示词稳定性，需要保持旧动作兼容与明确报错。
- 恢复风险：DistFS sidecar 若损坏，必须保证 JSON 兜底路径可用，避免不可恢复。
- 运行风险：triad 默认会占用更多本地端口与资源，需要可配置 base port 并提供冲突报错。
- 测试风险：live 拓扑测试依赖网络端口，需使用随机可用端口策略降低 flaky。
