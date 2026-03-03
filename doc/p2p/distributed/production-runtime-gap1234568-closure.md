# Agent World Runtime：生产级收口（Gap 1/2/3/4/5/6/8）设计文档

## 目标
- 收口 Gap 1：将节点状态复制从“本地目录侧车”推进为“网络优先拉取 + 本地持久化兜底”。
- 收口 Gap 2：补齐生产可用的区块/Blob 交换协议，支持按高度与按内容哈希拉取。
- 收口 Gap 3：把共识提交与执行结果绑定为同一主路径，禁止提交高度先于执行高度落账。
- 收口 Gap 4：关闭默认全员自动代投票，默认改为“仅本节点投票”，其余票据来自网络。
- 收口 Gap 5：复制写入从 single-writer 升级为 epoch-based writer rotation，支持主写者切换。
- 收口 Gap 6：将存储挑战纳入节点共识主循环门控，提交前必须通过挑战校验。
- 收口 Gap 8：提升分布式网络能力为默认集成路径（构建与运行默认启用生产链路）。

## 范围

### In Scope
- `crates/agent_world_node`
  - `PosNodeEngine` 提交/执行强绑定（先执行后落账）。
  - 自动投票策略重构：默认仅本节点投票。
  - replication record/guard 升级到 writer epoch 语义。
  - 新增 replication block exchange 协议（按高度拉 commit、按哈希拉 blob）。
  - 新增网络优先补洞同步（gap sync）主路径。
  - 存储挑战门控纳入提交路径。
- `crates/agent_world`
  - `world_viewer_live` 默认节点参数对齐生产语义（不默认代投票，默认启用分布式复制通道）。
- `crates/agent_world_net`
  - 默认 feature 升级为生产网络能力优先。
- 测试
  - `test_tier_required` 覆盖新增协议、门控、writer failover、默认行为变更。

### Out of Scope
- 浏览器 wasm32 完整 libp2p 节点实现（本轮不做 Gap 7）。
- 共识算法本体重写（PoS 阈值模型保持）。
- 跨机自动发现/自动运维编排（保留静态配置 + 显式参数）。

## 接口 / 数据

### 1) Replication Writer Epoch
- `FileReplicationRecord` 新增：
  - `writer_epoch: u64`（`>=1`）
- `SingleWriterReplicationGuard` 新增：
  - `writer_epoch: u64`
- 规则：
  - 同 writer 同 epoch：`sequence` 严格递增。
  - writer 变更：`writer_epoch` 必须提升，且新 writer 首条 `sequence=1`。
  - 同 writer epoch 提升：允许重置序列（首条 `sequence=1`）。

### 2) Replication Block Exchange Protocols
- `"/aw/node/replication/fetch-commit/1.0.0"`
  - 请求：`{ world_id, height }`
  - 响应：`{ found, message }`，`message=GossipReplicationMessage`
- `"/aw/node/replication/fetch-blob/1.0.0"`
  - 请求：`{ content_hash }`
  - 响应：`{ found, blob }`

### 3) Commit/Execution Hard Binding
- 节点提交路径调整为：
  - `decision -> execution_hook(on_commit) -> apply_decision`
- 若需要执行绑定且执行失败：
  - 当前提案不落账。
  - 不推进 `committed_height`。

### 4) Storage Challenge Consensus Gate
- 在本地提交复制前执行：
  - 本地 `probe_storage_challenges`。
  - 网络 blob challenge（若启用网络）。
- 门控失败：
  - 拒绝本次提交复制并返回共识错误。

### 5) 运行默认
- `world_viewer_live` 默认：
  - `node_auto_attest_all_validators=false`
  - 默认构建启用分布式网络 feature（见里程碑）。

## 里程碑
- M1：T0 文档冻结（本文件 + 项管文档）。
- M2：T1 共识执行强绑定 + 默认投票策略收口。
- M3：T2 writer epoch failover 语义落地。
- M4：T3/T4 block exchange + 网络补洞主路径。
- M5：T5 存储挑战共识门控。
- M6：T6 默认网络集成收口。
- M7：T7 回归与文档/devlog 收口。

## 风险
- 行为变更风险：默认关闭全员自动代投票后，错误拓扑可能出现 pending 提案堆积。
- 兼容风险：writer epoch 引入后需保持旧快照/旧消息的 serde 兼容。
- 可用性风险：网络补洞与挑战门控增加网络依赖，需要可观测错误和安全回退。
- 稳定性风险：默认网络 feature 增加编译与运行负担，需要 required-tier 覆盖。
