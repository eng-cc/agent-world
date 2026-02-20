> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node PoS 主循环接入（设计文档）

## 目标
- 将现有 `node` 基础主循环升级为可驱动“以太坊风格 PoS head 共识”的节点模块。
- 将 crate 包名从 `node` 迁移为 `agent_world_node`，作为 `agent_world` 的基础模块直接调用。
- 保证在启动模拟（`world_viewer_live`）时自动启动节点，并持续推进 PoS proposal/attestation/commit。

## 范围

### In Scope
- 在 `crates/agent_world_node` 内实现 PoS 驱动节点主循环：
  - 每 tick 推进 slot。
  - 按 slot 选择 proposer。
  - 生成 world head proposal。
  - 自动补齐 attestation（可配置），在本地闭环提交 head。
- 节点运行时快照增强：输出共识模式、最新高度、提交高度、最后状态、最后错误。
- `agent_world` 启动链路接线到新包名 `agent_world_node`。
- 覆盖单元测试与启动参数解析测试。

### Out of Scope
- 多进程真实网络 gossip。
- 真实 BLS 聚合签名与经济惩罚执行。
- fork choice / finality 完整信标链流程。

## 接口 / 数据

### NodeConfig（升级后）
- 基础字段：`node_id`、`world_id`、`tick_interval`、`role`。
- PoS 字段：`pos_config`、`auto_attest_all_validators`。

### NodeSnapshot（升级后）
- 运行状态：`running`、`tick_count`、`last_tick_unix_ms`。
- 共识状态：`mode`、`latest_height`、`committed_height`、`slot`、`epoch`、`last_status`、`last_block_hash`。
- 诊断字段：`last_error`。

### NodeRuntime（升级后）
- `start()`：启动线程并推进 PoS。
- `stop()`：停止线程并回收资源。
- `snapshot()`：读取节点与 PoS 的即时状态。

## 里程碑
- NPOS-1：设计文档与项目管理文档落地。
- NPOS-2：重构 `crates/agent_world_node`，落地 PoS 驱动主循环并更名为 `agent_world_node` 包。
- NPOS-3：`world_viewer_live` 启动链路接线与测试更新。
- NPOS-4：回归测试、文档状态收口与 devlog 收口。

## 风险
- 单进程模拟“自动补齐 attestation”不等同真实分布式网络，需要在下一阶段补网络传播语义。
- 若 validator 集配置不当（如 stake 分布导致 proposer 期望与策略冲突），可能出现持续 pending。
- 包名迁移会影响下游引用与脚本命令，需要同步修正测试命令与文档。
