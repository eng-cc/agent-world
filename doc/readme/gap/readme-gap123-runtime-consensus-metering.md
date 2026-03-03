# README 高优先级缺口收口（三期）：世界内编译 + 共识动作载荷 + WASM 运行计费（设计文档）

## 目标
- 收口缺口 1：补齐世界内 Rust -> WASM 编译闭环，不再只支持“链下编译后直接部署字节”。
- 收口缺口 2：让共识提交携带“有序动作载荷”，并驱动执行层按提交动作重放，而不是仅按高度空转 `step()`。
- 收口缺口 3：对 WASM 模块调用引入按次资源计费（算力/电力），并提供可审计记录与余额不足拒绝路径。

## 范围
- In scope
  - Runtime（`crates/agent_world/src/runtime`）
    - 新增 `CompileModuleArtifactFromSource` 动作，支持源文件包编译并注册 artifact。
    - 新增源文件编译器（临时工作目录 + 外部编译命令 + 默认脚本回退）。
    - 新增模块调用计费记录事件（world event），并在回放路径应用扣费。
  - Node（`crates/agent_world_node`）
    - 新增共识动作载荷类型与动作根（action root）计算。
    - 提案/提交/签名/复制链路携带动作序列与 action root。
    - 执行 hook 上下文携带已提交动作序列。
  - Viewer live 执行桥（`world_viewer_live`）
    - execution hook 改为“按提交动作解码并注入 runtime action，再 step”。
  - 测试
    - `test_tier_required`：覆盖 compile 成功/拒绝、动作根签名覆盖、执行 hook 动作透传、计费扣减/拒绝。
- Out of scope
  - 多节点全量 mempool 广播协议（本轮只做“节点可提交动作 + 共识提交携带动作”最小闭环）。
  - 完整链上 build sandbox 隔离（本轮采用受控外部命令 + 默认脚本）。
  - 新增 ResourceKind（算力沿用现有 `ResourceKind::Data` 作为 compute 计费口径）。

## 接口 / 数据
### 1) Runtime：世界内编译
- `Action` 新增：
  - `CompileModuleArtifactFromSource { publisher_agent_id, module_id, source_package }`
- 新增结构：
  - `ModuleSourcePackage { manifest_path, files: BTreeMap<String, Vec<u8>> }`
- 编译流程：
  1. 校验 agent、manifest path、文件路径合法性（拒绝绝对路径/`..`）。
  2. 将 source package 展开到临时目录。
  3. 优先执行 `AGENT_WORLD_MODULE_SOURCE_COMPILER`（可测试注入）。
  4. 若未配置则回退 `scripts/build-wasm-module.sh`。
  5. 读取产物 wasm，按现有 `DeployModuleArtifact` 路径注册与扣费。

### 2) Node：共识动作载荷
- 新增类型：
  - `NodeConsensusAction { action_id, payload_cbor, payload_hash }`
- 新增计算：
  - `compute_consensus_action_root(actions)`，对有序动作列表做确定性哈希。
- `PendingProposal` / `PosDecision` 扩展：
  - `action_root`
  - `committed_actions`
- `GossipProposalMessage` / `GossipCommitMessage` 扩展：
  - `action_root`
  - `actions`
- `ReplicatedCommitPayload` 扩展：
  - `action_root`
  - `actions`
- `NodeExecutionCommitContext` 扩展：
  - `action_root`
  - `committed_actions`

### 3) Runtime：模块按次计费
- 新增 world event：
  - `ModuleRuntimeCharged { module_id, trace_id, payer_agent_id, compute_fee_kind, compute_fee_amount, electricity_fee_kind, electricity_fee_amount, input_bytes, output_bytes, effect_count, emit_count }`
- 计费策略（确定性）：
  - `compute_fee_amount = ceil(input_bytes/1024) + ceil(output_bytes/1024) + effect_count*2 + emit_count`
  - `electricity_fee_amount = 1 + effect_count + emit_count + has_new_state`
  - `compute_fee_kind` 固定 `ResourceKind::Data`（算力代理），`electricity_fee_kind` 固定 `ResourceKind::Electricity`
- 扣费规则：
  - 仅当模块 artifact owner 可解析为现存 agent 时执行扣费。
  - 余额不足则模块调用失败（`ModuleCallFailed`），不应用输出。

## 里程碑
- M1：文档与任务拆解完成（T0）。
- M2：缺口 1（CompileModuleArtifactFromSource）实现 + required 测试通过。
- M3：缺口 2（共识动作载荷链路）实现 + required 测试通过。
- M4：缺口 3（按次计费）实现 + required 测试通过。
- M5：回归验证、项目文档状态回写、devlog 收口。

## 风险
- 工程风险：`agent_world_node` 主文件已接近 1200 行，需要同步拆分实现以满足文件长度约束。
- 兼容风险：gossip/replication 消息扩展后，旧消息兼容性依赖 `serde(default)`。
- 经济风险：计费引入新的失败路径，若测试默认资源不足会触发行为回归，需要补齐测试种子资源。
- 运行风险：默认编译脚本依赖本机 toolchain；需保留可注入编译器路径，保证 CI/测试稳定。
